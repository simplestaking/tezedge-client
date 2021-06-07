use serde::Deserialize;

use types::ImplicitAddress;
use crate::api::{
    get_contract_counter_url,
    GetContractCounterAsync, GetContractCounterResult,
    TransportError, GetContractCounterError, GetContractCounterErrorKind,
};
use crate::BoxFuture;
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetContractCounterErrorKind {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            Self::Unknown(format!(
                "Http status: ({}) {}",
                status,
                error,
            ))
        } else {
            Self::Transport(TransportError(Box::new(error)))
        }
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ContractCounter {
    #[serde(with = "utils::serde_str")]
    current: u64,
}

impl Into<u64> for ContractCounter {
    fn into(self) -> u64 {
        self.current
    }
}

impl GetContractCounterAsync for HttpApi {
    fn get_contract_counter<'a>(
        &'a self,
        addr: &'a ImplicitAddress,
    ) -> BoxFuture<'a, GetContractCounterResult>
    {
        Box::pin(async move {
            Ok(self.client.get(&get_contract_counter_url(&self.base_url, addr))
                .send().await
                .map_err(|err| GetContractCounterError::new(addr, err))?
                .json::<ContractCounter>().await
                .map_err(|err| GetContractCounterError::new(addr, err))?
                .into())
        })
    }
}
