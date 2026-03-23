use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const BUNDLE_HEADER: &[u8; 8] = b"#bundle\0";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityMode {
    #[default]
    Osc1_0Strict,
    Osc1_0LegacyTolerant,
    Osc1_1Extended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeTagSource {
    Explicit,
    LegacyMissing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OscMessage {
    pub address: String,
    pub type_tag_source: TypeTagSource,
    pub arguments: Vec<OscArgument>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LegacyUntypedMessage {
    pub address: String,
    pub raw_argument_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OscBundle {
    pub timetag: u64,
    pub elements: Vec<ParsedOscPacket>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpaqueReason {
    UnsupportedTypeTag(char),
    UnsupportedExtension(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpaqueOscPacket {
    pub address: String,
    pub type_tag_text: Option<String>,
    pub reason: OpaqueReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParsedOscPacket {
    Message(OscMessage),
    Bundle(OscBundle),
    LegacyUntypedMessage(LegacyUntypedMessage),
    Opaque(OpaqueOscPacket),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OscArgument {
    Int32(i32),
    Float32(f32),
    String(String),
    Blob(Vec<u8>),
    Int64(i64),
    Timetag(u64),
    Double64(f64),
    Symbol(String),
    Char(char),
    Rgba(u32),
    Midi4([u8; 4]),
    True,
    False,
    Nil,
    Impulse,
    Array(Vec<OscArgument>),
}

impl OscArgument {
    fn type_tag(&self) -> Vec<u8> {
        match self {
            Self::Int32(_) => vec![b'i'],
            Self::Float32(_) => vec![b'f'],
            Self::String(_) => vec![b's'],
            Self::Blob(_) => vec![b'b'],
            Self::Int64(_) => vec![b'h'],
            Self::Timetag(_) => vec![b't'],
            Self::Double64(_) => vec![b'd'],
            Self::Symbol(_) => vec![b'S'],
            Self::Char(_) => vec![b'c'],
            Self::Rgba(_) => vec![b'r'],
            Self::Midi4(_) => vec![b'm'],
            Self::True => vec![b'T'],
            Self::False => vec![b'F'],
            Self::Nil => vec![b'N'],
            Self::Impulse => vec![b'I'],
            Self::Array(values) => {
                let mut tags = vec![b'['];
                for value in values {
                    tags.extend(value.type_tag());
                }
                tags.push(b']');
                tags
            }
        }
    }
}

impl ParsedOscPacket {
    pub fn address(&self) -> Option<&str> {
        match self {
            Self::Message(message) => Some(&message.address),
            Self::LegacyUntypedMessage(message) => Some(&message.address),
            Self::Opaque(message) => Some(&message.address),
            Self::Bundle(_) => None,
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("packet is shorter than required for the requested structure")]
    UnexpectedEof,
    #[error("packet is malformed: {0}")]
    Malformed(&'static str),
    #[error("missing type tag string in strict compatibility mode")]
    MissingTypeTag,
    #[error("unsupported OSC type tag `{0}` for the active compatibility mode")]
    UnsupportedTypeTag(char),
    #[error("invalid UTF-8 in OSC string field")]
    InvalidUtf8,
}

#[derive(Debug, Error, PartialEq)]
pub enum EncodeError {
    #[error("opaque packets must be forwarded from their retained raw bytes")]
    OpaquePacketNotEncodable,
    #[error("OSC array nesting is not balanced")]
    UnbalancedArray,
    #[error("OSC char argument is not a valid scalar value")]
    InvalidCharValue,
}

pub fn parse_packet(bytes: &[u8], mode: CompatibilityMode) -> Result<ParsedOscPacket, ParseError> {
    if bytes.starts_with(BUNDLE_HEADER) {
        parse_bundle(bytes, mode)
    } else {
        parse_message(bytes, mode)
    }
}

pub fn encode_packet(packet: &ParsedOscPacket) -> Result<Vec<u8>, EncodeError> {
    match packet {
        ParsedOscPacket::Message(message) => encode_message(message),
        ParsedOscPacket::Bundle(bundle) => encode_bundle(bundle),
        ParsedOscPacket::LegacyUntypedMessage(message) => {
            let mut bytes = encode_osc_string(&message.address);
            bytes.extend_from_slice(&message.raw_argument_bytes);
            Ok(bytes)
        }
        ParsedOscPacket::Opaque(_) => Err(EncodeError::OpaquePacketNotEncodable),
    }
}

pub fn encode_message(message: &OscMessage) -> Result<Vec<u8>, EncodeError> {
    let mut bytes = encode_osc_string(&message.address);
    let mut type_tags = Vec::from([b',']);
    for argument in &message.arguments {
        type_tags.extend(argument.type_tag());
    }
    bytes.extend(encode_osc_string_bytes(&type_tags));
    for argument in &message.arguments {
        encode_argument(argument, &mut bytes)?;
    }
    Ok(bytes)
}

pub fn encode_bundle(bundle: &OscBundle) -> Result<Vec<u8>, EncodeError> {
    let mut bytes = BUNDLE_HEADER.to_vec();
    bytes.extend_from_slice(&bundle.timetag.to_be_bytes());

    for element in &bundle.elements {
        let encoded = encode_packet(element)?;
        bytes.extend_from_slice(&(encoded.len() as i32).to_be_bytes());
        bytes.extend_from_slice(&encoded);
    }

    Ok(bytes)
}

fn parse_bundle(bytes: &[u8], mode: CompatibilityMode) -> Result<ParsedOscPacket, ParseError> {
    if bytes.len() < 16 {
        return Err(ParseError::UnexpectedEof);
    }

    let timetag = u64::from_be_bytes(bytes[8..16].try_into().expect("slice length is checked"));
    let mut cursor = 16;
    let mut elements = Vec::new();

    while cursor < bytes.len() {
        if cursor + 4 > bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let element_len = i32::from_be_bytes(
            bytes[cursor..cursor + 4]
                .try_into()
                .expect("slice length is checked"),
        );
        cursor += 4;
        if element_len < 0 {
            return Err(ParseError::Malformed(
                "bundle element length must be non-negative",
            ));
        }
        let end = cursor + element_len as usize;
        if end > bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }

        elements.push(parse_packet(&bytes[cursor..end], mode)?);
        cursor = end;
    }

    Ok(ParsedOscPacket::Bundle(OscBundle { timetag, elements }))
}

fn parse_message(bytes: &[u8], mode: CompatibilityMode) -> Result<ParsedOscPacket, ParseError> {
    let (address, cursor) = parse_osc_string(bytes, 0)?;
    if cursor >= bytes.len() || bytes[cursor] != b',' {
        return match mode {
            CompatibilityMode::Osc1_0Strict => Err(ParseError::MissingTypeTag),
            CompatibilityMode::Osc1_0LegacyTolerant | CompatibilityMode::Osc1_1Extended => Ok(
                ParsedOscPacket::LegacyUntypedMessage(LegacyUntypedMessage {
                    address,
                    raw_argument_bytes: bytes[cursor..].to_vec(),
                }),
            ),
        };
    }

    let (type_tag_text, mut cursor) = parse_osc_string(bytes, cursor)?;
    if !type_tag_text.starts_with(',') {
        return Err(ParseError::Malformed(
            "type tag string must start with a comma",
        ));
    }

    let type_tags = type_tag_text.as_bytes();
    if let Some(unsupported) = first_unsupported_tag(type_tags) {
        return match mode {
            CompatibilityMode::Osc1_1Extended => Ok(ParsedOscPacket::Opaque(OpaqueOscPacket {
                address,
                type_tag_text: Some(type_tag_text),
                reason: OpaqueReason::UnsupportedTypeTag(unsupported),
            })),
            _ => Err(ParseError::UnsupportedTypeTag(unsupported)),
        };
    }

    let mut tag_cursor = 1usize;
    let arguments = parse_arguments(type_tags, &mut tag_cursor, &mut cursor, bytes, false)?;
    if tag_cursor != type_tags.len() {
        return Err(ParseError::Malformed(
            "type tag parser did not consume the full tag string",
        ));
    }
    if cursor != bytes.len() {
        return Err(ParseError::Malformed(
            "packet contains trailing bytes after arguments",
        ));
    }

    Ok(ParsedOscPacket::Message(OscMessage {
        address,
        type_tag_source: TypeTagSource::Explicit,
        arguments,
    }))
}

fn parse_arguments(
    type_tags: &[u8],
    tag_cursor: &mut usize,
    cursor: &mut usize,
    bytes: &[u8],
    in_array: bool,
) -> Result<Vec<OscArgument>, ParseError> {
    let mut arguments = Vec::new();

    while *tag_cursor < type_tags.len() {
        let tag = type_tags[*tag_cursor] as char;
        *tag_cursor += 1;

        match tag {
            'i' => arguments.push(OscArgument::Int32(read_i32(bytes, cursor)?)),
            'f' => arguments.push(OscArgument::Float32(f32::from_bits(read_u32(
                bytes, cursor,
            )?))),
            's' => {
                let (value, next_cursor) = parse_osc_string(bytes, *cursor)?;
                *cursor = next_cursor;
                arguments.push(OscArgument::String(value));
            }
            'b' => arguments.push(OscArgument::Blob(parse_blob(bytes, cursor)?)),
            'h' => arguments.push(OscArgument::Int64(read_i64(bytes, cursor)?)),
            't' => arguments.push(OscArgument::Timetag(read_u64(bytes, cursor)?)),
            'd' => arguments.push(OscArgument::Double64(f64::from_bits(read_u64(
                bytes, cursor,
            )?))),
            'S' => {
                let (value, next_cursor) = parse_osc_string(bytes, *cursor)?;
                *cursor = next_cursor;
                arguments.push(OscArgument::Symbol(value));
            }
            'c' => {
                let scalar = read_u32(bytes, cursor)?;
                let value = char::from_u32(scalar)
                    .ok_or(ParseError::Malformed("invalid char scalar value"))?;
                arguments.push(OscArgument::Char(value));
            }
            'r' => arguments.push(OscArgument::Rgba(read_u32(bytes, cursor)?)),
            'm' => arguments.push(OscArgument::Midi4(read_midi(bytes, cursor)?)),
            'T' => arguments.push(OscArgument::True),
            'F' => arguments.push(OscArgument::False),
            'N' => arguments.push(OscArgument::Nil),
            'I' => arguments.push(OscArgument::Impulse),
            '[' => {
                let nested = parse_arguments(type_tags, tag_cursor, cursor, bytes, true)?;
                arguments.push(OscArgument::Array(nested));
            }
            ']' => {
                if in_array {
                    return Ok(arguments);
                }
                return Err(ParseError::Malformed("unexpected array terminator"));
            }
            unknown => return Err(ParseError::UnsupportedTypeTag(unknown)),
        }
    }

    if in_array {
        Err(ParseError::Malformed("unterminated OSC array"))
    } else {
        Ok(arguments)
    }
}

fn first_unsupported_tag(type_tags: &[u8]) -> Option<char> {
    type_tags
        .iter()
        .skip(1)
        .map(|byte| *byte as char)
        .find(|tag| {
            !matches!(
                tag,
                'i' | 'f'
                    | 's'
                    | 'b'
                    | 'h'
                    | 't'
                    | 'd'
                    | 'S'
                    | 'c'
                    | 'r'
                    | 'm'
                    | 'T'
                    | 'F'
                    | 'N'
                    | 'I'
                    | '['
                    | ']'
            )
        })
}

fn parse_blob(bytes: &[u8], cursor: &mut usize) -> Result<Vec<u8>, ParseError> {
    let len = read_i32(bytes, cursor)?;
    if len < 0 {
        return Err(ParseError::Malformed("blob length must be non-negative"));
    }
    let end = *cursor + len as usize;
    if end > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let blob = bytes[*cursor..end].to_vec();
    *cursor = align_four(end);
    if *cursor > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    Ok(blob)
}

fn parse_osc_string(bytes: &[u8], cursor: usize) -> Result<(String, usize), ParseError> {
    let Some(relative_end) = bytes[cursor..].iter().position(|byte| *byte == 0) else {
        return Err(ParseError::UnexpectedEof);
    };
    let end = cursor + relative_end;
    let padded_end = align_four(end + 1);
    if padded_end > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    if bytes[end + 1..padded_end].iter().any(|byte| *byte != 0) {
        return Err(ParseError::Malformed(
            "OSC string padding bytes must be zero",
        ));
    }
    let text = std::str::from_utf8(&bytes[cursor..end]).map_err(|_| ParseError::InvalidUtf8)?;
    Ok((text.to_owned(), padded_end))
}

fn encode_argument(argument: &OscArgument, bytes: &mut Vec<u8>) -> Result<(), EncodeError> {
    match argument {
        OscArgument::Int32(value) => bytes.extend_from_slice(&value.to_be_bytes()),
        OscArgument::Float32(value) => bytes.extend_from_slice(&value.to_bits().to_be_bytes()),
        OscArgument::String(value) | OscArgument::Symbol(value) => {
            bytes.extend(encode_osc_string(value))
        }
        OscArgument::Blob(value) => {
            bytes.extend_from_slice(&(value.len() as i32).to_be_bytes());
            bytes.extend_from_slice(value);
            pad_to_four(bytes);
        }
        OscArgument::Int64(value) => bytes.extend_from_slice(&value.to_be_bytes()),
        OscArgument::Timetag(value) => bytes.extend_from_slice(&value.to_be_bytes()),
        OscArgument::Double64(value) => bytes.extend_from_slice(&value.to_bits().to_be_bytes()),
        OscArgument::Char(value) => bytes.extend_from_slice(&(*value as u32).to_be_bytes()),
        OscArgument::Rgba(value) => bytes.extend_from_slice(&value.to_be_bytes()),
        OscArgument::Midi4(value) => bytes.extend_from_slice(value),
        OscArgument::True | OscArgument::False | OscArgument::Nil | OscArgument::Impulse => {}
        OscArgument::Array(values) => {
            for value in values {
                encode_argument(value, bytes)?;
            }
        }
    }
    Ok(())
}

fn encode_osc_string(value: &str) -> Vec<u8> {
    encode_osc_string_bytes(value.as_bytes())
}

fn encode_osc_string_bytes(value: &[u8]) -> Vec<u8> {
    let mut bytes = value.to_vec();
    bytes.push(0);
    pad_to_four(&mut bytes);
    bytes
}

fn pad_to_four(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

fn align_four(value: usize) -> usize {
    (value + 3) & !3
}

fn read_i32(bytes: &[u8], cursor: &mut usize) -> Result<i32, ParseError> {
    Ok(read_u32(bytes, cursor)? as i32)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, ParseError> {
    if *cursor + 4 > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let value = u32::from_be_bytes(
        bytes[*cursor..*cursor + 4]
            .try_into()
            .expect("slice length is checked"),
    );
    *cursor += 4;
    Ok(value)
}

fn read_i64(bytes: &[u8], cursor: &mut usize) -> Result<i64, ParseError> {
    Ok(read_u64(bytes, cursor)? as i64)
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, ParseError> {
    if *cursor + 8 > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let value = u64::from_be_bytes(
        bytes[*cursor..*cursor + 8]
            .try_into()
            .expect("slice length is checked"),
    );
    *cursor += 8;
    Ok(value)
}

fn read_midi(bytes: &[u8], cursor: &mut usize) -> Result<[u8; 4], ParseError> {
    if *cursor + 4 > bytes.len() {
        return Err(ParseError::UnexpectedEof);
    }
    let value = bytes[*cursor..*cursor + 4]
        .try_into()
        .expect("slice length is checked");
    *cursor += 4;
    Ok(value)
}

impl fmt::Display for CompatibilityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Osc1_0Strict => write!(f, "osc1_0_strict"),
            Self::Osc1_0LegacyTolerant => write!(f, "osc1_0_legacy_tolerant"),
            Self::Osc1_1Extended => write!(f, "osc1_1_extended"),
        }
    }
}
