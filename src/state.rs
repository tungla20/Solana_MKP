use std::collections::BTreeMap;

use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct MarketItem {
    pub item_id: u128,
    pub nft_contract: Pubkey, // program id,
    pub token_id: Pubkey,     // ATA
    pub seller: Pubkey,
    pub owner: Option<Pubkey>,
    pub price: u128,
    pub file_name: String,
    pub description: String,
    pub cash_back: u8,
    pub sold: bool,
    pub gacha: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct State {
    pub map: BTreeMap<u128, MarketItem>, // 100
    pub item_ids: u128,
    pub item_sold: u128,
    pub owner: Pubkey,
    pub seed: u64,
}

impl State {
    pub const LEN: usize = 1 + (4 + (10 * 64)); // 10 user -> blog
}