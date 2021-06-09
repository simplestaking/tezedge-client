use serde::{Serialize, Deserialize};

use utils::estimate_operation_fee;
use crate::{Forge, ImplicitAddress, ImplicitOrOriginatedWithManager, OriginatedAddressWithManager};
use super::{NewOperation, NewTransactionOperation, NewTransactionParameters, ManagerParameter};

#[derive(Debug, Clone)]
pub struct NewDelegationOperationBuilder {
    pub source: ImplicitOrOriginatedWithManager,
    /// Optional.
    ///
    /// If set to `None`, account will stop delegating to anyone.
    pub delegate_to: Option<ImplicitAddress>,
    pub fee: u64,
    pub counter: u64,
    pub gas_limit: u64,
    pub storage_limit: u64,
}

impl NewDelegationOperationBuilder {
    /// Build Delegation Operation.
    ///
    /// - Will return [NewOperation::Delegation]: when delegating from
    ///   an implicit account.
    /// - Will return [NewOperation::Transaction]: when delegating from
    ///   an originate account. So when delegating funds from smart contract.
    ///
    ///   Instead of delegation operation, it will create a transaction,
    ///   which will call function inside given smart contract, which will
    ///   activate delegation for that smart contract.
    pub fn build(self) -> NewOperation {
        use ImplicitOrOriginatedWithManager::*;
        match self.source {
            Implicit(source) => {
                NewOperation::Delegation(NewDelegationOperation {
                    source,
                    delegate_to: self.delegate_to,
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                })
            }
            OriginatedWithManager(OriginatedAddressWithManager {
                address,
                manager,
            }) => {
                let parameters = match self.delegate_to {
                    Some(delegate) => ManagerParameter::SetDelegate(delegate),
                    None => ManagerParameter::CancelDelegate,
                };
                NewOperation::Transaction(NewTransactionOperation {
                    source: manager,
                    destination: address.into(),
                    fee: self.fee,
                    counter: self.counter,
                    gas_limit: self.gas_limit,
                    storage_limit: self.storage_limit,
                    amount: 0,
                    parameters: Some(NewTransactionParameters::Manager(parameters)),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewDelegationOperation {
    pub source: ImplicitAddress,
    #[serde(rename = "delegate", skip_serializing_if = "Option::is_none")]
    pub delegate_to: Option<ImplicitAddress>,
    #[serde(with = "utils::serde_amount")]
    pub fee: u64,
    #[serde(with = "utils::serde_str")]
    pub counter: u64,
    #[serde(with = "utils::serde_str")]
    pub gas_limit: u64,
    #[serde(with = "utils::serde_str")]
    pub storage_limit: u64,
}

impl NewDelegationOperation {
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
