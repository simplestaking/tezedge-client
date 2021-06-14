use serde_json::Value as SerdeValue;

use crate::BoxFuture;
use crate::api::{
    inject_operations_url,
    InjectOperationsAsync, InjectOperationsResult,
    TransportError, InjectOperationsError,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for InjectOperationsError {
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

impl InjectOperationsAsync for HttpApi {
    fn inject_operations(
        &self,
        operation_with_signature: &str,
    ) -> BoxFuture<'static, InjectOperationsResult>
    {
        let req = self.client.post(&inject_operations_url(&self.base_url));
        let operation_with_signature_json =
            SerdeValue::String(operation_with_signature.to_owned());

        Box::pin(async move {
            Ok(req
               .json(&operation_with_signature_json)
               .send().await?
               .json().await?)
        })
    }
}
