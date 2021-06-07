use types::{NewOperationGroup, NewOperationWithKind};
use crate::BoxFuture;
use crate::api::{
    preapply_operations_url,
    PreapplyOperationsAsync, PreapplyOperationsResult,
    TransportError, PreapplyOperationsError,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for PreapplyOperationsError {
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

impl PreapplyOperationsAsync for HttpApi {
    fn preapply_operations<'a>(
        &'a self,
        operation_group: &'a NewOperationGroup,
        signature: &'a str,
    ) -> BoxFuture<'a, PreapplyOperationsResult>
    {
        Box::pin(async move {
            Ok(self.client.post(&preapply_operations_url(&self.base_url))
                .json(&serde_json::json!([{
                    "protocol": &operation_group.next_protocol_hash,
                    "branch": &operation_group.branch,
                    "signature": signature,
                    "contents": operation_group.to_operations_vec()
                        .into_iter()
                        .map(|op| NewOperationWithKind::from(op))
                        .collect::<Vec<_>>(),
                }]))
                .send().await?
                .json().await?)
        })
    }
}
