use types::OriginatedAddress;
use crate::BoxFuture;
use crate::api::{
    get_contract_storage_url,
    GetContractStorageAsync, GetContractStorageResult,
    TransportError, GetContractStorageError, GetContractStorageErrorKind,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetContractStorageErrorKind {
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

impl GetContractStorageAsync for HttpApi {
    fn get_contract_storage(
        &self,
        addr: &OriginatedAddress,
    ) -> BoxFuture<'static, GetContractStorageResult>
    {
        let req = self.client.get(&get_contract_storage_url(&self.base_url, addr));
        let addr = addr.clone();
        Box::pin(async move {
            req
                .send().await
                .map_err(|err| GetContractStorageError::new(&addr, err))?
                .json().await
                .map_err(|err| GetContractStorageError::new(&addr, err))
        })
    }
}
