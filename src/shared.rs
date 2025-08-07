use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;
use zerocopy::*;
use zerocopy_derive::*;

/// Errors this crate can return
#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("database error")]
    Sled(#[from] sled::Error),
    #[error("connection error")]
    Subxt(#[from] subxt::Error),
    #[error("connection error")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("parse error")]
    Hex(#[from] hex::FromHexError),
    #[error("parse error")]
    ParseError,
    #[error("connection error")]
    BlockNotFound(u32),
    #[error("RPC error")]
    RpcError(#[from] subxt::ext::subxt_rpcs::Error),
    #[error("codec error")]
    CodecError(#[from] subxt::ext::codec::Error),
    #[error("metadata error")]
    MetadataError(#[from] subxt::error::MetadataTryFromError),
}

/// On-disk format for span value
#[derive(FromBytes, IntoBytes, Unaligned, PartialEq, Debug, Immutable)]
#[repr(C)]
pub struct SpanDbValue {
    pub start: U32<BigEndian>,
    // pub version: U16<BigEndian>,
}

/// On-disk format for span value
#[derive(FromBytes, IntoBytes, Unaligned, PartialEq, Debug, Immutable)]
#[repr(C)]
pub struct FeatherDbKey {
    pub block_number: U32<BigEndian>,
    pub index: U16<BigEndian>,
    pub account_id: [u8; 32],
}

/// Start and end block number for a span of blocks
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}
