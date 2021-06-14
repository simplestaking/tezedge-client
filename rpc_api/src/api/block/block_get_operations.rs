use std::fmt::{self, Display};
use serde::{Deserialize, Deserializer};

use types::{
    Address, ImplicitAddress,
    PublicKey, BlockHash, OperationHash, FromPrefixedBase58CheckError,
};
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum BlockGetOperationsError {
    Transport(#[from] TransportError),
    Base58Decode(#[from] FromPrefixedBase58CheckError),
    Unknown(String),
}

impl Display for BlockGetOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "getting head block hash failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Base58Decode(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockOperationContentReveal {
    pub source: ImplicitAddress,
    pub public_key: PublicKey,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockOperationContentTransaction {
    pub source: ImplicitAddress,
    pub destination: Address,
    #[serde(with = "utils::serde_amount")]
    pub amount: u64,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockOperationContentDelegation {
    pub source: ImplicitAddress,
    pub delegate: Option<ImplicitAddress>,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockOperationContentOrigination {
    pub source: ImplicitAddress,
    pub public_key: PublicKey,
    #[serde(with = "utils::serde_amount")]
    pub balance: u64,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
    pub script: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum BlockOperationContent {
    Reveal(BlockOperationContentReveal),
    Transaction(BlockOperationContentTransaction),
    Delegation(BlockOperationContentDelegation),
    Origination(BlockOperationContentOrigination),
    Other,
}

impl<'de> Deserialize<'de> for BlockOperationContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        #[allow(non_camel_case_types)]
        #[derive(Deserialize)]
        #[serde(tag = "kind")]
        enum WithKind {
            reveal(BlockOperationContentReveal),
            transaction(BlockOperationContentTransaction),
            delegation(BlockOperationContentDelegation),
            origination(BlockOperationContentOrigination),
            #[serde(other)]
            other,
        }

        Ok(match WithKind::deserialize(deserializer)? {
            WithKind::reveal(op) => Self::Reveal(op),
            WithKind::transaction(op) => Self::Transaction(op),
            WithKind::delegation(op) => Self::Delegation(op),
            WithKind::origination(op) => Self::Origination(op),
            WithKind::other => Self::Other,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockOperation {
    hash: OperationHash,
    contents: Vec<BlockOperationContent>,
}

pub type BlockGetOperationsResult = Result<Vec<BlockOperation>, BlockGetOperationsError>;

pub trait BlockGetOperations {
    /// Get head block's hash.
    fn block_get_operations(&self) -> BlockGetOperationsResult;
}

pub trait BlockGetOperationsAsync {
    /// Get head block's hash.
    fn block_get_operations(&self, block: &BlockHash) -> BoxFuture<'static, BlockGetOperationsResult>;
}

pub(crate) fn block_get_operations_url(base_url: &str, block_hash: &str) -> String {
    format!("{}/chains/main/blocks/{}/operations", base_url, block_hash)
}
