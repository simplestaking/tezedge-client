use serde::Deserialize;

use types::ImplicitAddress;
use crate::api::{
    get_contract_counter_url,
    GetContractCounter, GetContractCounterResult,
    TransportError, GetContractCounterError, GetContractCounterErrorKind,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetContractCounterErrorKind {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Transport(error) => {
                Self::Transport(TransportError(Box::new(error)))
            }
            ureq::Error::Status(code, resp) => {
                let status_text = resp.status_text().to_string();
                Self::Unknown(format!(
                    "Http status: ({}, {}){}",
                    code,
                    status_text,
                    match resp.into_string() {
                        Ok(s) => format!(", message: {}", s),
                        Err(_) => "".to_string(),
                    },
                ))
            }
        }
    }
}

impl From<std::io::Error> for GetContractCounterErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
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

impl GetContractCounter for HttpApi {
    fn get_contract_counter(&self, addr: &ImplicitAddress) -> GetContractCounterResult {
        Ok(self.client.get(&get_contract_counter_url(&self.base_url, addr))
           .call()
           .map_err(|err| GetContractCounterError::new(addr, err))?
           .into_json::<ContractCounter>()
           .map_err(|err| GetContractCounterError::new(addr, err))?
           .into())
    }
}
