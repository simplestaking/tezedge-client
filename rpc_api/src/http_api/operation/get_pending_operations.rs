use serde::{Serialize, Deserialize};

use crate::api::{
    get_pending_operations_url,
    TransportError, GetPendingOperations, GetPendingOperationsResult,
    GetPendingOperationsError, PendingOperations, PendingOperation,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetPendingOperationsError {
    fn from(error: ureq::Error) -> Self {
        match error {
            ureq::Error::Transport(error) => {
                Self::Transport(TransportError(Box::new(error)))
            }
            ureq::Error::Status(code, resp) => {
                let status_text = resp.status_text().to_string();
                Self::Unknown(format!(
                    "Http status: ({}, {}){}",
                    code,
                    status_text,
                    match resp.into_string() {
                        Ok(s) => format!(", message: {}", s),
                        Err(_) => "".to_string(),
                    },
                ))
            }
        }
    }
}

impl From<std::io::Error> for GetPendingOperationsError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
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

impl GetPendingOperations for HttpApi {
    fn get_pending_operations(&self) -> GetPendingOperationsResult {
        Ok(self.client.get(&get_pending_operations_url(&self.base_url))
           .call()?
           .into_json::<PendingOperationsJson>()?
           .into())
    }
}
