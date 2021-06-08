use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

use crate::BoxFuture;
use crate::api::TransportError;
use types::Network;

#[derive(thiserror::Error, Debug)]
pub enum GetVersionInfoError {
    Transport(#[from] TransportError),
    Unknown(String),
}

impl Display for GetVersionInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "getting version information failed! Reason: ")?;
        match self {
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => write!(f, "Unknown! {}", err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default, Clone)]
pub struct NodeVersion {
    major: usize,
    minor: usize,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default, Clone)]
pub struct NetworkVersion {
    pub chain_name: String,
    pub distributed_db_version: usize,
    pub p2p_version: usize,
}

impl NetworkVersion {
    pub fn get_network(&self) -> Network {
        self.chain_name.parse().unwrap()
    }

    pub fn is_mainnet(&self) -> bool {
        matches!(
            self.get_network(),
            Network::Main(_) | Network::Beta(_),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default, Clone)]
pub struct CommitInfo {
    pub commit_hash: String,
    pub commit_date: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default, Clone)]
pub struct VersionInfo {
    #[serde(rename = "version")]
    pub node_version: NodeVersion,
    pub network_version: NetworkVersion,
    pub commit_info: CommitInfo,
}

impl VersionInfo {
    pub fn is_mainnet(&self) -> bool {
        self.network_version.is_mainnet()
    }

    pub fn get_network(&self) -> Network {
        self.network_version.get_network()
    }
}

pub type GetVersionInfoResult = Result<VersionInfo, GetVersionInfoError>;

pub trait GetVersionInfo {
    fn get_version_info(&self) -> GetVersionInfoResult;
}

pub trait GetVersionInfoAsync {
    fn get_version_info<'a>(&'a self) -> BoxFuture<'a, GetVersionInfoResult>;
}

pub(crate) fn get_version_info_url(base_url: &str) -> String {
    format!("{}/version", base_url)
}
