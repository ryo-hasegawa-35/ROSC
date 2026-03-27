use std::fs;
use std::path::PathBuf;

use rosc_osc::{
    CompatibilityMode, OscArgument, OscMessage, OscMessageView, ParsedOscPacket,
    ParsedOscPacketView, TypeTagSource, encode_packet, parse_packet, parse_packet_view,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Catalog {
    vectors: Vec<Vector>,
}

#[test]
fn borrowed_parser_keeps_string_and_blob_data_in_source_buffer() {
    let fixture = encode_packet(&ParsedOscPacket::Message(OscMessage {
        address: "/blob".to_owned(),
        type_tag_source: TypeTagSource::Explicit,
        arguments: vec![
            OscArgument::String("ok".to_owned()),
            OscArgument::Blob(vec![1, 2, 3, 4]),
        ],
    }))
    .unwrap();

    let parsed = parse_packet_view(&fixture, CompatibilityMode::Osc1_0Strict)
        .expect("borrowed parser should parse fixture");
    let ParsedOscPacketView::Message(OscMessageView {
        address, arguments, ..
    }) = parsed
    else {
        panic!("expected borrowed OSC message");
    };

    assert!(is_borrowed_from(
        address.as_ptr() as usize,
        address.len(),
        &fixture
    ));
    assert_eq!(arguments.len(), 2);
    match &arguments[0] {
        rosc_osc::OscArgumentView::String(value) => {
            assert!(is_borrowed_from(
                value.as_ptr() as usize,
                value.len(),
                &fixture
            ));
        }
        other => panic!("expected borrowed string argument, got {other:?}"),
    }
    match &arguments[1] {
        rosc_osc::OscArgumentView::Blob(value) => {
            assert!(is_borrowed_from(
                value.as_ptr() as usize,
                value.len(),
                &fixture
            ));
        }
        other => panic!("expected borrowed blob argument, got {other:?}"),
    }
}

#[derive(Debug, Deserialize)]
struct Vector {
    id: String,
    mode: CompatibilityMode,
    expected: String,
    fixture_path: String,
}

#[test]
fn conformance_catalog_matches_current_parser_behavior() {
    let catalog = read_catalog();
    assert_eq!(
        catalog.vectors.len(),
        7,
        "fixture coverage changed unexpectedly"
    );

    for vector in catalog.vectors {
        let fixture = read_fixture(&vector.fixture_path);
        let parsed = parse_packet(&fixture, vector.mode);

        match vector.id.as_str() {
            "osc10-strict-basic-int" => {
                let ParsedOscPacket::Message(message) =
                    parsed.expect("strict int fixture must parse")
                else {
                    panic!("expected parsed message");
                };
                assert_eq!(message.address, "/foo");
                assert_eq!(message.arguments, vec![OscArgument::Int32(42)]);
                assert_eq!(
                    encode_packet(&ParsedOscPacket::Message(message)).unwrap(),
                    fixture
                );
            }
            "osc10-strict-basic-string" => {
                let ParsedOscPacket::Message(message) =
                    parsed.expect("strict string fixture must parse")
                else {
                    panic!("expected parsed message");
                };
                assert_eq!(message.address, "/status");
                assert_eq!(
                    message.arguments,
                    vec![OscArgument::String("ok".to_owned())]
                );
                assert_eq!(
                    encode_packet(&ParsedOscPacket::Message(message)).unwrap(),
                    fixture
                );
            }
            "osc10-strict-basic-bundle" => {
                let ParsedOscPacket::Bundle(bundle) =
                    parsed.expect("strict bundle fixture must parse")
                else {
                    panic!("expected parsed bundle");
                };
                assert_eq!(bundle.elements.len(), 1);
                assert_eq!(
                    encode_packet(&ParsedOscPacket::Bundle(bundle)).unwrap(),
                    fixture
                );
            }
            "osc10-legacy-missing-type-tag" => {
                let ParsedOscPacket::LegacyUntypedMessage(message) =
                    parsed.expect("legacy missing type tag must be tolerated")
                else {
                    panic!("expected legacy untyped packet");
                };
                assert_eq!(message.address, "/legacy");
                assert_eq!(message.raw_argument_bytes, vec![0, 0, 0, 42]);
            }
            "osc10-malformed-unaligned-address" => {
                assert!(parsed.is_err(), "malformed fixture must be rejected");
            }
            "osc11-extended-boolean-true" => {
                let ParsedOscPacket::Message(message) =
                    parsed.expect("extended boolean fixture must parse")
                else {
                    panic!("expected parsed message");
                };
                assert_eq!(message.address, "/flag");
                assert_eq!(message.arguments, vec![OscArgument::True]);
                assert_eq!(
                    encode_packet(&ParsedOscPacket::Message(message)).unwrap(),
                    fixture
                );
            }
            "osc11-unknown-extension-tag" => {
                let ParsedOscPacket::Opaque(packet) =
                    parsed.expect("unknown extension should become opaque")
                else {
                    panic!("expected opaque packet");
                };
                assert_eq!(packet.address, "/exp");
            }
            other => panic!("unexpected fixture id {other}"),
        }

        if vector.expected == "reject" {
            assert!(parse_packet(&fixture, vector.mode).is_err());
        }
    }
}

fn read_catalog() -> Catalog {
    let path = repo_root().join("fixtures/conformance/catalog.json");
    let content = fs::read_to_string(path).expect("catalog should be readable");
    serde_json::from_str(&content).expect("catalog should be valid JSON")
}

fn read_fixture(relative_path: &str) -> Vec<u8> {
    let path = repo_root().join("fixtures/conformance").join(relative_path);
    let content = fs::read_to_string(path).expect("fixture should be readable");
    decode_hex(content.trim())
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert!(
        hex.len().is_multiple_of(2),
        "hex fixture length must be even"
    );
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let text = std::str::from_utf8(chunk).expect("hex bytes are valid UTF-8");
            u8::from_str_radix(text, 16).expect("fixture contains valid hex")
        })
        .collect()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has parent")
        .parent()
        .expect("workspace dir has parent")
        .to_path_buf()
}

fn is_borrowed_from(ptr: usize, len: usize, source: &[u8]) -> bool {
    let start = source.as_ptr() as usize;
    let end = start + source.len();
    ptr >= start && ptr + len <= end
}
