use types::{Address, PublicKey};
use crypto::ToBase58Check;
use crate::api::{
    get_manager_key_url,
    GetManagerPublicKey, GetManagerPublicKeyResult,
    TransportError, GetManagerPublicKeyError, GetManagerPublicKeyErrorKind,
};
use crate::http_api::HttpApi;

impl From<ureq::Error> for GetManagerPublicKeyErrorKind {
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

impl From<std::io::Error> for GetManagerPublicKeyErrorKind {
    fn from(error: std::io::Error) -> Self {
        Self::Transport(TransportError(Box::new(error)))
    }
}

impl GetManagerPublicKey for HttpApi {
    fn get_manager_public_key(&self, addr: &Address) -> GetManagerPublicKeyResult {
        Ok(self.client.get(&get_manager_key_url(&self.base_url, addr))
           .call()
           .map_err(|err| GetManagerPublicKeyError::new(addr, err))?
           .into_json::<Option<String>>()
           .map_err(|err| GetManagerPublicKeyError::new(addr, err))?
           .map(|key| PublicKey::from_base58check(&key))
           .transpose()
           .map_err(|err| GetManagerPublicKeyError::new(addr, err))?)
    }
}

