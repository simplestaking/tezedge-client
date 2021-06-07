use crate::BoxFuture;
use crate::api::{
    get_chain_id_url,
    GetChainIDAsync, GetChainIDResult,
    TransportError, GetChainIDError,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetChainIDError {
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

impl GetChainIDAsync for HttpApi {
    fn get_chain_id<'a>(&'a self) -> BoxFuture<'a, GetChainIDResult> {
        Box::pin(async move {
            Ok(self.client.get(&get_chain_id_url(&self.base_url))
               .send().await?
               .json().await?)
        })
    }
}
