use chrono::prelude::*;
use futures::future::join_all;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::chunks::ChunkReference;
use near_primitives::{
    types::{BlockId, BlockReference, Finality},
    views::BlockView,
};
use near_workspaces::AccountId;
use parking_lot::RwLock;
use std::{collections::HashSet, sync::Arc};
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc_client = JsonRpcClient::connect("https://beta.rpc.mainnet.near.org");
    // let rpc_client = JsonRpcClient::connect("http://localhost");

    let block_count = 500_000;
    let all_signers = Arc::new(RwLock::new(HashSet::new()));

    let block_height = if let Ok(block) = rpc_client
        .call(methods::block::RpcBlockRequest {
            block_reference: BlockReference::Finality(Finality::Final),
        })
        .await
    {
        let block_height = block.header.height;
        handle_block(block, &rpc_client, all_signers.clone()).await;
        block_height
    } else {
        panic!();
    };
    let first_block_timestamp = if let Ok(block) = rpc_client
        .call(methods::block::RpcBlockRequest {
            block_reference: BlockReference::BlockId(BlockId::Height(block_height - block_count)),
        })
        .await
    {
        let block_timestamp = block.header.timestamp;
        handle_block(block, &rpc_client, all_signers.clone()).await;
        block_timestamp
    } else {
        panic!();
    };
    let tasks = ((block_height - block_count + 1)..block_height)
        .rev()
        .map(|height| {
            let rpc_client = rpc_client.clone();
            let all_signers = all_signers.clone();
            tokio::spawn(async move {
                if let Ok(block) = rpc_client
                    .call(methods::block::RpcBlockRequest {
                        block_reference: BlockReference::BlockId(BlockId::Height(height)),
                    })
                    .await
                {
                    if (height - block_height + block_count) % 10_000 == 0 {
                        println!("remaining: {}", height - block_height + block_count);
                    }
                    handle_block(block, &rpc_client, all_signers.clone()).await;
                }
            })
        })
        .collect::<Vec<_>>();
    join_all(tasks).await;
    // let mut i = 0;
    // for block_height in (block_height - block_count + 1)..block_height {
    //     if let Ok(block) = rpc_client
    //         .call(methods::block::RpcBlockRequest {
    //             block_reference: BlockReference::BlockId(BlockId::Height(block_height)),
    //         })
    //         .await
    //     {
    //         handle_block(block, &rpc_client, all_signers.clone()).await;
    //     }
    //     i += 1;
    //     if i % 100 == 0 {
    //         println!("remaining: {}", block_count - i);
    //     }
    // }

    dbg!(all_signers.read().len());
    let naive = NaiveDateTime::from_timestamp((first_block_timestamp / 1_000_000_000) as i64, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    dbg!(datetime);

    dbg!();
    let all_signers = Arc::try_unwrap(all_signers).unwrap().into_inner();
    // fs::write(
    //     "account_ids",
    //     all_signers
    //         .iter()
    //         .map(|account_id| account_id.as_str())
    //         .collect::<Vec<_>>()
    //         .join("\n"),
    // )
    // .await?;

    anyhow::Ok(())
}

async fn handle_block(
    block: BlockView,
    rpc_client: &JsonRpcClient,
    all_signers: Arc<RwLock<HashSet<AccountId>>>,
) {
    let near: AccountId = "near".parse().unwrap();
    for chunk in block.chunks {
        if let Ok(chunk) = rpc_client
            .call(methods::chunk::RpcChunkRequest {
                chunk_reference: ChunkReference::ChunkHash {
                    chunk_id: chunk.chunk_hash,
                },
            })
            .await
        {
            for transaction in chunk.transactions {
                if !transaction.signer_id.as_str().ends_with(".near") {
                    continue;
                }
                if !transaction.signer_id.is_sub_account_of(&near) {
                    continue;
                }
                if transaction.signer_id.as_str().contains("relayer") {
                    continue;
                }
                if transaction.signer_id.as_str().contains("oracle") {
                    continue;
                }
                if !all_signers.read().contains(&transaction.signer_id) {
                    all_signers.write().insert(transaction.signer_id);
                }
            }
        }
    }
}
