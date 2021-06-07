use crate::api::{get_chain_id_url, TransportError, GetChainID, GetChainIDResult, GetChainIDError};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetChainIDError {
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

impl From<std::io::Error> for GetChainIDError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetChainID for HttpApi {
    fn get_chain_id(&self) -> GetChainIDResult {
        Ok(self.client.get(&get_chain_id_url(&self.base_url))
            .call()?
            .into_json()?)
    }
}
