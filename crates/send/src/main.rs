use near_workspaces::{
    operations::Function,
    types::{Gas, NearToken},
    Account,
};
use serde_json::json;
use std::env;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    let worker = near_workspaces::custom("https://rpc.shitzuapes.xyz").await?;
    let sender = Account::from_secret_key(
        env::var("ACCOUNT_ID")?.parse()?,
        env::var("PRIVATE_KEY")?.parse()?,
        &worker,
    );

    let accounts = fs::read_to_string("./account_ids").await?;
    let accounts: Vec<_> = accounts.split('\n').collect();

    for accounts in accounts.chunks(500) {
        let mut batch = sender.batch(sender.id());
        for accounts in accounts.chunks(25) {
            batch = batch.call(
                Function::new("mint")
                    .args_json(json!({
                        "account_ids": accounts
                    }))
                    .gas(Gas::from_tgas(15)),
            );
        }
        let _ = batch.transact_async().await?;
    }

    for accounts in accounts.chunks(25) {
        let mut batch = sender.batch(&"token.0xshitzu.near".parse()?);
        for account in accounts {
            batch = batch
                .call(
                    Function::new("storage_deposit")
                        .args_json(json!({
                            "account_id": account,
                            "registration_only": true,
                        }))
                        .gas(Gas::from_tgas(5))
                        .deposit(NearToken::from_yoctonear(1_250_000_000_000_000_000_000)),
                )
                .call(
                    Function::new("ft_transfer")
                        .args_json(json!({
                            "receiver_id": account,
                            "amount": "50000000000000000000",
                            "memo": "Shitzu airdrop"
                        }))
                        .gas(Gas::from_tgas(7))
                        .deposit(NearToken::from_yoctonear(1)),
                );
        }
        let _ = batch.transact_async().await?;
    }

    Ok(())
}
