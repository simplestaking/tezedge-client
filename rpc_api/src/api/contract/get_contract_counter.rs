use std::fmt::{self, Display};

use types::ImplicitAddress;
use crypto::ToBase58Check;
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractCounterErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractCounterError {
    pub address: ImplicitAddress,
    pub kind: GetContractCounterErrorKind,
}

impl GetContractCounterError {
    pub(crate) fn new<E>(address: &ImplicitAddress, kind: E) -> Self
        where E: Into<GetContractCounterErrorKind>,
    {
        Self {
            address: address.clone(),
            kind: kind.into(),
        }
    }
}

impl Display for GetContractCounterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting counter for address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractCounterResult = Result<u64, GetContractCounterError>;

pub trait GetContractCounter {
    /// Get counter for a contract.
    fn get_contract_counter(&self, address: &ImplicitAddress) -> GetContractCounterResult;
}

pub trait GetContractCounterAsync {
    /// Get counter for a contract.
    fn get_contract_counter(
        &self,
        address: &ImplicitAddress,
    ) -> BoxFuture<'static, GetContractCounterResult>;
}

pub(crate) fn get_contract_counter_url(base_url: &str, addr: &ImplicitAddress) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/counter",
        base_url,
        addr.to_base58check(),
    )
}
