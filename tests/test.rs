use std::{collections::BTreeMap, mem};

use bincode;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_program::{
    account_info::AccountInfo,
    borsh::try_from_slice_unchecked,
    instruction::{AccountMeta, Instruction},
    lamports,
    pubkey::Pubkey,
    stake_history::Epoch,
    system_program,
};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    program_pack::Pack, signature::Keypair, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_token::{
    id, instruction,
    state::{Account, Mint},
};
use testsolana::{entrypoint::process_instruction, state::MarketItem};

#[tokio::test]
async fn test_initialize_mint() {
    let program_id = Pubkey::new_unique();
    let program_test = ProgramTest::new(
        "testsolana", // Run the BPF version with `cargo test-bpf`
        program_id,
        processor!(process_instruction), // Run the native version with `cargo test`
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // let mint_account = Keypair::new();
    // let owner = Keypair::new();
    // let token_program = &id();
    // let rent = banks_client.get_rent().await.unwrap();
    // let mint_rent = rent.minimum_balance(Mint::LEN);

    // let token_mint_a_account_ix = solana_program::system_instruction::create_account(
    //     &payer.pubkey(),
    //     &mint_account.pubkey(),
    //     mint_rent,
    //     Mint::LEN as u64,
    //     token_program,
    // );

    // let token_mint_a_ix = instruction::initialize_mint(
    //     token_program,
    //     &mint_account.pubkey(),
    //     &owner.pubkey(),
    //     None,
    //     9,
    // )
    // .unwrap();

    // // create mint transaction
    // let token_mint_a_tx = Transaction::new_signed_with_payer(
    //     &[token_mint_a_account_ix, token_mint_a_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer, &mint_account],
    //     recent_blockhash,
    // );

    // banks_client
    //     .process_transaction(token_mint_a_tx)
    //     .await
    //     .unwrap();

    // // Create account that can hold the newly minted tokens
    // let account_rent = rent.minimum_balance(Account::LEN);
    // let token_account = Keypair::new();
    // let new_token_account_ix = system_instruction::create_account(
    //     &payer.pubkey(),
    //     &token_account.pubkey(),
    //     account_rent,
    //     Account::LEN as u64,
    //     token_program,
    // );

    // let my_account = Keypair::new();
    // let initialize_account_ix = instruction::initialize_account(
    //     token_program,
    //     &token_account.pubkey(),
    //     &mint_account.pubkey(),
    //     &my_account.pubkey(),
    // )
    // .unwrap();

    // let create_new_token_account_tx = Transaction::new_signed_with_payer(
    //     &[new_token_account_ix, initialize_account_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer, &token_account],
    //     recent_blockhash,
    // );
    // banks_client
    //     .process_transaction(create_new_token_account_tx)
    //     .await
    //     .unwrap();

    // // Mint tokens into newly created account
    // let mint_amount: u64 = 10;
    // let mint_to_ix = instruction::mint_to(
    //     &token_program,
    //     &mint_account.pubkey(),
    //     &token_account.pubkey(),
    //     &owner.pubkey(),
    //     &[],
    //     mint_amount.clone(),
    // )
    // .unwrap();

    // let mint_to_tx = Transaction::new_signed_with_payer(
    //     &[mint_to_ix],
    //     Some(&payer.pubkey()),
    //     &[&payer, &owner],
    //     recent_blockhash,
    // );
    // banks_client.process_transaction(mint_to_tx).await.unwrap();

    // // Inspect account
    // let token_account_info = banks_client
    //     .get_account(token_account.pubkey().clone())
    //     .await
    //     .unwrap()
    //     .expect("could not fetch account information");
    // let account_data = Account::unpack(&token_account_info.data).unwrap();
    // println!("account data: {:?}", account_data);
    // assert_eq!(
    //     account_data.amount,
    //     mint_amount.clone(),
    //     "not correct amount"
    // );

    let key = Pubkey::default();
    let mut lamports = 0;
    let mut data = vec![
        0;
        mem::size_of::<u128>()
            + mem::size_of::<Pubkey>()
            + mem::size_of::<Pubkey>()
            + mem::size_of::<Pubkey>()
            + 1
            + mem::size_of::<u128>()
            + 4
            + 4
            + mem::size_of::<u8>()
            + mem::size_of::<bool>()
            + mem::size_of::<bool>()
    ];
    let new_owner = Pubkey::default();

    // let state_account = Pubkey::default();

    #[derive(BorshDeserialize, BorshSerialize)]
    struct InitState<'a> {
        variant: &'a u8,
    }
    let param_init_state = InitState { variant: &4 };

    #[derive(BorshDeserialize, BorshSerialize)]
    struct CreateMarketItem<'a> {
        nft_contract: &'a Pubkey, // program id,
        token_id: &'a Pubkey,     // ATA
        price: &'a u128,
        file_name: &'a String,
        description: &'a String,
        cash_back: &'a u8,
        variant: &'a u8,
    }

    #[derive(BorshDeserialize, BorshSerialize)]
    struct PurchaseSale<'a> {
        nft_contract: &'a Pubkey, // program id,
        token_id: &'a Pubkey,     // ATA
        price: &'a u128,
        variant: &'a u8,
    }

    let param = CreateMarketItem {
        nft_contract: &new_owner, // program id,
        token_id: &new_owner,     // ATA
        price: &1,
        file_name: &"zxczxc".to_string(),
        description: &"zxczxc".to_string(),
        cash_back: &0,
        variant: &0,
    };

    let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state".as_ref()], &program_id);
    let state_account = AccountInfo::new(
        &state_pda,
        false,
        true,
        &mut lamports,
        &mut data,
        &new_owner,
        false,
        Epoch::default(),
    );

    // INIT STATE
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param_init_state,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(*state_account.key, false),
                AccountMeta::new(system_program::ID, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);

    match banks_client.process_transaction(transaction).await {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }

    #[derive(BorshDeserialize, BorshSerialize)]
    pub struct State {
        pub map: BTreeMap<u128, MarketItem>, // 100
        pub item_ids: u128,
        pub item_sold: u128,
        pub owner: Pubkey,
        pub seed: u64,
    }

    let new_state_account = banks_client
        .get_account(*state_account.key)
        .await
        .expect("get_account")
        .expect("state_account not found");
    let state = try_from_slice_unchecked::<State>(&new_state_account.data).unwrap();
    assert_eq!(state.seed, 99999999999);

    // CREATE MARKET ITEM
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(*state_account.key, false),
                AccountMeta::new(system_program::ID, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);

    match banks_client.process_transaction(transaction).await {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }
}
