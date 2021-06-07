use serde::{Serialize, Deserialize};

use crate::BoxFuture;
use crate::api::{
    get_pending_operations_url,
    TransportError, GetPendingOperationsAsync, GetPendingOperationsResult,
    GetPendingOperationsError, PendingOperations, PendingOperation,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for GetPendingOperationsError {
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

#[derive(Serialize, Deserialize, Clone)]
struct PendingOperationsJson {
    applied: Vec<PendingOperation>,
    refused: Vec<(String, serde_json::Value)>,
}

impl From<PendingOperationsJson> for PendingOperations {
    fn from(op: PendingOperationsJson) -> Self {
        Self {
            applied: op.applied,
            refused: op.refused.into_iter()
                .map(|(hash, _)| PendingOperation { hash })
                .collect(),
        }
    }
}

impl GetPendingOperationsAsync for HttpApi {
    fn get_pending_operations<'a>(&'a self) -> BoxFuture<'a, GetPendingOperationsResult> {
        Box::pin(async move {
            Ok(self.client.get(&get_pending_operations_url(&self.base_url))
               .send().await?
               .json::<PendingOperationsJson>().await?
               .into())
        })
    }
}
