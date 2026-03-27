use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::SystemTime;

use bitflags::bitflags;
use rosc_osc::{
    CompatibilityMode, EncodeError, OpaqueReason, ParseError, ParsedOscPacket, encode_message,
    parse_packet,
};
use thiserror::Error;

static NEXT_PACKET_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PacketId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportKind {
    OscUdp,
    OscTcp,
    OscSlip,
    WsJson,
    Mqtt,
    Ipc,
    Internal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngressMetadata {
    pub ingress_id: String,
    pub transport: TransportKind,
    pub source_endpoint: Option<String>,
    pub compatibility_mode: CompatibilityMode,
    pub received_at: SystemTime,
}

#[derive(Clone, Debug)]
pub struct PacketEnvelope {
    pub packet_id: PacketId,
    pub raw_bytes: Arc<[u8]>,
    pub metadata: IngressMetadata,
    pub parsed: ParsedOscPacket,
    pub capabilities: PacketCapabilities,
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct PacketCapabilities: u32 {
        const FORWARDABLE = 0b0000_0001;
        const INSPECTABLE_ADDRESS = 0b0000_0010;
        const INSPECTABLE_ARGUMENTS = 0b0000_0100;
        const TRANSFORMABLE = 0b0000_1000;
        const CACHEABLE_CANDIDATE = 0b0001_0000;
        const REPLAYABLE = 0b0010_0000;
        const SECURITY_CHECKED = 0b0100_0000;
    }
}

#[derive(Debug, Error)]
pub enum PacketBuildError {
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum PacketTransformError {
    #[error("packet does not expose an addressable message that can be transformed")]
    NotTransformable,
    #[error(transparent)]
    Encode(#[from] EncodeError),
}

impl PacketEnvelope {
    pub fn parse_osc(
        raw_bytes: impl Into<Arc<[u8]>>,
        metadata: IngressMetadata,
    ) -> Result<Self, PacketBuildError> {
        let raw_bytes = raw_bytes.into();
        let parsed = parse_packet(&raw_bytes, metadata.compatibility_mode)?;
        let capabilities = capabilities_for(&parsed);

        Ok(Self {
            packet_id: PacketId(NEXT_PACKET_ID.fetch_add(1, Ordering::Relaxed)),
            raw_bytes,
            metadata,
            parsed,
            capabilities,
        })
    }

    pub fn address(&self) -> Option<&str> {
        self.parsed.address()
    }

    pub fn is_forwardable(&self) -> bool {
        self.capabilities.contains(PacketCapabilities::FORWARDABLE)
    }

    pub fn derive_with_renamed_address(
        &self,
        new_address: impl Into<String>,
    ) -> Result<Self, PacketTransformError> {
        let new_address = new_address.into();
        let rosc_osc::ParsedOscPacket::Message(message) = &self.parsed else {
            return Err(PacketTransformError::NotTransformable);
        };

        if !self
            .capabilities
            .contains(PacketCapabilities::TRANSFORMABLE)
        {
            return Err(PacketTransformError::NotTransformable);
        }

        let mut derived_message = message.clone();
        derived_message.address = new_address;
        let encoded = encode_message(&derived_message)?;
        let raw_bytes: Arc<[u8]> = encoded.into();

        Ok(Self {
            packet_id: PacketId(NEXT_PACKET_ID.fetch_add(1, Ordering::Relaxed)),
            raw_bytes: raw_bytes.clone(),
            metadata: self.metadata.clone(),
            parsed: ParsedOscPacket::Message(derived_message.clone()),
            capabilities: capabilities_for(&ParsedOscPacket::Message(derived_message)),
        })
    }

    pub fn clone_with_metadata(&self, metadata: IngressMetadata) -> Self {
        Self {
            packet_id: PacketId(NEXT_PACKET_ID.fetch_add(1, Ordering::Relaxed)),
            raw_bytes: Arc::clone(&self.raw_bytes),
            metadata,
            parsed: self.parsed.clone(),
            capabilities: self.capabilities,
        }
    }
}

fn capabilities_for(parsed: &ParsedOscPacket) -> PacketCapabilities {
    match parsed {
        ParsedOscPacket::Message(_) => {
            PacketCapabilities::FORWARDABLE
                | PacketCapabilities::INSPECTABLE_ADDRESS
                | PacketCapabilities::INSPECTABLE_ARGUMENTS
                | PacketCapabilities::TRANSFORMABLE
                | PacketCapabilities::CACHEABLE_CANDIDATE
                | PacketCapabilities::REPLAYABLE
        }
        ParsedOscPacket::Bundle(_) => {
            PacketCapabilities::FORWARDABLE | PacketCapabilities::REPLAYABLE
        }
        ParsedOscPacket::LegacyUntypedMessage(_) => {
            PacketCapabilities::FORWARDABLE
                | PacketCapabilities::INSPECTABLE_ADDRESS
                | PacketCapabilities::REPLAYABLE
        }
        ParsedOscPacket::Opaque(opaque) => {
            let mut capabilities = PacketCapabilities::FORWARDABLE
                | PacketCapabilities::INSPECTABLE_ADDRESS
                | PacketCapabilities::REPLAYABLE;
            if matches!(opaque.reason, OpaqueReason::UnsupportedExtension(_)) {
                capabilities |= PacketCapabilities::SECURITY_CHECKED;
            }
            capabilities
        }
    }
}
