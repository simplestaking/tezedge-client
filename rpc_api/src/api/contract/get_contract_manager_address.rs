use std::fmt::{self, Display};

use types::{Address, ImplicitAddress, OriginatedAddress, FromPrefixedBase58CheckError};
use crypto::ToBase58Check;
use crate::BoxFuture;
use crate::api::{TransportError, GetContractStorage, GetContractStorageAsync, GetContractStorageError, GetContractStorageErrorKind};

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GetContractManagerAddressErrorKind {
    Transport(#[from] TransportError),
    Base58Decode(#[from] FromPrefixedBase58CheckError),
    #[error("Getting manager address is only supported for contracts originated with \"manager.tz\" script.")]
    UnsupportedContract,
    #[error("Unknown! {0}")]
    Unknown(String),
}

impl From<GetContractStorageErrorKind> for GetContractManagerAddressErrorKind {
    fn from(error: GetContractStorageErrorKind) -> Self {
        use GetContractStorageErrorKind::*;

        match error {
            Transport(err) => Self::Transport(err),
            Unknown(err) => Self::Unknown(err),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct GetContractManagerAddressError {
    pub address: OriginatedAddress,
    pub kind: GetContractManagerAddressErrorKind,
}

impl Display for GetContractManagerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "getting manager's address for contract \"{}\" failed! Reason: {}",
            self.address.to_base58check(),
            self.kind,
        )
    }
}

impl From<GetContractStorageError> for GetContractManagerAddressError {
    fn from(error: GetContractStorageError) -> Self {
        Self {
            address: error.address.into(),
            kind: error.kind.into(),
        }
    }
}

pub type GetContractManagerAddressResult = Result<ImplicitAddress, GetContractManagerAddressError>;

pub trait GetContractManagerAddress {
    /// Get manager address for given contract.
    ///
    /// - If given address is `ImplicitAddress`, manager address = contract address.
    /// - If given address is `OriginatedAddress`, manager address will
    /// be the one, that originated this contract.
    fn get_contract_manager_address(&self, addr: &Address) -> GetContractManagerAddressResult;
}

pub trait GetContractManagerAddressAsync {
    /// Get manager address for given contract.
    ///
    /// - If given address is `ImplicitAddress`, manager address = contract address.
    /// - If given address is `OriginatedAddress`, manager address will
    /// be the one, that originated this contract.
    fn get_contract_manager_address(
        &self,
        addr: &Address,
    ) -> BoxFuture<'static, GetContractManagerAddressResult>;
}

#[inline]
fn build_error<E>(address: &OriginatedAddress, kind: E) -> GetContractManagerAddressError
    where E: Into<GetContractManagerAddressErrorKind>,
{
    GetContractManagerAddressError {
        address: address.clone(),
        kind: kind.into(),
    }
}

fn get_address_from_contract_storage(
    contract_addr: &OriginatedAddress,
    storage: serde_json::Value,
) -> Result<ImplicitAddress, GetContractManagerAddressError>
{
    let manager_str = storage["string"].as_str()
        .ok_or_else(|| build_error(contract_addr, GetContractManagerAddressErrorKind::UnsupportedContract))?;
    ImplicitAddress::from_base58check(manager_str)
        .map_err(|err| build_error(contract_addr, err))
}

impl<T> GetContractManagerAddress for T
    where T: GetContractStorage,
{
    fn get_contract_manager_address(&self, addr: &Address) -> GetContractManagerAddressResult {
        Ok(match addr {
            Address::Implicit(addr) => addr.clone(),
            Address::Originated(addr) => {
                get_address_from_contract_storage(
                    addr,
                    self.get_contract_storage(addr)?,
                )?
            }
        })
    }
}

impl<T> GetContractManagerAddressAsync for T
    where T: GetContractStorageAsync + Send,
{
    fn get_contract_manager_address(
        &self,
        addr: &Address,
    ) -> BoxFuture<'static, GetContractManagerAddressResult>
    {
        use std::future::ready;

        let addr = addr.clone();

        match addr {
            Address::Implicit(addr) => Box::pin(ready(Ok(addr))),
            Address::Originated(addr) => {
                let contract_storage_fut = self.get_contract_storage(&addr);
                Box::pin(async move {
                    get_address_from_contract_storage(
                        &addr,
                        contract_storage_fut.await?,
                    )
                })
            }
        }
    }
}
