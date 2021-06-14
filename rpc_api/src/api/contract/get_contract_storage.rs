use std::fmt::{self, Display};

use types::OriginatedAddress;
use crypto::ToBase58Check;
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractStorageErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractStorageError {
    pub address: OriginatedAddress,
    pub kind: GetContractStorageErrorKind,
}

impl GetContractStorageError {
    pub(crate) fn new<E>(address: &OriginatedAddress, kind: E) -> Self
        where E: Into<GetContractStorageErrorKind>,
    {
        Self {
            address: address.clone(),
            kind: kind.into(),
        }
    }
}

impl Display for GetContractStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting storage for contract with an address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractStorageResult = Result<serde_json::Value, GetContractStorageError>;

pub trait GetContractStorage {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult;
}

pub trait GetContractStorageAsync {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> BoxFuture<'static, GetContractStorageResult>;
}

/// Get manager key
pub(crate) fn get_contract_storage_url(base_url: &str, addr: &OriginatedAddress) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/storage",
        base_url,
        addr.to_base58check(),
    )
}
