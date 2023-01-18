use std::{collections::BTreeMap, mem};

use borsh::{BorshDeserialize, BorshSerialize};
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
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account, ID
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

    let mint_account = Keypair::new();
    let token_program = &id();
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let token_mint_a_account_ix = solana_program::system_instruction::create_account(
        &payer.pubkey(),
        &mint_account.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        token_program,
    );

    let token_mint_a_ix = instruction::initialize_mint(
        token_program,
        &mint_account.pubkey(),
        &payer.pubkey(),
        None,
        0,
    )
    .unwrap();

    // create mint transaction
    let token_mint_a_tx = Transaction::new_signed_with_payer(
        &[token_mint_a_account_ix, token_mint_a_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(token_mint_a_tx)
        .await
        .unwrap();

    // Create account that can hold the newly minted tokens
    let account_rent = rent.minimum_balance(Account::LEN);
    let token_account = Keypair::new();
    let new_token_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &token_account.pubkey(),
        account_rent,
        Account::LEN as u64,
        token_program,
    );

    let initialize_account_ix = instruction::initialize_account(
        token_program,
        &token_account.pubkey(),
        &mint_account.pubkey(),
        &payer.pubkey(),
    )
    .unwrap();

    let create_new_token_account_tx = Transaction::new_signed_with_payer(
        &[new_token_account_ix, initialize_account_ix],
        Some(&payer.pubkey()),
        &[&payer, &token_account],
        recent_blockhash,
    );
    banks_client
        .process_transaction(create_new_token_account_tx)
        .await
        .unwrap();

    // Mint tokens into newly created account
    let mint_amount: u64 = 1;
    let mint_to_ix = instruction::mint_to(
        &token_program,
        &mint_account.pubkey(),
        &token_account.pubkey(),
        &payer.pubkey(),
        &[],
        mint_amount.clone(),
    )
    .unwrap();

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer, &payer],
        recent_blockhash,
    );
    banks_client.process_transaction(mint_to_tx).await.unwrap();

    // Inspect account
    let token_account_info = banks_client
        .get_account(token_account.pubkey().clone())
        .await
        .unwrap()
        .expect("could not fetch account information");
    let account_data = Account::unpack(&token_account_info.data).unwrap();
    println!("account data: {:?}", account_data);
    assert_eq!(
        account_data.amount,
        mint_amount.clone(),
        "not correct amount"
    );

    let rs = banks_client
        .get_account(account_data.mint)
        .await
        .unwrap()
        .expect("could not fetch account information");
    println!("...........{:?}", rs);

    let other_user = Keypair::new();
    let new_owner = Pubkey::default();

    #[derive(BorshDeserialize, BorshSerialize)]
    struct GachaMarketplacePayload {
        token_program_id: Pubkey, // program id,
        mint_address: Pubkey,     // ATA
        price: u128,
        file_name: String,
        description: String,
        cash_back: u8,
        qty: u8,
        fee: u128,
        item_id: u128,
        listing_price: u128,
        variant: u8,
    }

    let (state_pda, state_bump) = Pubkey::find_program_address(&[b"state".as_ref()], &program_id);

    // INIT STATE
    let param_init_state = GachaMarketplacePayload {
        token_program_id: *token_program,     // program id,
        mint_address: token_account.pubkey(), // ATA
        price: 1,
        file_name: "zxczxc".to_string(),
        description: "zxczxc".to_string(),
        cash_back: 0,
        variant: 4,
        qty: 0,
        fee: 0,
        item_id: 0,
        listing_price: 1,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param_init_state,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(state_pda, false),
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

    // let new_state_account_after = banks_client
    //     .get_account(other_user.pubkey())
    //     .await
    //     .expect("get_account")
    //     .expect("state_account not found");
    // assert_eq!(new_state_account_after.lamports, 5380080);
    let new_account = Keypair::new();
    let (nft_pda, nft_bump) = Pubkey::find_program_address(
        &[b"nft".as_ref(), &token_account.pubkey().to_bytes()],
        &program_id,
    );

    let mint_keypair = Keypair::new();
    let token_address = get_associated_token_address(&mint_keypair.pubkey(), &payer.pubkey());

    // CREATE MARKET ITEM
    let param_create_item = GachaMarketplacePayload {
        token_program_id: *token_program,     // program id,
        mint_address: token_account.pubkey(), // ATA
        price: 1,
        file_name: "zxczxc".to_string(),
        description: "zxczxc".to_string(),
        cash_back: 0,
        variant: 0,
        qty: 0,
        fee: 0,
        item_id: 0,
        listing_price: 0,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param_create_item,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(state_pda, false),
                // AccountMeta::new(nft_pda, false),
                AccountMeta::new(mint_keypair.pubkey(), true),
                AccountMeta::new(token_account.pubkey(), false),
                AccountMeta::new_readonly(id(), false),
                AccountMeta::new_readonly(ID, false),
                // AccountMeta::new(system_program::ID, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &mint_keypair], recent_blockhash);

    match banks_client.process_transaction(transaction).await {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }

    let new_state_account_after = banks_client
        .get_account(other_user.pubkey())
        .await
        .expect("get_account")
        .expect("state_account not found");
    assert_eq!(new_state_account_after.lamports, 5381080);

    // ANOTHER MARKET ITEM
    let param_create_item2 = GachaMarketplacePayload {
        token_program_id: *token_program,     // program id,
        mint_address: token_account.pubkey(), // ATA
        price: 3,
        file_name: "file_name".to_string(),
        description: "file_name".to_string(),
        cash_back: 0,
        variant: 0,
        qty: 0,
        fee: 0,
        item_id: 0,
        listing_price: 0,
    };
    let mut transaction2 = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param_create_item2,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(state_pda, false),
                AccountMeta::new(system_program::ID, false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    transaction2.sign(&[&payer], recent_blockhash);

    match banks_client.process_transaction(transaction2).await {
        Ok(()) => (),
        Err(e) => panic!("{}", e),
    }

    let new_state_account = banks_client
        .get_account(state_pda)
        .await
        .expect("get_account")
        .expect("state_account not found");
    let state = try_from_slice_unchecked::<State>(&new_state_account.data).unwrap();
    assert_eq!(state.map.len(), 2);
    assert_eq!(state.map.get(&1).unwrap().file_name, "zxczxc");

    // PURCHASE SALE
    let param_create_item = GachaMarketplacePayload {
        token_program_id: *token_program,     // program id,
        mint_address: token_account.pubkey(), // ATA
        price: 3,

        file_name: "file_name".to_string(),
        description: "file_name".to_string(),
        cash_back: 0,
        variant: 0,
        qty: 0,
        fee: 0,
        item_id: 0,
        listing_price: 0,
    };
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &param_create_item,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(state_pda, false),
                AccountMeta::new(
                    state.map.get(&param_create_item.item_id).unwrap().seller,
                    false,
                ),
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
