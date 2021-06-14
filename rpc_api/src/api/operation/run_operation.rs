use std::fmt::{self, Display};
use serde::Deserialize;
use serde::de;

use types::NewOperationGroup;
use crate::BoxFuture;
use crate::api::{TransportError, GetChainIDError};

#[derive(thiserror::Error, Debug)]
pub enum RunOperationError {
    Transport(#[from] TransportError),
    GetChainID(#[from] GetChainIDError),
    Unknown(String),
}

impl Display for RunOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation simulation failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::GetChainID(err) => write!(f, "\n{}", err),
            Self::Unknown(err) => write!(f, "Unknown! {}", err),
        }
    }
}

#[derive(Deserialize)]
pub struct RunOperationJson {
    pub contents: RunOperationContents,
}

impl Into<RunOperationContents> for RunOperationJson {
    fn into(self) -> RunOperationContents {
        self.contents
    }
}

pub type RunOperationContents = Vec<RunOperationContent>;

pub struct RunOperationContent {
    pub kind: String,
    pub consumed_gas: u64,
}

impl<'de> Deserialize<'de> for RunOperationContent {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawOperationResult {
            kind: String,
            metadata: Metadata,
        }

        #[derive(Deserialize)]
        struct Metadata {
            operation_result: MetadataResult,
        }

        #[derive(Deserialize)]
        struct MetadataResult {
            #[serde(with = "utils::serde_str")]
            consumed_gas: u64,
        }

        let result = RawOperationResult::deserialize(d)?;

        Ok(Self {
            kind: result.kind,
            consumed_gas: result.metadata.operation_result.consumed_gas,
        })
    }
}

pub type RunOperationResult = Result<RunOperationContents, RunOperationError>;

pub trait RunOperation {
    /// Simulate an operation.
    ///
    /// Useful for calculating fees as is returns estimated consumed gas,
    /// and it doesn't require signing the operation first.
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> RunOperationResult;
}

pub trait RunOperationAsync {
    /// Simulate an operation.
    ///
    /// Useful for calculating fees as is returns estimated consumed gas,
    /// and it doesn't require signing the operation first.
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> BoxFuture<'static, RunOperationResult>;
}

pub(crate) fn run_operation_url(base_url: &str) -> String {
    format!("{}/chains/main/blocks/head/helpers/scripts/run_operation", base_url)
}
