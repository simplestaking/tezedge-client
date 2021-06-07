use types::Address;
use crypto::ToBase58Check;
use crate::api::{
    get_contract_delegate_url,
    GetContractDelegate, GetContractDelegateResult,
    TransportError, GetContractDelegateError, GetContractDelegateErrorKind,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetContractDelegateErrorKind {
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

impl From<std::io::Error> for GetContractDelegateErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetContractDelegate for HttpApi {
    fn get_contract_delegate(&self, addr: &Address) -> GetContractDelegateResult {
        Ok(self.client.get(&get_contract_delegate_url(&self.base_url, addr))
           .call()
           .map(|resp| resp.into_json())
           .or_else(|err| {
               match &err {
                   // will return 404 status if `addr` hasn't an active delegation.
                   ureq::Error::Status(404, _) => Ok(Ok(None)),
                   _ => Err(GetContractDelegateError::new(addr, err))
               }
           })?
           .map_err(|err| GetContractDelegateError::new(addr, err))?)
    }
}
