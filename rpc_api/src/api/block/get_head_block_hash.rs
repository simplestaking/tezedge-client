use std::fmt::{self, Display};

use types::{BlockHash, FromPrefixedBase58CheckError};
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum GetHeadBlockHashError {
    Transport(#[from] TransportError),
    Base58Decode(#[from] FromPrefixedBase58CheckError),
    Unknown(String),
}

impl Display for GetHeadBlockHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "getting head block hash failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Base58Decode(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

pub type GetHeadBlockHashResult = Result<BlockHash, GetHeadBlockHashError>;

pub trait GetHeadBlockHash {
    /// Get head block's hash.
    fn get_head_block_hash(&self) -> GetHeadBlockHashResult;
}

pub trait GetHeadBlockHashAsync {
    /// Get head block's hash.
    fn get_head_block_hash(&self) -> BoxFuture<'static, GetHeadBlockHashResult>;
}

pub(crate) fn get_head_block_hash_url(base_url: &str) -> String {
    format!("{}/chains/main/blocks/head/hash", base_url)
}
