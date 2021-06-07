use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};

use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum GetPendingOperationsError {
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for GetPendingOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "getting pending operations from mempool failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct PendingOperation {
    pub hash: String,
}

#[derive(Serialize, PartialEq, Debug, Default, Clone)]
pub struct PendingOperations {
    pub applied: Vec<PendingOperation>,
    pub refused: Vec<PendingOperation>,
}

pub type GetPendingOperationsResult = Result<PendingOperations, GetPendingOperationsError>;

pub trait GetPendingOperations {
    /// Get pending operations from mempool.
    fn get_pending_operations(&self) -> GetPendingOperationsResult;
}

pub trait GetPendingOperationsAsync {
    /// Get pending operations from mempool.
    fn get_pending_operations<'a>(&'a self) -> BoxFuture<'a, GetPendingOperationsResult>;
}

pub(crate) fn get_pending_operations_url(base_url: &str) -> String {
    format!("{}/chains/main/mempool/pending_operations", base_url)
}
