//! Tezos Node's RPC
//!
//! [rpc_api::api] defines traits and types for the api.
//!
//! Then there can be multiple implementation for those traits, like:
//! [rpc_api::http_api], which interacts with the node using **http** protocol.
//! To use it, when adding a dependency, use the feature: "sync", like this:
//! `rpc_api = { version = "*", features = ["sync"] }`
//!
//! [rpc_api::http_api_async], which interacts with the node using **http** protocol asynchronously.
//! To use it, when adding a dependency, use the feature: "async", like this:
//! `rpc_api = { version = "*", features = ["async"] }`
//!
//! Other protocols like **WebSockets**, **GRPC + Protobuf**, etc... Can
//! be implemented using traits defined in [rpc_api::api], If the target
//! node supports chosen protocol.
//!
//!
//! Each endpoint has it's own error types. Common error is [rpc_api::api::TransportError].
//! It will contain errors, which happened because of transport issues
//! like: connection was lost, timeout, etc...

use std::pin::Pin;
use std::future::Future;

pub mod api;
#[cfg(feature = "sync")]
pub mod http_api;
#[cfg(feature = "async")]
pub mod http_api_async;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type BoxStream<'a, T> = futures_core::stream::BoxStream<'a, T>;
