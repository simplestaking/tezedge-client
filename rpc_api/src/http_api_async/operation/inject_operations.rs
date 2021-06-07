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
    fn inject_operations<'a>(
        &'a self,
        operation_with_signature: &'a str,
    ) -> BoxFuture<'a, InjectOperationsResult>
    {
        Box::pin(async move {
            let operation_with_signature_json =
                SerdeValue::String(operation_with_signature.to_owned());

            Ok(self.client.post(&inject_operations_url(&self.base_url))
               .json(&operation_with_signature_json)
               .send().await?
               .json().await?)
        })
    }
}
