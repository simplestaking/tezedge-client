use serde::Serialize;

use crate::utils::estimate_operation_fee;
use super::{NewRevealOperation, NewTransactionOperation, NewDelegationOperation};

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum NewOperation {
    Reveal(NewRevealOperation),
    Transaction(NewTransactionOperation),
    Delegation(NewDelegationOperation),
}

impl NewOperation {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Reveal(_) => "reveal",
            Self::Transaction(_) => "transaction",
            Self::Delegation(_) => "delegation",
        }
    }

    pub fn get_fee(&self) -> u64 {
        match self {
            Self::Reveal(op) => op.fee,
            Self::Transaction(op) => op.fee,
            Self::Delegation(op) => op.fee,
        }
    }

    pub fn set_fee(&mut self, fee: u64) {
        match self {
            Self::Reveal(op) => op.fee = fee,
            Self::Transaction(op) => op.fee = fee,
            Self::Delegation(op) => op.fee = fee,
        }
    }

    pub fn estimate_bytes(&self) -> u64 {
        match self {
            Self::Reveal(op) => op.estimate_bytes(),
            Self::Transaction(op) => op.estimate_bytes(),
            Self::Delegation(op) => op.estimate_bytes(),
        }
    }

    pub fn estimate_fee(
        &self,
        base_fee: u64,
        ntez_per_byte: u64,
        ntez_per_gas: u64,
        estimated_gas: u64,
    ) -> u64 {
        estimate_operation_fee(
            base_fee,
            ntez_per_byte,
            ntez_per_gas,
            estimated_gas,
            self.estimate_bytes(),
        )
    }
}

impl From<NewRevealOperation> for NewOperation {
    fn from(op: NewRevealOperation) -> Self {
        Self::Reveal(op)
    }
}

impl From<NewTransactionOperation> for NewOperation {
    fn from(op: NewTransactionOperation) -> Self {
        Self::Transaction(op)
    }
}

impl From<NewDelegationOperation> for NewOperation {
    fn from(op: NewDelegationOperation) -> Self {
        Self::Delegation(op)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct NewOperationWithKind {
    kind: String,

    #[serde(flatten)]
    operation: NewOperation,
}

impl From<NewOperation> for NewOperationWithKind {
    fn from(op: NewOperation) -> Self {
        Self {
            kind: op.kind_str().to_owned(),
            operation: op,
        }
    }
}
