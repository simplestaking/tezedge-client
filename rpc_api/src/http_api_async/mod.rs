mod contract;
pub use contract::*;

mod operation;
pub use operation::*;

mod get_chain_id;
pub use get_chain_id::*;

mod get_version_info;
pub use get_version_info::*;

mod get_protocol_info;
pub use get_protocol_info::*;

mod get_head_block_hash;
pub use get_head_block_hash::*;

mod get_manager_public_key;
pub use get_manager_public_key::*;

mod monitor_heads;
pub use monitor_heads::*;

pub struct HttpApi {
    base_url: String,
    client: reqwest::Client,
}

impl HttpApi {
    pub fn new<S: AsRef<str>>(base_url: S) -> Self {
        Self {
            base_url: base_url.as_ref().to_owned(),
            client: reqwest::Client::new(),
        }
    }
}
