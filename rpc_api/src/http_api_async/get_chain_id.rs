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
    fn get_chain_id(&self) -> BoxFuture<'static, GetChainIDResult> {
        let req = self.client.get(&get_chain_id_url(&self.base_url));
        Box::pin(async move {
            Ok(req
               .send().await?
               .json().await?)
        })
    }
}
