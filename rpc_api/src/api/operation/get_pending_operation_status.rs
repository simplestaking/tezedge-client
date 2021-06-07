use std::fmt::{self, Display};

use crate::BoxFuture;
use crate::api::{
    TransportError, GetPendingOperationsError,
    GetPendingOperations, GetPendingOperationsAsync,
    PendingOperation, PendingOperations,
};

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetPendingOperationStatusErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

impl From<GetPendingOperationsError> for GetPendingOperationStatusErrorKind {
    fn from(error: GetPendingOperationsError) -> Self {
        use GetPendingOperationsError::*;

        match error {
            Transport(err) => Self::Transport(err),
            Unknown(err) => Self::Unknown(err),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct GetPendingOperationStatusError {
    pub operation_hash: String,
    pub kind: GetPendingOperationStatusErrorKind,
}

impl Display for GetPendingOperationStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting pending operation status for operation with hash \"{}\" failed! Reason: {}",
            self.operation_hash,
            self.kind,
        )
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PendingOperationStatus {
    Applied,
    Refused,
    /// transaction is finished but not necessarily successful
    Finished,
}

pub type GetPendingOperationStatusResult = Result<PendingOperationStatus, GetPendingOperationStatusError>;

pub trait GetPendingOperationStatus {
    fn get_pending_operation_status(
        &self,
        operation_hash: &str,
    ) -> GetPendingOperationStatusResult;
}

pub trait GetPendingOperationStatusAsync {
    fn get_pending_operation_status<'a>(
        &'a self,
        operation_hash: &'a str,
    ) -> BoxFuture<'a, GetPendingOperationStatusResult>;
}

#[inline]
fn build_error<E>(op_hash: &str, kind: E) -> GetPendingOperationStatusError
    where E: Into<GetPendingOperationStatusErrorKind>,
{
    GetPendingOperationStatusError {
        operation_hash: op_hash.to_string(),
        kind: kind.into(),
    }
}

#[inline]
fn get_status_from_pending_operations(
    operation_hash: &str,
    pending_operations: PendingOperations,
) -> GetPendingOperationStatusResult
{
    let contained_by = |ops: &[PendingOperation]| {
        ops.iter()
            .find(|op| op.hash == operation_hash)
            .is_some()
    };

    let status = if contained_by(&pending_operations.applied) {
        PendingOperationStatus::Applied
    } else if contained_by(&pending_operations.refused) {
        PendingOperationStatus::Refused
    } else {
        PendingOperationStatus::Finished
    };

    Ok(status)
}

impl<T> GetPendingOperationStatus for T
    where T: GetPendingOperations,
{
    fn get_pending_operation_status(
        &self,
        operation_hash: &str,
    ) -> GetPendingOperationStatusResult
    {
        let pending_operations = self.get_pending_operations()
            .map_err(|err| build_error(operation_hash, err))?;

        get_status_from_pending_operations(operation_hash, pending_operations)
    }
}

impl<T> GetPendingOperationStatusAsync for T
    where T: GetPendingOperationsAsync,
{
    fn get_pending_operation_status<'a>(
        &'a self,
        operation_hash: &'a str,
    ) -> BoxFuture<'a, GetPendingOperationStatusResult>
    {
        Box::pin(async move {
            let pending_operations = self.get_pending_operations().await
                .map_err(|err| build_error(operation_hash, err))?;

            get_status_from_pending_operations(operation_hash, pending_operations)
        })
    }
}
