use std::fmt::{self, Display};
use futures_util::{FutureExt, StreamExt};

use types::{BlockHash, OperationHash, FromPrefixedBase58CheckError};
use crate::{BoxFuture, BoxStream};
use crate::api::{
    MonitorHeadsAsync,
    BlockGetOperationsAsync, BlockOperation, BlockOperationContent,
    TransportError, MonitorHeadsError, BlockGetOperationsError,
};

#[derive(thiserror::Error, Debug)]
pub enum MonitorOperationsError {
    ParseChunk(#[from] serde_json::Error),
    Base58Decode(#[from] FromPrefixedBase58CheckError),
    Transport(#[from] TransportError),
    Unknown(String),
}

impl From<MonitorHeadsError> for MonitorOperationsError {
    fn from(error: MonitorHeadsError) -> Self {
        use MonitorHeadsError::*;

        match error {
            ParseChunk(err) => Self::ParseChunk(err),
            Transport(err) => Self::Transport(err),
            Unknown(err) => Self::Unknown(err),
        }
    }
}

impl From<BlockGetOperationsError> for MonitorOperationsError {
    fn from(error: BlockGetOperationsError) -> Self {
        use BlockGetOperationsError::*;

        match error {
            Base58Decode(err) => Self::Base58Decode(err),
            Transport(err) => Self::Transport(err),
            Unknown(err) => Self::Unknown(err),
        }
    }
}

impl Display for MonitorOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        write!(f, "Monitoring contracts failed! Reason: ")?;
        match self {
            Self::ParseChunk(err) => err.fmt(f),
            Self::Base58Decode(err) => err.fmt(f),
            Self::Transport(err) => err.fmt(f),
            Self::Unknown(err) => err.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitoredOperation {
    pub block_level: u64,
    pub block_hash: BlockHash,
    pub hash: OperationHash,
    pub contents: Vec<BlockOperationContent>,
}

impl MonitoredOperation {
    fn new(block_level: u64, block_op: BlockOperation) -> Self {
        Self {
            block_level,
            block_hash: block_op.branch,
            hash: block_op.hash,
            contents: block_op.contents,
        }
    }
}

use std::future::Ready;
use futures_core::stream::Stream;

#[inline]
fn value_to_stream<T>(value: T) -> impl Stream<Item = T> {
    futures_util::stream::once(
        futures_util::future::ready(value)
    )
}

pub type MonitorOperationsResult = Result<MonitoredOperation, MonitorOperationsError>;
pub type StartMonitorOperationsResult = Result<BoxStream<'static, MonitorOperationsResult>, MonitorHeadsError>;

pub trait MonitorOperationsAsync {
    /// Monitor contracts.
    ///
    /// Monitors new blocks and finds operations which reference given contracts.
    fn monitor_operations(
        &self,
    ) -> BoxFuture<'static, StartMonitorOperationsResult>;
}

impl<U> MonitorOperationsAsync for U
    where U: 'static + MonitorHeadsAsync + BlockGetOperationsAsync + Clone + Send + Sync,
{
    fn monitor_operations(
        &self,
    ) -> BoxFuture<'static, StartMonitorOperationsResult>
    {
        let monitor_heads_fut = self.monitor_heads();
        let client = self.clone();
        Box::pin(async move {
            Ok(monitor_heads_fut.await?
                .map(move |result| {
                    let client = client.clone();
                    async move {
                        match result {
                            Ok(head) => {
                                println!("received head: {}", head.level);
                                match client.block_get_operations(&head.hash).await {
                                    Ok(ops) => {
                                        println!("received ops for head: {}", head.level);
                                        let ops_iter = ops.into_iter()
                                            .map(move |op| Ok(MonitoredOperation::new(head.level, op)));

                                        futures_util::stream::iter(ops_iter).boxed()
                                    }
                                    Err(err) => value_to_stream(Err(err.into())).boxed(),
                                }
                            },
                            Err(err) => value_to_stream(Err(err.into())).boxed(),
                        }
                    }
                })
                .flat_map(|fut| fut.into_stream())
                .flatten()
                // .flat_map(|res| {
                //     match res {
                //         Ok(ops) => 
                //             ops.into_iter().map(|x| Ok(MonitoredOperation::new(x)))
                //         ).boxed(),
                //         Err(err) => futures_util::stream::once(
                //             std::future::ready(Err(err))
                //         ).boxed(),
                //     }
                // })
                .boxed())
        })
    }
}
