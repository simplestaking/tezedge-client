use types::{NewOperationGroup, NewOperationWithKind};
use crate::api::{
    preapply_operations_url,
    PreapplyOperations, PreapplyOperationsResult,
    TransportError, PreapplyOperationsError,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for PreapplyOperationsError {
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

impl From<std::io::Error> for PreapplyOperationsError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl PreapplyOperations for HttpApi {
    fn preapply_operations(
        &self,
        operation_group: &NewOperationGroup,
        signature: &str,
    ) -> PreapplyOperationsResult
    {
        Ok(self.client.post(&preapply_operations_url(&self.base_url))
           .send_json(ureq::json!([{
               "protocol": &operation_group.next_protocol_hash,
               "branch": &operation_group.branch,
               "signature": signature,
               "contents": operation_group.to_operations_vec()
                   .into_iter()
                   .map(|op| NewOperationWithKind::from(op))
                   .collect::<Vec<_>>(),
           }]))?
           .into_json()?)
    }
}
