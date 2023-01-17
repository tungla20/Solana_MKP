use std::{
    collections::{BTreeMap, HashMap},
    mem,
};

use crate::{
    error,
    instruction::{self, GachaMarketplaceInstruction},
    state::{MarketItem, State},
};
use borsh::{BorshDeserialize, BorshSerialize};
use nanorand::{ChaCha, RNG};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    feature, msg,
    program::{self, invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    stake_history::Epoch,
    system_instruction::{self, transfer},
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::{
    instruction::{self as token_instruction, mint_to},
};
use spl_associated_token_account::{
    instruction as token_account_instruction, get_associated_token_address, get_associated_token_address_with_program_id,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // let instruction: GachaMarketplaceInstruction = try_from_slice_unchecked(instruction_data)?;
        let instruction = GachaMarketplaceInstruction::unpack(instruction_data)?;
        println!("//////////////////");
        println!("{:?}", instruction);
        println!("//////////////////");
        match instruction {
            GachaMarketplaceInstruction::CreateMarketItem {
                token_program_id, // program id,
                mint_address,     // ATA
                price,
                file_name,
                description,
                cash_back,
            } => Self::create_market_item(
                accounts,
                program_id,
                token_program_id, // program id,
                mint_address,     // ATA
                price,
                file_name,
                description,
                cash_back,
            ),
            GachaMarketplaceInstruction::PurchaseSale {
                token_program_id,
                price,
                item_id,
            } => Self::purchase_sale(accounts, program_id, token_program_id, price, item_id),
            GachaMarketplaceInstruction::CreateGacha {
                token_program_id,
                qty,
            } => Self::create_gacha(accounts, program_id, token_program_id, qty),
            GachaMarketplaceInstruction::Gacha {
                token_program_id,
                qty,
                price,
                fee,
            } => Self::gacha(accounts, program_id, token_program_id, qty, price, fee),
            GachaMarketplaceInstruction::InitState { listing_price } => {
                Self::init_state(accounts, program_id, listing_price)
            }
        }
    }

    fn init_state(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _listing_price: u128,
    ) -> ProgramResult {
        println!("111111111111111111");
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let state_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (state_pda, state_bump) =
            Pubkey::find_program_address(&[b"state".as_ref()], program_id);

        if state_pda != *state_account.key
            || !state_account.is_writable
            || !state_account.data_is_empty()
        {
            return Err(error::GachaError::InvalidStateAccount.into());
        }

        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(State::LEN);

        let create_map_ix = &system_instruction::create_account(
            authority_account.key,
            state_account.key,
            rent_lamports,
            State::LEN.try_into().unwrap(),
            program_id,
        );

        // msg!("Creating MapAccount account");
        invoke_signed(
            create_map_ix,
            &[
                authority_account.clone(),
                state_account.clone(),
                system_program.clone(),
            ],
            &[&[b"state".as_ref(), &[state_bump]]],
        )?;

        // msg!("Deserializing MapAccount account");
        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow()).unwrap();
        if state.initialized == true {
            return Err(error::GachaError::StateAlreadyInitialized.into());
        }

        let empty_map: BTreeMap<u128, MarketItem> = BTreeMap::new();

        state.map = empty_map;
        state.item_ids = 0;
        state.item_sold = 0;
        state.owner = *authority_account.key;
        state.listing_price = _listing_price;
        state.initialized = true;

        // msg!("Serializing MapAccount account");
        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn create_market_item(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _token_program_id: Pubkey, // program id,
        _mint_address: Pubkey,     // ATA
        _price: u128,
        _file_name: String,
        _description: String,
        _cash_back: u8,
    ) -> ProgramResult {
        println!("222222222222222");
        if _cash_back >= 100 {
            return Err(error::GachaError::CashbackMax.into());
        }
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let state_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        state.item_ids += 1;

        let item: MarketItem = MarketItem {
            item_id: state.item_ids,
            token_program_id: _token_program_id,
            mint_address: _mint_address,
            seller: *authority_account.key,
            owner: None,
            price: _price,
            file_name: _file_name,
            description: _description,
            cash_back: _cash_back,
            sold: false,
            gacha: false,
        };

        state.map.insert(state.item_ids, item.clone());
        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
        
        let token_address = get_associated_token_address_with_program_id(authority_account.key, &_mint_address, program_id);
        // transfer nft from sender to this contract
        invoke(
            &spl_token::instruction::transfer_checked(
                &_token_program_id,
                &authority_account.key,
                &_mint_address,
                &token_address,
                authority_account.key,
                &[authority_account.key],
                1,
                0,
            )
            .unwrap(),
            &[authority_account.clone(), system_program.clone()],
        )?;
        Ok(())
    }

    fn purchase_sale(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _nft_contract: Pubkey, // program id
        _price: u128,
        _item_id: u128,
    ) -> ProgramResult {
        println!("33333333333333");
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let state_account = next_account_info(account_info_iter)?;
        let item_seller = next_account_info(account_info_iter)?;
        let owner_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let mut item = state.map.get(&_item_id).unwrap().to_owned();
        let price = item.price;
        let mint_address = item.mint_address;

        if _price != price {
            return Err(error::GachaError::InvalidPayment.into());
        }

        // transfer price to seller
        // need item.seller accountInfo
        invoke(
            &transfer(
                authority_account.key,
                &item.seller,
                price.try_into().unwrap(),
            ),
            &[authority_account.to_owned(), item_seller.to_owned()],
        )?;
        println!("zxczxczczxczxczxc");
        // transfer nft from contract to sender
        invoke(
            &spl_token::instruction::transfer_checked(
                &item.token_program_id,
                &system_program.key,
                &mint_address,
                authority_account.key,
                system_program.key,
                &[system_program.key],
                1,
                0,
            )
            .unwrap(),
            &[authority_account.clone(), system_program.clone()],
        )?;

        item.owner = Some(*authority_account.key);
        item.sold = true;

        // transfer listing price to owner
        // need owner accountInfo
        invoke(
            &transfer(
                authority_account.key,
                &owner_account.key,
                price.try_into().unwrap(),
            ),
            &[authority_account.to_owned(), owner_account.to_owned()],
        )?;

        state.item_sold += 1;
        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
        Ok(())
    }

    fn gacha(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _nft_contract: Pubkey,
        _qty: u8,
        _price: u128,
        _fee: u128,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let state_account = next_account_info(account_info_iter)?;
        let seller_0th_item_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let item_count = state.item_ids;
        let item_sold = state.item_sold;
        let unsold_item_count = item_count - item_sold;
        let mut current_index = 0;

        let mut items = HashMap::new();
        for i in 0..item_count {
            let current_id = i + 1;
            let item = state.map.get(&current_id).unwrap().to_owned();
            if item.owner != None && item.price == _price {
                items.insert(current_index, item);
                current_index += 1;
            }
        }

        let mut gacha_items = HashMap::new();
        for i in 0.._qty {
            let mut rng = ChaCha::new(254);
            let gacha_index = rng.generate_range(0, (unsold_item_count - 1) as u64) as u128;
            let item = state.map.get(&gacha_index).unwrap().to_owned();
            gacha_items.insert(i, item);
            items.remove(&gacha_index);
        }

        for i in 0.._qty {
            let item_id = gacha_items.get(&i).unwrap().item_id;
            let mut selected_item = state.map.get(&item_id).unwrap().to_owned();
            let mint_address = selected_item.mint_address;

            // transfer nft
            invoke(
                &spl_token::instruction::transfer_checked(
                    &selected_item.token_program_id,
                    &system_program.key,
                    &mint_address,
                    authority_account.key,
                    system_program.key,
                    &[system_program.key],
                    1,
                    0,
                )
                .unwrap(),
                &[authority_account.clone(), system_program.clone()],
            )?;

            selected_item.owner = Some(*authority_account.key);
            selected_item.gacha = true;
        }

        // transfer fee to seller map[0] // accountInfo receiver
        invoke(
            &transfer(
                authority_account.key,
                &seller_0th_item_account.key,
                _fee.try_into().unwrap(),
            ),
            &[
                authority_account.to_owned(),
                seller_0th_item_account.to_owned(),
            ],
        )?;

        state.item_sold += 1;
        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
        Ok(())
    }

    fn create_gacha(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _nft_contract: Pubkey,
        _qty: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let state_account = next_account_info(account_info_iter)?;
        let owner_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let item_count = state.item_ids;
        let item_sold = state.item_sold;
        let unsold_item_count = item_count - item_sold;
        let mut current_index = 0;

        let mut items = HashMap::new();
        for i in 0..item_count {
            let current_id = i + 1;
            let item = state.map.get(&current_id).unwrap().to_owned();
            if item.owner != None {
                items.insert(current_index, item);
                current_index += 1;
            }
        }

        let mut gacha_items = HashMap::new();
        let mut len = items.len();
        for i in 0.._qty {
            let mut rng = ChaCha::new(254);
            let index = rng.generate_range(0, len - 1);
            let item = items.get(&index).unwrap().to_owned();
            gacha_items.insert(i, item);
            items.remove(&index);
            len -= 1;
        }

        for i in 0.._qty {
            let item = gacha_items.get(&i).unwrap().to_owned();
            let item_id = item.item_id;
            let mint_address = item.mint_address;

            // transfer nft
            invoke(
                &spl_token::instruction::transfer_checked(
                    &item.token_program_id,
                    &system_program.key,
                    &mint_address,
                    authority_account.key,
                    system_program.key,
                    &[system_program.key],
                    1,
                    0,
                )
                .unwrap(),
                &[authority_account.clone(), system_program.clone()],
            )?;

            let mut item = state.map.get(&item_id).unwrap().to_owned();
            item.owner = Some(state.owner);
            item.gacha = true;

            state.item_sold += 1;
            invoke(
                &transfer(
                    authority_account.key,
                    &owner_account.key,
                    state.listing_price.try_into().unwrap(),
                ),
                &[authority_account.to_owned(), owner_account.to_owned()],
            )?;
        }

        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
        Ok(())
    }
}
