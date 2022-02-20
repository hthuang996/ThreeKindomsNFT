use near_sdk::{
    env, near_bindgen, AccountId, Promise, PromiseOrValue, PromiseResult, ext_contract, log
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::convert::{TryFrom};
// use nft_loot_box::Contract as NFTLootBox;

near_sdk::setup_alloc!();

// define the methods we'll use on ContractB
#[ext_contract(ext_loot_box)]
pub trait ExtLootBox {
    fn nft_tokens_for_owner(
        &self,
        account_id: ValidAccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Token>;
    fn nft_transfer(
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
}

#[ext_contract(ext_hero)]
pub trait ExtHero {
    fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: ValidAccountId,
    );
}

// define methods we'll use as callbacks on ContractA
#[ext_contract(ext_factory)]
pub trait ExtFactory {
    fn nft_tokens_for_owner_callback(&mut self, sender: AccountId) -> Vec<Token>;
    fn nft_transfer_callback(&self);
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    loot_box_id: Option<AccountId>,
    hero_id: Option<AccountId>,
    heros_to_be_minted: Vec<TokenId>
}

const MAX_NFT_NUM: u32 = 10;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, num: u32) -> Self {
        let mut inst = Self {
            owner_id,
            loot_box_id: None,
            hero_id: None,
            heros_to_be_minted: Vec::new(),
        };

        let mut index: u32 = 0;
        while index < num {
            inst.heros_to_be_minted.push(index.to_string());
            index += 1;
        }

        inst
    }

    fn assert_owner(&mut self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id.to_string(), "Only owner can do this");
    }

    pub fn transfer_ownership(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }

    pub fn set_loot_box_id(&mut self, id: AccountId) {
        self.assert_owner();
        self.loot_box_id = Some(id);
    }

    pub fn set_hero_id(&mut self, id: AccountId) {
        self.assert_owner();
        self.hero_id = Some(id);
    }

    pub fn unpack(&mut self, receiver_id: ValidAccountId) -> Promise {
        let sender = env::predecessor_account_id();
        assert_eq!(sender, self.loot_box_id.as_ref().unwrap().clone(), "Can only be called from Loot Box");

        let len: u64 = self.heros_to_be_minted.len() as u64;
        let r = (env::block_timestamp() % len) as usize;
        let token_id = self.heros_to_be_minted[r].clone();
        self.heros_to_be_minted.remove(r);
        ext_hero::nft_mint(token_id, receiver_id, &self.hero_id.as_ref().unwrap(), 5850000000000000000000, 50_000_000_000_000)
    }

    // purchase one loot box
    #[payable]
    pub fn purchase_box(&mut self) -> Promise {
        let sender = env::predecessor_account_id();
        let account_id = env::current_account_id();
        ext_loot_box::nft_tokens_for_owner(ValidAccountId::try_from(account_id.clone()).unwrap(), None, None, &self.loot_box_id.as_ref().unwrap(), 0, 5_000_000_000_000)
        .then(ext_factory::nft_tokens_for_owner_callback(sender, &env::current_account_id(), 0, 50_000_000_000_000))
    }

    fn transfer_to(&mut self, box_id: TokenId, to: ValidAccountId) -> Promise {
        ext_loot_box::nft_transfer(to, box_id, None, None, &self.loot_box_id.as_ref().unwrap(), 1, 5_000_000_000_000)
        .then(ext_factory::nft_transfer_callback(&env::current_account_id(), 0, 5_000_000_000_000))
    }

    #[private]
    pub fn nft_tokens_for_owner_callback(&mut self, sender: AccountId) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let value = near_sdk::serde_json::from_slice::<Vec<Token>>(&result).unwrap();
                if value.len() == 0 {
                    panic!("boxes are sold out");
                }
                else {
                    let token: &Token = &value[0];
                    self.transfer_to(token.token_id.clone(), ValidAccountId::try_from(sender).unwrap());
                }
            }
            _ => {
                panic!("cross contract call failed");
            }
        }
    }

    #[private]
    pub fn nft_transfer_callback(&self) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                log!("transfer successfully");
            }
            _ => {
                panic!("cross contract call failed");
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::testing_env;

    use super::*;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let contract = Contract::new(accounts(0).into(), 10);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.hero_id, None);
        assert_eq!(contract.loot_box_id, None);
    }

    #[test]
    #[should_panic(expected = "Only owner can do this")]
    fn test_transfer_ownership_with_not_owner() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(0).into(), 10);

        // caller is not owner
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(1))
            .build());
        contract.transfer_ownership(accounts(0).into());
    }

    #[test]
    #[should_panic(expected = "Only owner can do this")]
    fn test_set_hero_id_with_not_owner() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(0).into(), 10);

        // caller is not owner
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(1))
            .build());
        contract.set_hero_id(accounts(0).into());
    }

    #[test]
    #[should_panic(expected = "Only owner can do this")]
    fn test_set_loot_box_id_with_not_owner() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(0).into(), 10);

        // caller is not owner
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(1))
            .build());
        contract.set_loot_box_id(accounts(0).into());
    }
}