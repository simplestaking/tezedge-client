use types::Address;
use crate::api::{
    get_contract_delegate_url,
    GetContractDelegateAsync, GetContractDelegateResult,
    TransportError, GetContractDelegateError, GetContractDelegateErrorKind,
};
use crate::BoxFuture;
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetContractDelegateErrorKind {
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

impl GetContractDelegateAsync for HttpApi {
    fn get_contract_delegate<'a>(
        &'a self,
        addr: &'a Address,
    ) -> BoxFuture<'a, GetContractDelegateResult>
    {
        Box::pin(async move {
            let url = get_contract_delegate_url(&self.base_url, addr);
            let result = self.client.get(&url).send().await;

            let status = match &result {
                Ok(resp) => Some(resp.status()),
                Err(err) => err.status(),
            };

            if status.map(|x| x.as_u16()).filter(|&x| x == 404).is_some() {
                Ok(None)
            } else {
                result
                    .map_err(|err| GetContractDelegateError::new(addr, err))?
                    .json().await
                    .map_err(|err| GetContractDelegateError::new(addr, err))
            }
        })
    }
}
