use std::fmt::{self, Display};

use types::NewOperationGroup;
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum PreapplyOperationsError {
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for PreapplyOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "preapplying operation failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

pub type PreapplyOperationsResult = Result<serde_json::Value, PreapplyOperationsError>;

pub trait PreapplyOperations {
    fn preapply_operations(
        &self,
        operation_group: &NewOperationGroup,
        signature: &str,
    ) -> PreapplyOperationsResult;
}

pub trait PreapplyOperationsAsync {
    fn preapply_operations<'a>(
        &'a self,
        operation_group: &'a NewOperationGroup,
        signature: &'a str,
    ) -> BoxFuture<'a, PreapplyOperationsResult>;
}

pub(crate) fn preapply_operations_url(base_url: &str) -> String {
    format!("{}/chains/main/blocks/head/helpers/preapply/operations", base_url)
}
