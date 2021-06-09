use serde::{Serialize, Deserialize};

use crate::{
    Forge, NewTransactionParameters, ManagerParameter, Address, ImplicitAddress,
    ImplicitOrOriginatedWithManager, OriginatedAddressWithManager,
};
use utils::estimate_operation_fee;

#[derive(Debug, Clone)]
pub struct NewTransactionOperationBuilder {
    pub source: ImplicitOrOriginatedWithManager,
    pub destination: Address,
    pub amount: u64,
    pub fee: u64,
    pub counter: u64,
    pub gas_limit: u64,
    pub storage_limit: u64,
}

impl NewTransactionOperationBuilder {
    pub fn build(self) -> NewTransactionOperation {
        use ImplicitOrOriginatedWithManager::*;
        match self.source {
            Implicit(source) => {
                NewTransactionOperation {
                    source,
                    destination: self.destination,
                    amount: self.amount,
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                    parameters: None,
                }
            }
            OriginatedWithManager(OriginatedAddressWithManager { address, manager }) => {
                NewTransactionOperation {
                    source: manager,
                    destination: address.into(),
                    amount: 0,
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                    parameters: Some(NewTransactionParameters::Manager(
                        ManagerParameter::Transfer {
                            to: self.destination,
                            amount: self.amount,
                        }
                    )),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTransactionOperation {
    pub source: ImplicitAddress,
    pub destination: Address,
    #[serde(with = "utils::serde_amount")]
    pub amount: u64,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<NewTransactionParameters>,
}

impl NewTransactionOperation {
    /// Estimate byte size of the operation.
    ///
    /// Forges the operation and counts bytes.
    pub fn estimate_bytes(&self) -> u64 {
        self.forge().take().len() as u64
    }

    /// Estimate minimal fee.
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
