use serde::Deserialize;

use crypto::ToBase58Check;
use types::BlockHash;

use crate::BoxFuture;
use crate::api::{
    block_get_operations_url,
    BlockGetOperationsAsync, BlockGetOperationsResult,
    TransportError, BlockGetOperationsError,
    BlockOperation, BlockOperationContent,
};
use crate::http_api_async::HttpApi;

impl From<reqwest::Error> for BlockGetOperationsError {
    fn from(error: reqwest::Error) -> Self {
        if let Some(status) = error.status() {
            Self::Unknown(format!(
                "Http status: ({}) {}",
                status,
                error,
            ))
        } else {
            Self::Transport(TransportError(Box::new(error)))
        }
    }
}

#[derive(Deserialize)]
struct BlockOperations(Vec<Vec<BlockOperation>>);

impl BlockGetOperationsAsync for HttpApi {
    fn block_get_operations<'a>(&'a self, block: &'a BlockHash) -> BoxFuture<'a, BlockGetOperationsResult> {
        let block_hash = block.to_base58check();
        let url = block_get_operations_url(&self.base_url, &block_hash);
        Box::pin(async move {
            let ops = self.client.get(&url)
               .send().await?
               .json::<BlockOperations>().await?;

            Ok(ops.0.into_iter().flatten().collect())
        })
    }
}
