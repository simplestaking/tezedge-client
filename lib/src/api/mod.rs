/// Defines traits with their types, errors, of the available
/// api actions/operations.

mod operation;
pub use operation::*;

mod contract;
pub use contract::*;

mod get_version_info;
pub use get_version_info::*;

mod get_constants;
pub use get_constants::*;

mod get_protocol_info;
pub use get_protocol_info::*;

mod get_head_block_hash;
pub use get_head_block_hash::*;

mod get_chain_id;
pub use get_chain_id::*;

// TODO: move inside contract/ and rename to get_contract_counter
mod get_counter_for_key;
pub use get_counter_for_key::*;

// TODO: move inside contract/ and rename
mod get_manager_public_key;
pub use get_manager_public_key::*;

// TODO: move inside contract/ and rename
mod get_manager_address;
pub use get_manager_address::*;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct TransportError(pub Box<dyn std::error::Error>);
