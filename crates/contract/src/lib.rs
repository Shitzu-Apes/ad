use near_contract_standards::{
    fungible_token::{
        events::FtMint,
        metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
        FungibleTokenCore,
    },
    storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement},
};
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, AccountId, NearToken, PanicOnDefault, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
#[allow(deprecated)]
pub struct Contract;

const DEFAULT_AMOUNT: U128 = U128(1_000_000);

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    pub fn mint(&mut self, account_ids: Vec<AccountId>) {
        for account_id in account_ids {
            FtMint {
                owner_id: &account_id,
                amount: DEFAULT_AMOUNT,
                memo: Some("Shitzu ad token"),
            }
            .emit();
        }
    }
}

#[allow(unused_variables)]
#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        FtMint {
            owner_id: &receiver_id,
            amount: DEFAULT_AMOUNT,
            memo: Some("Shitzu ad token"),
        }
        .emit();
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        env::panic_str("unimplemented");
    }

    fn ft_total_supply(&self) -> U128 {
        DEFAULT_AMOUNT
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        DEFAULT_AMOUNT
    }
}

#[allow(unused_variables)]
#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        env::panic_str("unimplemented");
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        env::panic_str("unimplemented");
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        env::panic_str("unimplemented");
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        StorageBalanceBounds {
            min: NearToken::from_yoctonear(0),
            max: None,
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        Some(StorageBalance {
            total: NearToken::from_millinear(50),
            available: NearToken::from_yoctonear(0),
        })
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: "ft-1.0.0".to_string(),
            name: "app.shitzuapes.xyz - Stake Near & earn meme tokens".to_string(),
            symbol: "app.shitzuapes.xyz - Stake Near & earn meme tokens".to_string(),
            icon: Some("data:image/webp;base64,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".to_string()),
            reference: None,
            reference_hash: None,
            decimals: 0,
        }
    }
}
