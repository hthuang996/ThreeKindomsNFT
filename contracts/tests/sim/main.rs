pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64, U128};
use near_sdk::serde_json::json;
use near_sdk::{AccountId};
use near_sdk_sim::{call, view, deploy, init_simulator, ContractAccount, UserAccount, ExecutionResult, STORAGE_AMOUNT, DEFAULT_GAS};
use nft_factory::ContractContract as NFTFactory;
use nft_loot_box::ContractContract as NFTLootBox;
use nft_hero::ContractContract as NFTHero;
use std::convert::{TryFrom, From};
use near_contract_standards::non_fungible_token::{Token, TokenId};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NFT_FACTORY_BYTES => "res/nft_factory.wasm",
    NFT_HERO_BYTES => "res/nft_hero.wasm",
    NFT_LOOT_BOX_BYTES => "res/nft_loot_box.wasm",
}

fn init() -> (UserAccount, ContractAccount<NFTFactory>,
    ContractAccount<NFTLootBox>, ContractAccount<NFTHero>) {
    let root = init_simulator(None);

    // Deploy the compiled Wasm bytes
    let factory: ContractAccount<NFTFactory> = deploy!(
         contract: NFTFactory,
         contract_id: "factory".to_string(),
         bytes: &NFT_FACTORY_BYTES,
         signer_account: root
     );

     // Deploy the compiled Wasm bytes
    let loot_box: ContractAccount<NFTLootBox> = deploy!(
        contract: NFTLootBox,
        contract_id: "lootbox".to_string(),
        bytes: &NFT_LOOT_BOX_BYTES,
        signer_account: root
    );

    // Deploy the compiled Wasm bytes
    let hero: ContractAccount<NFTHero> = deploy!(
        contract: NFTHero,
        contract_id: "hero".to_string(),
        bytes: &NFT_HERO_BYTES,
        signer_account: root
    );

    (root, factory, loot_box, hero)
}

#[test]
fn simulate_purchase_box() {
    let (root, factory, lootbox, hero) = init();

    // init loot box
    call!(
        root,
        lootbox.new(ValidAccountId::try_from(root.account_id()).unwrap())
    ).assert_success();

    // mint boxes
    call!(
        root,
        lootbox.nft_mint("0".to_string(), ValidAccountId::try_from("factory").unwrap(), "ipfs".into()),
        STORAGE_AMOUNT, 
        DEFAULT_GAS
    ).assert_success();

    // init factory
    call!(
        root,
        factory.new(AccountId::try_from(root.account_id()).unwrap(), 10)
    ).assert_success();

    // set box id
    call!(
        root,
        factory.set_loot_box_id(AccountId::try_from("lootbox").unwrap())
    ).assert_success();

    let num: U128 = view!(
        lootbox.nft_supply_for_owner(ValidAccountId::try_from("factory").unwrap())
    ).unwrap_json();

    // purchase box
    let ret = call!(
        root,
        factory.purchase_box()
    );
    println!("ret:{:?}", ret);

    let nfts: U128 = view!(
        lootbox.nft_supply_for_owner(ValidAccountId::try_from(root.account_id()).unwrap())
    ).unwrap_json();
    assert_eq!(nfts.0, 1);
}

#[test]
fn simulate_unpack_box() {
    let (root, factory, lootbox, hero) = init();

    // init hero
    call!(
        root,
        hero.new(ValidAccountId::try_from("factory").unwrap())
    ).assert_success();
    
    // init loot box
    call!(
        root,
        lootbox.new(ValidAccountId::try_from(root.account_id()).unwrap())
    ).assert_success();

    // mint boxes
    for i in 0..10 {
        call!(
            root,
            lootbox.nft_mint(i.to_string(), ValidAccountId::try_from(root.account_id()).unwrap(), "ipfs".into()),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        ).assert_success();
    }

    // set factory id
    call!(
        root,
        lootbox.set_factory_id(AccountId::try_from("factory").unwrap())
    ).assert_success();

    // init factory
    call!(
        root,
        factory.new(AccountId::try_from(root.account_id()).unwrap(), 10)
    ).assert_success();

    // set box id
    call!(
        root,
        factory.set_loot_box_id(AccountId::try_from("lootbox").unwrap())
    ).assert_success();

    // set hero id
    call!(
        root,
        factory.set_hero_id(AccountId::try_from("hero").unwrap())
    ).assert_success();

    // unpack box
    for i in 0..10 {
        call!(
            root,
            lootbox.unpack(i.to_string()),
            deposit = 1
        ).assert_success();
    }

    // check hero nft
    let num: U128 = view!(
        hero.nft_supply_for_owner(ValidAccountId::try_from(root.account_id()).unwrap())
    ).unwrap_json();
    assert_eq!(num.0, 10);
}