use serde::Deserialize;

use crate::BoxFuture;
use crate::api::{
    get_protocol_info_url,
    GetProtocolInfoAsync, GetProtocolInfoResult, ProtocolInfo,
    TransportError, GetProtocolInfoError,
};
use crate::http_api_async::HttpApi;


impl From<reqwest::Error> for GetProtocolInfoError {
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
struct ProtocolInfoJson {
    protocol: String,
    next_protocol: String,
}

impl Into<ProtocolInfo> for ProtocolInfoJson {
    fn into(self) -> ProtocolInfo {
        let mut info = ProtocolInfo::default();
        info.protocol_hash = self.protocol;
        info.next_protocol_hash = self.next_protocol;
        info
    }
}

impl GetProtocolInfoAsync for HttpApi {
    fn get_protocol_info(&self) -> BoxFuture<'static, GetProtocolInfoResult> {
        let req = self.client.get(&get_protocol_info_url(&self.base_url));
        Box::pin(async move {
            Ok(req
               .send().await?
               .json::<ProtocolInfoJson>().await?
               .into())
        })
    }
}
