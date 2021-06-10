use std::fmt::{self, Display};
use std::time::Instant;
use serde::{Serialize, Deserialize};

use types::BlockHash;
use crate::{BoxFuture, BoxStream};
use crate::api::TransportError;

#[derive(thiserror::Error, Debug)]
pub enum MonitorHeadsError {
    ParseChunk(#[from] serde_json::Error),
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for MonitorHeadsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error while monitoring heads! Reason: ")?;
        match self {
            Self::ParseChunk(err) => err.fmt(f),
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct BlockHead {
    pub level: u64,
    pub hash: BlockHash,
    pub predecessor: BlockHash,
    pub timestamp: String,
}

pub type MonitorHeadsResult = Result<BlockHead, MonitorHeadsError>;
pub type StartMonitorHeadsResult<'a> = Result<BoxStream<'a, MonitorHeadsResult>, MonitorHeadsError>;

pub trait MonitorHeadsAsync {
    fn monitor_heads<'a>(&'a self) -> BoxFuture<'a, StartMonitorHeadsResult<'a>>;
}

pub(crate) fn monitor_heads_url(base_url: &str, chain_name: &str) -> String {
    format!("{}/monitor/heads/{}", base_url, chain_name)
}
