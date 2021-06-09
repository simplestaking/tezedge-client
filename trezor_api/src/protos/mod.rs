pub mod messages_common;
pub use messages_common::*;

pub mod messages;
pub use messages::*;

pub mod messages_tezos;
pub use messages_tezos::*;

pub mod messages_management;
pub use messages_management::*;

use types::{
    Forge, Address, ImplicitAddress, OriginatedAddress, OriginatedAddressWithManager,
    NewOperationGroup, NewRevealOperation, NewTransactionOperation, NewDelegationOperation,
    NewOriginationOperation, NewTransactionParameters, ManagerParameter,
};

impl Into<TezosSignTx_TezosContractID> for Address {
    fn into(self) -> TezosSignTx_TezosContractID {
        match self {
            Self::Implicit(addr) => addr.into(),
            Self::Originated(addr) => addr.into(),
        }
    }
}

impl Into<TezosSignTx_TezosContractID> for ImplicitAddress {
    fn into(self) -> TezosSignTx_TezosContractID {
        let mut contract_id = TezosSignTx_TezosContractID::new();
        contract_id.set_hash(self.forge().take());
        contract_id.set_tag(TezosSignTx_TezosContractID_TezosContractType::Implicit);

        contract_id
    }
}

impl Into<TezosSignTx_TezosContractID> for OriginatedAddress {
    fn into(self) -> TezosSignTx_TezosContractID {
        let mut contract_id = TezosSignTx_TezosContractID::new();
        contract_id.set_hash(self.forge().take());
        contract_id.set_tag(TezosSignTx_TezosContractID_TezosContractType::Originated);

        contract_id
    }
}

impl Into<TezosSignTx_TezosContractID> for OriginatedAddressWithManager {
    fn into(self) -> TezosSignTx_TezosContractID {
        self.address.into()
    }
}

// Operations
impl Into<TezosSignTx> for NewOperationGroup {
    /// Creates `TezosSignTx`, protobuf type for Trezor.
    ///
    /// **Warning**: make sure to set `address_n` field after, since
    /// it's required and not added here.
    fn into(self) -> TezosSignTx {
        let mut new_tx = TezosSignTx::new();
        new_tx.set_branch(self.branch.as_ref().to_vec());

        if let Some(op) = self.reveal {
            new_tx.set_reveal(op.into());
        }

        if let Some(op) = self.transaction {
            new_tx.set_transaction(op.into());
        }

        if let Some(op) = self.delegation {
            new_tx.set_delegation(op.into());
        }

        if let Some(op) = self.origination {
            new_tx.set_origination(op.into());
        }

        new_tx
    }
}

impl Into<TezosSignTx_TezosRevealOp> for NewRevealOperation {
    /// Creates `TezosSignTx_TezosRevealOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosRevealOp {
        let mut new_op = TezosSignTx_TezosRevealOp::new();

        new_op.set_source(self.source.forge().take());
        new_op.set_public_key(self.public_key.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}

impl Into<TezosSignTx_TezosTransactionOp> for NewTransactionOperation {
    /// Creates `TezosSignTx_TezosTransactionOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosTransactionOp {
        let mut new_tx = TezosSignTx_TezosTransactionOp::new();

        new_tx.set_source(self.source.forge().take());
        new_tx.set_destination(self.destination.into());
        new_tx.set_counter(self.counter);
        new_tx.set_fee(self.fee);
        new_tx.set_amount(self.amount);
        new_tx.set_gas_limit(self.gas_limit);
        new_tx.set_storage_limit(self.storage_limit);

        match self.parameters {
            Some(NewTransactionParameters::Manager(params)) => {
                new_tx.set_parameters_manager(params.into());
            }
            Some(params @ NewTransactionParameters::Custom { .. }) => {
                new_tx.set_parameters(params.forge().take());
            }
            None => {}
        };

        new_tx
    }
}

impl Into<TezosSignTx_TezosDelegationOp> for NewDelegationOperation {
    /// Creates `TezosSignTx_TezosDelegationOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosDelegationOp {
        let mut new_op = TezosSignTx_TezosDelegationOp::new();

        if let Some(delegate_to) = self.delegate_to {
            new_op.set_delegate(delegate_to.forge().take());
        }

        new_op.set_source(self.source.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);

        new_op
    }
}

impl Into<TezosSignTx_TezosOriginationOp> for NewOriginationOperation {
    /// Creates `TezosSignTx_TezosOriginationOp`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosOriginationOp {
        let mut new_op = TezosSignTx_TezosOriginationOp::new();

        new_op.set_source(self.source.forge().take());
        new_op.set_counter(self.counter);
        new_op.set_fee(self.fee);
        new_op.set_balance(self.balance);
        new_op.set_gas_limit(self.gas_limit);
        new_op.set_storage_limit(self.storage_limit);
        new_op.set_script(self.script.forge().take());

        new_op
    }
}

impl Into<TezosSignTx_TezosTransactionOp_TezosParametersManager> for ManagerParameter {
    /// Creates `TezosSignTx_TezosTransactionOp_TezosParametersManager`, protobuf type for Trezor.
    fn into(self) -> TezosSignTx_TezosTransactionOp_TezosParametersManager {
        let mut params = TezosSignTx_TezosTransactionOp_TezosParametersManager::new();

        match self {
            Self::SetDelegate(addr) => {
                params.set_set_delegate(addr.forge().take());
            }
            Self::CancelDelegate => {
                params.set_cancel_delegate(true);
            }
            Self::Transfer { to, amount } => {
                let mut transfer = TezosSignTx_TezosTransactionOp_TezosParametersManager_TezosManagerTransfer::new();
                transfer.set_destination(to.into());
                transfer.set_amount(amount);

                params.set_transfer(transfer);
            }
        }
        params
    }
}
