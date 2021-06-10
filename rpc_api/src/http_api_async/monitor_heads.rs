use futures_util::StreamExt;

use crate::BoxFuture;
use crate::api::{
    monitor_heads_url,
    TransportError, MonitorHeadsError,
    MonitorHeadsAsync, StartMonitorHeadsResult, BlockHead,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for MonitorHeadsError {
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

impl MonitorHeadsAsync for HttpApi {
    fn monitor_heads<'a>(&'a self) -> BoxFuture<'a, StartMonitorHeadsResult<'a>> {
        Box::pin(async move {
            Ok(self.client.get(&monitor_heads_url(&self.base_url, "main"))
                .send().await?
                .bytes_stream()
                .map(|result| {
                    let bytes = result?;
                    Ok(serde_json::from_slice::<BlockHead>(&bytes)?)
                })
                .boxed())
        })
    }
}
