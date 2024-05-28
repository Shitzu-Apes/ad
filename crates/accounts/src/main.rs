use chrono::prelude::*;
use dashmap::DashMap;
use near_jsonrpc_client::{methods, JsonRpcClient};
use near_jsonrpc_primitives::types::chunks::ChunkReference;
use near_primitives::{
    types::{BlockId, BlockReference, Finality},
    views::BlockView,
};
use near_workspaces::AccountId;
use parking_lot::RwLock;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::{fs, time::sleep};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rpc_client = JsonRpcClient::connect("https://beta.rpc.mainnet.near.org");
    // let rpc_client = JsonRpcClient::connect("http://localhost");

    let block_count = 2_000_000;
    let all_signers = Arc::new(RwLock::new(HashSet::new()));
    let stakers = Arc::new(DashMap::new());

    let block_height = if let Ok(block) = rpc_client
        .call(methods::block::RpcBlockRequest {
            block_reference: BlockReference::Finality(Finality::Final),
        })
        .await
    {
        let block_height = block.header.height;
        handle_block(block, stakers.clone(), &rpc_client, all_signers.clone()).await;
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
        handle_block(block, stakers.clone(), &rpc_client, all_signers.clone()).await;
        block_timestamp
    } else {
        panic!();
    };

    let cpus = num_cpus::get();
    let lock = Arc::new(RwLock::new(0));
    for height in ((block_height - block_count + 1)..block_height).rev() {
        loop {
            if *lock.read() + 1 == cpus * 3 {
                sleep(Duration::from_millis(100)).await;
            } else {
                break;
            }
        }
        {
            let mut lock = lock.write();
            *lock += 1;
        }
        let rpc_client = rpc_client.clone();
        let all_signers = all_signers.clone();
        let stakers = stakers.clone();
        let lock = lock.clone();
        tokio::spawn(async move {
            if let Ok(block) = tryhard::retry_fn(|| {
                rpc_client.call(methods::block::RpcBlockRequest {
                    block_reference: BlockReference::BlockId(BlockId::Height(height)),
                })
            })
            .retries(20)
            .fixed_backoff(Duration::from_millis(100))
            .await
            {
                handle_block(block, stakers, &rpc_client, all_signers.clone()).await;
            }

            if (height + block_count - block_height) % 100 == 0 {
                println!(
                    "remaining: {}; accounts: {}",
                    height + block_count - block_height,
                    all_signers.read().len()
                );
            }
            let mut lock = lock.write();
            *lock -= 1;
        });
    }

    let naive = NaiveDateTime::from_timestamp((first_block_timestamp / 1_000_000_000) as i64, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    dbg!(datetime);

    while Arc::strong_count(&all_signers) > 1 {
        sleep(Duration::from_millis(100)).await;
    }
    let all_signers = Arc::try_unwrap(all_signers).unwrap().into_inner();
    fs::write(
        "account_ids",
        all_signers
            .iter()
            .map(|account_id| account_id.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .await?;

    anyhow::Ok(())
}

async fn handle_block(
    block: BlockView,
    stakers: Arc<DashMap<AccountId, bool>>,
    rpc_client: &JsonRpcClient,
    all_signers: Arc<RwLock<HashSet<AccountId>>>,
) {
    let client = Client::new();
    let near: AccountId = "near".parse().unwrap();
    for chunk in block.chunks {
        if let Ok(chunk) = tryhard::retry_fn(|| {
            rpc_client.call(methods::chunk::RpcChunkRequest {
                chunk_reference: ChunkReference::ChunkHash {
                    chunk_id: chunk.chunk_hash,
                },
            })
        })
        .retries(20)
        .fixed_backoff(Duration::from_millis(100))
        .await
        {
            for transaction in chunk.transactions {
                if !transaction.signer_id.is_sub_account_of(&near) {
                    continue;
                }
                if transaction.signer_id.as_str().contains("relayer") {
                    continue;
                }
                if transaction.signer_id.as_str().contains("oracle") {
                    continue;
                }
                let is_staking =
                    fetch_staking(&transaction.signer_id, stakers.clone(), &client).await;
                if !is_staking {
                    continue;
                }
                if !all_signers.read().contains(&transaction.signer_id) {
                    all_signers.write().insert(transaction.signer_id);
                }
            }
        }
    }
}

async fn fetch_staking(
    account_id: &AccountId,
    stakers: Arc<DashMap<AccountId, bool>>,
    client: &Client,
) -> bool {
    let is_staking = stakers.get_mut(account_id);
    match is_staking {
        Some(is_staking) => *is_staking,
        None => {
            let check_is_staking = if let Ok(res) = tryhard::retry_fn(|| {
                client
                    .get(format!(
                        "https://api.fastnear.com/v1/account/{}/staking",
                        account_id
                    ))
                    .send()
            })
            .retries(20)
            .fixed_backoff(Duration::from_millis(100))
            .await
            {
                if let Ok(staking) = res.json::<StakingResponse>().await {
                    !staking.pools.is_empty()
                } else {
                    false
                }
            } else {
                false
            };
            stakers.insert(account_id.clone(), check_is_staking);
            check_is_staking
        }
    }
}

#[derive(Deserialize)]
pub struct StakingResponse {
    pub pools: Vec<StakedPool>,
}

#[derive(Deserialize)]
pub struct StakedPool {
    pub pool_id: String,
}
