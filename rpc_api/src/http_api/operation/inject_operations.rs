use serde_json::Value as SerdeValue;

use crate::api::{
    inject_operations_url,
    InjectOperations, InjectOperationsResult,
    TransportError, InjectOperationsError,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for InjectOperationsError {
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

impl From<std::io::Error> for InjectOperationsError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl InjectOperations for HttpApi {
    fn inject_operations(
        &self,
        operation_with_signature: &str,
    ) -> InjectOperationsResult
    {
        let operation_with_signature_json =
            SerdeValue::String(operation_with_signature.to_owned());

        Ok(self.client.post(&inject_operations_url(&self.base_url))
           .send_json(operation_with_signature_json)?
           .into_json()?)
    }
}
