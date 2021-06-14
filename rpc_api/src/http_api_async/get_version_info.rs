use crate::BoxFuture;
use crate::api::{
    get_version_info_url,
    TransportError,
    GetVersionInfoAsync, GetVersionInfoResult, GetVersionInfoError,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetVersionInfoError {
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

impl GetVersionInfoAsync for HttpApi {
    fn get_version_info(&self) -> BoxFuture<'static, GetVersionInfoResult> {
        let req = self.client.get(&get_version_info_url(&self.base_url));
        Box::pin(async move {
            Ok(req
               .send().await?
               .json().await?)
        })
    }
}
