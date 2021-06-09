use lib::{NewOperationGroup, NewTransactionParameters, ManagerParameter, Address};
use lib::api::{RunOperation, RunOperationError, RunOperationContents};

#[derive(PartialEq, Debug, Clone)]
pub struct OperationGroupGasConsumption {
    pub reveal: Option<u64>,
    pub transaction: Option<u64>,
    pub delegation: Option<u64>,
}

impl OperationGroupGasConsumption {
    pub fn total(&self) -> u64 {
        self.reveal.unwrap_or(0)
            + self.transaction.unwrap_or(0)
            + self.delegation.unwrap_or(0)
    }
}

fn find_consumed_gas_for_kind(
    kind: &str,
    run_op_contents: &RunOperationContents,
) -> Option<u64> {
    run_op_contents.iter()
        .find(|op| op.kind.as_str() == kind)
        // Add 100 for safety
        .map(|op| op.consumed_gas + 100)
}


pub fn estimate_gas_consumption<A>(
    op: &NewOperationGroup,
    api: &A,
) -> Result<OperationGroupGasConsumption, RunOperationError>
    where A: RunOperation + ?Sized,
{
    let op_results = api.run_operation(op)?;
    // additional gas required when sending/delegating from Smart Contract (KT1).
    let tx_additional_gas = op.transaction.as_ref()
        .map(|op| {
            use NewTransactionParameters::*;
            use ManagerParameter::*;
            match op.parameters.as_ref() {
                Some(Manager(Transfer { to, .. })) => {
                    match to {
                        Address::Implicit(_) => 1427,
                        Address::Originated(_) => 2863,
                    }
                }
                Some(Manager(SetDelegate(_))) => 1000,
                Some(Manager(CancelDelegate)) => 1000,
                Some(Custom { .. }) => 0, // Unknown gas consumption
                None => 0,
            }
        })
        .unwrap_or(0);

    Ok(OperationGroupGasConsumption {
        reveal: find_consumed_gas_for_kind("reveal", &op_results),
        transaction: find_consumed_gas_for_kind("transaction", &op_results)
            .map(|gas| gas + tx_additional_gas),
        delegation: find_consumed_gas_for_kind("delegation", &op_results),
    })
}
