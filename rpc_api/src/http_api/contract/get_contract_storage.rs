use types::OriginatedAddress;
use crate::api::{
    get_contract_storage_url,
    GetContractStorage, GetContractStorageResult,
    TransportError, GetContractStorageError, GetContractStorageErrorKind,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetContractStorageErrorKind {
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

impl From<std::io::Error> for GetContractStorageErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetContractStorage for HttpApi {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> GetContractStorageResult
    {
        Ok(self.client.get(&get_contract_storage_url(&self.base_url, addr))
           .call()
           .map_err(|err| GetContractStorageError::new(addr, err))?
           .into_json()
           .map_err(|err| GetContractStorageError::new(addr, err))?)
    }
}
