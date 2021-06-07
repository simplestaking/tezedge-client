use serde::Deserialize;

use crate::api::{
    get_protocol_info_url,
    GetProtocolInfo, GetProtocolInfoResult, ProtocolInfo,
    TransportError, GetProtocolInfoError,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetProtocolInfoError {
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

impl From<std::io::Error> for GetProtocolInfoError {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

#[derive(Deserialize)]
struct ProtocolInfoJson {
    protocol: String,
    next_protocol: String,
}

impl Into<ProtocolInfo> for ProtocolInfoJson {
    fn into(self) -> ProtocolInfo {
        let mut info = ProtocolInfo::default();
        info.protocol_hash = self.protocol;
        info.next_protocol_hash = self.next_protocol;
        info
    }
}

impl GetProtocolInfo for HttpApi {
    fn get_protocol_info(&self) -> GetProtocolInfoResult {
        Ok(self.client.get(&get_protocol_info_url(&self.base_url))
            .call()?
            .into_json::<ProtocolInfoJson>()?
            .into())
    }
}

