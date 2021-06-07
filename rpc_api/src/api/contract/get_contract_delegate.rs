use std::fmt::{self, Display};

use types::{Address, ImplicitAddress};
use crypto::ToBase58Check;
use crate::BoxFuture;
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractDelegateErrorKind {
    Transport(#[from] TransportError),
    #[error("Unknown! {0}")]
    Unknown(String),
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractDelegateError {
    pub address: Address,
    pub kind: GetContractDelegateErrorKind,
}

impl GetContractDelegateError {
    pub(crate) fn new<E>(address: &Address, kind: E) -> Self
        where E: Into<GetContractDelegateErrorKind>,
    {
        Self {
            address: address.clone(),
            kind: kind.into(),
        }
    }
}

impl Display for GetContractDelegateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting delegate for address \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

pub type GetContractDelegateResult = Result<Option<ImplicitAddress>, GetContractDelegateError>;

pub trait GetContractDelegate {
    /// Get active delegate for a contract.
    fn get_contract_delegate(&self, address: &Address) -> GetContractDelegateResult;
}

pub trait GetContractDelegateAsync {
    /// Get active delegate for a contract.
    fn get_contract_delegate<'a>(
        &'a self,
        address: &'a Address,
    ) -> BoxFuture<'a, GetContractDelegateResult>;
}

pub(crate) fn get_contract_delegate_url(base_url: &str, addr: &Address) -> String {
    format!(
        "{}/chains/main/blocks/head/context/contracts/{}/delegate",
        base_url,
        addr.to_base58check(),
    )
}
