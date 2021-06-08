pub mod tests_common;
use tests_common::{build_http_apis, account_1};

use rpc_api::api::*;

macro_rules! test_get {
    ($fn_name:ident) => {
        paste::item! {
            #[tokio::test]
            async fn [< test_ $fn_name >] () {
                let (sync_api, async_api) = build_http_apis();

                assert_eq!(
                    tests_common::HttpApi::$fn_name(&sync_api).unwrap(),
                    tests_common::HttpApiAsync::$fn_name(&async_api).await.unwrap(),
                );
            }
        }
    };
}

test_get!(get_chain_id);
// test_get!(get_constants);
test_get!(get_head_block_hash);
test_get!(get_protocol_info);
test_get!(get_version_info);


#[tokio::test]
async fn test_get_manager_public_key() {
    let address = account_1().address.into();

    let (sync_api, async_api) = build_http_apis();

    assert_eq!(
        sync_api.get_manager_public_key(&address).unwrap(),
        async_api.get_manager_public_key(&address).await.unwrap(),
    );
}

#[tokio::test]
async fn test_get_contract_counter() {
    let address = account_1().address;

    let (sync_api, async_api) = build_http_apis();

    assert_eq!(
        sync_api.get_contract_counter(&address).unwrap(),
        async_api.get_contract_counter(&address).await.unwrap(),
    );
}
