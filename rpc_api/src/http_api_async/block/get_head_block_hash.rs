use crate::BoxFuture;
use crate::api::{
    get_head_block_hash_url,
    GetHeadBlockHashAsync, GetHeadBlockHashResult,
    TransportError, GetHeadBlockHashError,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetHeadBlockHashError {
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

impl GetHeadBlockHashAsync for HttpApi {
    fn get_head_block_hash(&self) -> BoxFuture<'static, GetHeadBlockHashResult> {
        let req = self.client.get(&get_head_block_hash_url(&self.base_url));

        Box::pin(async move {
            Ok(req
                .send().await?
                .json().await?)
        })
    }
}
