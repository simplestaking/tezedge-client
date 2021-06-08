use rpc_api::*;
use types::{ImplicitAddress, PrivateKey, PublicKey};

/// Sync Http Api
pub type HttpApi = http_api::HttpApi;

/// Async Http Api
pub type HttpApiAsync = http_api_async::HttpApi;

pub struct Account {
    pub address: ImplicitAddress,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
}

#[inline]
pub const fn base_url() -> &'static str {
    "https://api.tez.ie/rpc/edonet"
}

pub fn build_http_apis() -> (HttpApi, HttpApiAsync) {
    (
        HttpApi::new(base_url()),
        HttpApiAsync::new(base_url()),
    )
}

pub fn account_1() -> Account {
    Account {
        address: ImplicitAddress::from_base58check("tz1cY73TfXg3CYxGhQwJviYG8gN7WYn9NM2t").unwrap(),
        private_key: PrivateKey::from_base58check("edsk324owL57dydrDEBHXYSnNq7zAAoVzc61sugi2rD4qcVrMzogG9").unwrap(),
        public_key: PublicKey::from_base58check("edpku7rPeaYLJpyfwkmGJs6cyxUQQEZuKgGyMvLxWfLiAckGT5WrRZ").unwrap(),
    }
}
