use serde::{Serialize, Deserialize};

use crate::micheline::{Micheline, MichelineEntrypoint};

/// Parameters for Smart Contract.
///
/// In order to interact with the Smart Contract, transaction must be
/// created with the destination set to smart contract's address.
///
/// Note: Smart contract's address starts with **KT1**.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum NewTransactionParameters {
    Manager(super::ManagerParameter),
    #[serde(skip)]
    Custom {
        entrypoint: MichelineEntrypoint,
        data: Micheline,
    }
}
