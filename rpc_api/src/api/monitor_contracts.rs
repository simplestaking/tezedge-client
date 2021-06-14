use std::fmt::{self, Display};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashSet;
use futures_util::{FutureExt, StreamExt};

use types::{Address, ImplicitAddress, OriginatedAddress, FromPrefixedBase58CheckError};
use crate::{BoxFuture, BoxStream};
use crate::api::{
    MonitorOperationsAsync, MonitoredOperation, BlockOperationContent,
    TransportError, MonitorOperationsError,
};

#[derive(thiserror::Error, Debug)]
pub enum MonitorContractsError {
    ParseChunk(#[from] serde_json::Error),
    Base58Decode(#[from] FromPrefixedBase58CheckError),
    Transport(#[from] TransportError),
    Unknown(String),
}

impl From<MonitorOperationsError> for MonitorContractsError {
    fn from(error: MonitorOperationsError) -> Self {
        use MonitorOperationsError::*;

        match error {
            ParseChunk(err) => Self::ParseChunk(err),
            Base58Decode(err) => Self::Base58Decode(err),
            Transport(err) => Self::Transport(err),
            Unknown(err) => Self::Unknown(err),
        }
    }
}

impl Display for MonitorContractsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "Monitoring contracts failed! Reason: ")?;
        match self {
            Self::ParseChunk(err) => err.fmt(f),
            Self::Base58Decode(err) => err.fmt(f),
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContractOperation {
    /// Contracts that are being listened, that was affected by following operation.
    pub affected_contracts: Vec<Address>,
    pub operation: MonitoredOperation,
}

pub type MonitorContractsResult = Result<ContractOperation, MonitorContractsError>;
pub type StartMonitorContractsResult = Result<BoxStream<'static, MonitorContractsResult>, MonitorOperationsError>;

pub trait MonitorContractsAsync {
    /// Monitor contracts.
    ///
    /// Monitors new blocks and finds operations which reference given contracts.
    fn monitor_contracts<T>(
        &self,
        contracts: T,
    ) -> BoxFuture<'static, StartMonitorContractsResult>
        where T: 'static + Send + Sync,
              for<'a> &'a T: BorrowContracts;
}

pub struct ContractsVecGuard<T>(T);

impl<T> AsRef<[Address]> for ContractsVecGuard<T>
    where T: std::ops::Deref<Target = Vec<Address>>,
{
    fn as_ref(&self) -> &[Address] {
        self.0.as_ref()
    }
}

pub trait ContainsContract {
    fn contains_contract(&self, contract: &Address) -> bool;
}

impl ContainsContract for [Address] {
    fn contains_contract(&self, contract: &Address) -> bool {
        self.contains(contract)
    }
}

impl ContainsContract for Vec<Address> {
    fn contains_contract(&self, contract: &Address) -> bool {
        self.contains(contract)
    }
}

impl<'a> ContainsContract for &'a HashSet<Address> {
    fn contains_contract(&self, contract: &Address) -> bool {
        self.contains(contract)
    }
}

impl<'a, T> ContainsContract for &'a T
    where T: ?Sized + ContainsContract,
{
    fn contains_contract(&self, contract: &Address) -> bool {
        (*self).contains_contract(contract)
    }
}

impl<'a, T> ContainsContract for MutexGuard<'a, T>
    where for<'b> &'b T: ContainsContract,
{
    fn contains_contract(&self, contract: &Address) -> bool {
        (&**self).contains_contract(contract)
    }
}

pub trait BorrowContracts {
    type Output: ContainsContract;

    fn borrow_contracts(self) -> Self::Output;
}

impl<'a> BorrowContracts for &'a [Address] {
    type Output = &'a [Address];

    fn borrow_contracts(self) -> Self::Output {
        self
    }
}

impl<'a, const N: usize> BorrowContracts for &'a [Address; N] {
    type Output = &'a [Address];

    fn borrow_contracts(self) -> Self::Output {
        self
    }
}

impl<'a> BorrowContracts for &'a Mutex<Vec<Address>> {
    type Output = MutexGuard<'a, Vec<Address>>;

    fn borrow_contracts(self) -> Self::Output {
        self.lock().unwrap()
    }
}

impl<'a> BorrowContracts for &'a Arc<Mutex<Vec<Address>>> {
    type Output = MutexGuard<'a, Vec<Address>>;

    fn borrow_contracts(self) -> Self::Output {
        self.lock().unwrap()
    }
}

impl<'a> BorrowContracts for &'a Arc<Mutex<HashSet<Address>>> {
    type Output = MutexGuard<'a, HashSet<Address>>;

    fn borrow_contracts(self) -> Self::Output {
        self.lock().unwrap()
    }
}

impl<U> MonitorContractsAsync for U
    where U: 'static + MonitorOperationsAsync + Clone + Send + Sync,
{
    fn monitor_contracts<T>(
        &self,
        contracts: T,
    ) -> BoxFuture<'static, StartMonitorContractsResult>
        where T: 'static + Send + Sync,
              for<'a> &'a T: BorrowContracts,
    {
        let monitor_operations_fut = self.monitor_operations();
        let client = self.clone();
        Box::pin(async move {
            Ok(monitor_operations_fut.await?
                .map(move |res| {
                    match res {
                        Ok(op) => {
                            // let contracts_container = contracts.clone();
                            let contracts = contracts.borrow_contracts();
                            let mut affected_contracts = vec![];
                            for content in &op.contents {
                                match content {
                                    BlockOperationContent::Reveal(op) => {
                                        let source = op.source.clone().into();
                                        if contracts.contains_contract(&source) {
                                            affected_contracts.push(source);
                                        }
                                    },
                                    BlockOperationContent::Transaction(op) => {
                                        let source = op.source.clone().into();
                                        if contracts.contains_contract(&source) {
                                            affected_contracts.push(source);
                                        }
                                        if contracts.contains_contract(&op.destination) {
                                            affected_contracts.push(op.destination.clone());
                                        }
                                    },
                                    BlockOperationContent::Delegation(op) => {
                                        let source = op.source.clone().into();
                                        if contracts.contains_contract(&source) {
                                            affected_contracts.push(source);
                                        }
                                        if let Some(delegate) = op.delegate.as_ref() {
                                            let delegate = delegate.clone().into();
                                            if contracts.contains_contract(&delegate) {
                                                affected_contracts.push(delegate);
                                            }
                                        }
                                    },
                                    BlockOperationContent::Origination(op) => {
                                        let source = op.source.clone().into();
                                        if contracts.contains_contract(&source) {
                                            affected_contracts.push(source);
                                        }
                                    },
                                    BlockOperationContent::Other => {},
                                }
                            }

                            Ok(ContractOperation {
                                affected_contracts,
                                operation: op,
                            })
                        }
                        Err(err) => Err(err.into()),
                    }
                })
                .filter(|res| {
                    std::future::ready(match res {
                        Ok(x) => x.affected_contracts.len() > 0,
                        Err(_) => true,
                    })
                })
                .boxed())
        })
    }
}
