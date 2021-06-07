use crate::api::{
    get_version_info_url,
    GetVersionInfo, GetVersionInfoResult,
    TransportError, GetVersionInfoError,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetVersionInfoError {
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

impl From<std::io::Error> for GetVersionInfoError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetVersionInfo for HttpApi {
    fn get_version_info(&self) -> GetVersionInfoResult {
        Ok(self.client.get(&get_version_info_url(&self.base_url))
            .call()?
            .into_json()?)
    }
}
