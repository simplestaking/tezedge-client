use types::{NewOperationGroup, NewOperationWithKind};
use crate::BoxFuture;
use crate::api::{
    run_operation_url,
    GetChainIDAsync, RunOperationAsync, RunOperationResult,
    TransportError, RunOperationError, RunOperationJson,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for RunOperationError {
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

impl RunOperationAsync for HttpApi {
    fn run_operation(
        &self,
        operation_group: &NewOperationGroup,
    ) -> BoxFuture<'static, RunOperationResult>
    {
        let get_chain_id_fut = self.get_chain_id();
        let req = self.client.post(&run_operation_url(&self.base_url));
        let operation_group = operation_group.clone();
        Box::pin(async move {
            Ok(req
                .json(&serde_json::json!({
                    "chain_id": get_chain_id_fut.await?,
                    "operation": {
                        "branch": &operation_group.branch,
                        // this is necessary to be valid signature for this call
                        // to work, but doesn't need to match the actual operation signature.
                        "signature": "edsigthZLBZKMBUCwHpMCXHkGtBSzwh7wdUxqs7C1LRMk64xpcVU8tyBDnuFuf9CLkdL3urGem1zkHXFV9JbBBabi6k8QnhW4RG",
                        "contents": operation_group.to_operations_vec()
                            .into_iter()
                            .map(|op| NewOperationWithKind::from(op))
                            .collect::<Vec<_>>(),
                    },

                }))
                .send().await?
                .json::<RunOperationJson>().await?
                .into())
        })
    }
}
