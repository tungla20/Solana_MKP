use std::collections::{BTreeMap, HashMap};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    error,
    instruction::{GachaMarketplaceInstruction, self},
    state::{MarketItem, State},
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        println!("------------------");
        // let instruction: GachaMarketplaceInstruction = try_from_slice_unchecked(instruction_data)?;
        let instruction = GachaMarketplaceInstruction::unpack(instruction_data)?;
        println!("------------------");
        println!("//////////////////");
        println!("{:?}", instruction);
        println!("//////////////////");
        match instruction {
            GachaMarketplaceInstruction::CreateMarketItem {
                nft_contract, // program id,
                token_id,     // ATA
                price,
                file_name,
                description,
                cash_back,
            } => Self::create_market_item(
                accounts,
                program_id,
                nft_contract, // program id,
                token_id,     // ATA
                price,
                file_name,
                description,
                cash_back,
            ),
            GachaMarketplaceInstruction::PurchaseSale {
                nft_contract,
                price,
                item_id,
            } => Self::purchase_sale(accounts, program_id, nft_contract, price, item_id),
            GachaMarketplaceInstruction::CreateGacha { nft_contract, qty } => {
                Self::create_gacha(accounts, program_id, nft_contract, qty)
            }
            GachaMarketplaceInstruction::Gacha {
                nft_contract,
                qty,
                price,
                fee,
            } => Self::gacha(accounts, program_id, nft_contract, qty, price, fee),
            GachaMarketplaceInstruction::InitState {} => Self::init_state(accounts, program_id),
        }
    }

    fn init_state(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
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

        msg!("Creating MapAccount account");
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
        let empty_map: BTreeMap<u128, MarketItem> = BTreeMap::new();

        state.map = empty_map;
        state.item_ids = 0;
        state.item_sold = 0;
        state.owner = *authority_account.key;
        state.seed = 99999999999;

        // msg!("Serializing MapAccount account");
        state.serialize(&mut &mut state_account.data.borrow_mut()[..])?;
        Ok(())
    }

    fn create_market_item(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _nft_contract: Pubkey, // program id,
        _token_id: Pubkey,     // ATA
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

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        state.item_ids += 1;

        let item: MarketItem = MarketItem {
            item_id: state.item_ids,
            nft_contract: _nft_contract,
            token_id: _token_id,
            seller: *authority_account.key,
            owner: None,
            price: _price,
            file_name: _file_name,
            description: _description,
            cash_back: _cash_back,
            sold: false,
            gacha: false,
        };

        state.map.insert(state.item_ids, item);

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

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let mut item = state.map.get(&_item_id).unwrap().to_owned();
        let price = item.price;
        let token_id = item.token_id;

        if _price != price {
            return Err(error::GachaError::InvalidPayment.into());
        }

        // transfer price to seller
        // transfer nft from contract to sender

        item.owner = Some(*authority_account.key);
        item.sold = true;

        // transfer listing price to owner

        state.item_sold += 1;

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

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let item_count = state.item_ids;
        let item_sold = state.item_sold;
        let unsold_item_count = item_count - item_sold;
        let mut current_index = 0;
        let mut seed = state.seed;

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
            seed ^= seed >> 12;
            seed ^= seed << 25;
            seed ^= seed >> 27;
            seed *= 0x2545F4914F6CDD1D;

            let gacha_index = (seed % unsold_item_count as u64) as u128;
            let item = state.map.get(&gacha_index).unwrap().to_owned();
            gacha_items.insert(i, item);
            items.remove(&gacha_index);
        }

        for i in 0.._qty {
            let item_id = gacha_items.get(&i).unwrap().item_id;
            let mut selected_item = state.map.get(&item_id).unwrap().to_owned();
            let price = selected_item.price;
            let token_id = selected_item.token_id;

            // transfer nft

            selected_item.owner = Some(*authority_account.key);
            selected_item.gacha = true;
        }

        // transfer fee to seller map[0]

        state.item_sold += 1;
        /*
        items[MarketItem; unsold_item_count] // =price + owner != none
        gacha_items[MarketItem; _qty] // random get item from items to gacha_items
        transfer all nft in gacha_items to sender
            .owner = sender
            .gacha = true
        transfer fee to MarketItem[0].seller
        ctx.accounts.items_sold.count ++
        */
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

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut state = try_from_slice_unchecked::<State>(&state_account.data.borrow())?;

        let item_count = state.item_ids;
        let item_sold = state.item_sold;
        let unsold_item_count = item_count - item_sold;
        let mut current_index = 0;
        let mut seed = state.seed;

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
            seed ^= seed >> 12;
            seed ^= seed << 25;
            seed ^= seed >> 27;
            seed *= 0x2545F4914F6CDD1D;

            let index = (seed % len as u64) as u8;
            let item = items.get(&index).unwrap().to_owned();
            gacha_items.insert(i, item);
            items.remove(&index);
            len -= 1;
        }

        for i in 0.._qty {
            let item_id = gacha_items.get(&i).unwrap().to_owned().item_id;
            let token_id = gacha_items.get(&i).unwrap().to_owned().token_id;

            // transfer nft

            let mut item = state.map.get(&item_id).unwrap().to_owned();
            item.owner = Some(state.owner);
            item.gacha = true;

            state.item_sold += 1;
        }

        // let mut map_state = anchor_lang::solana_program::borsh::try_from_slice_unchecked:: <MapAccount>(&map_account.data.borrow())?;

        /*
        items[MarketItem; unsold_item_count] // owner != none
        gacha_items[MarketItem; _qty] // random get item from items (items.len--) to gacha_items
        transfer all nft in gacha_items to sender
            .owner = sender
            .gacha = true
            ctx.accounts.items_sold.count ++
            tranfer listing_price to owner of program_id
        */
        Ok(())
    }
}
