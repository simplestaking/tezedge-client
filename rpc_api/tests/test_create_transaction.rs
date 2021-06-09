use std::time::Duration;

pub mod tests_common;
use tests_common::{build_http_apis, account_1};

use rpc_api::api::*;
use explorer_api::TzStats;
use signer::LocalSigner;
use types::{Address, Forge, Network, NewOperationGroup, NewTransactionOperationBuilder};
use utils::parse_float_amount;

#[tokio::test]
async fn test_create_transaction() {
    let (_, async_api) = build_http_apis();
    let account = account_1();

    let signer = LocalSigner::new(account.public_key.clone(), account.private_key.clone());

    let tx = NewTransactionOperationBuilder {
        source: account.address.clone().into(),
        destination: Address::from_base58check("tz1Yx6DUHcxDz4ye5gXePAC3p2K36wLKKCJa").unwrap(),
        amount: parse_float_amount("0.01").unwrap(),
        fee: parse_float_amount("0.01").unwrap(),
        counter: async_api.get_contract_counter(&account.address).await.unwrap() + 1,
        gas_limit: 20000,
        storage_limit: 20000,
    }.build();

    let operation_group = NewOperationGroup::new(
        async_api.get_head_block_hash().await.unwrap(),
        async_api.get_protocol_info().await.unwrap().next_protocol_hash,
    ).with_transaction(tx);

    let forged = operation_group.forge();
    let signed = signer.sign_forged_operation_bytes(forged.as_ref());
    let op_hash = signed.operation_hash;

    async_api.inject_operations(&signed.operation_with_signature).await.unwrap();
    assert_eq!(
        async_api.get_pending_operation_status(&op_hash).await.unwrap(),
        PendingOperationStatus::Applied,
    );

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let status = async_api.get_pending_operation_status(&op_hash).await.unwrap();

        if let PendingOperationStatus::Finished = status {
            break;
        }
    }

    let version = async_api.get_version_info().await.unwrap();
    let tzstats = TzStats::new(version.get_network()).unwrap();

    let ops = tzstats.get_operation(&op_hash).unwrap();
    assert_eq!(ops.len(), 1);

    println!("transaction successfuly created: {}", op_hash);
}
