use std::collections::BTreeMap;

use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{pubkey::Pubkey, account_info::AccountInfo};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct MarketItem {
    pub item_id: u128,
    pub token_program_id: Pubkey, // program id,
    pub mint_address: Pubkey,
    pub seller: Pubkey,
    pub owner: Option<Pubkey>,
    pub price: u128,
    pub file_name: String,
    pub description: String,
    pub cash_back: u8,
    pub sold: bool,
    pub gacha: bool,
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct State {
    pub map: BTreeMap<u128, MarketItem>, // 100
    pub item_ids: u128,
    pub item_sold: u128,
    pub owner: Pubkey,
    pub listing_price: u128,
    pub initialized: bool
}

impl State {
    pub const LEN: usize = 1 + (4 + (10 * 64)); // 10 user -> blog
}