#[cfg(feature = "client")]
pub mod factory;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

// NOTE could hold a reference to description and metadata args
// to avoid cloning them, in the factory, but performance is not
// crucial in that part of the code.
#[allow(clippy::large_enum_variant)]
#[repr(C)]
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum GachaMarketplaceInstruction {
    CreateMarketItem {
        token_program_id: Pubkey, // program id,
        mint_address: Pubkey,     // ATA
        price: u128,
        file_name: String,
        description: String,
        cash_back: u8,
    },
    PurchaseSale {
        token_program_id: Pubkey, // program id
        price: u128,
        item_id: u128,
    },
    Gacha {
        token_program_id: Pubkey,
        qty: u8,
        price: u128,
        fee: u128,
    },
    CreateGacha {
        token_program_id: Pubkey,
        qty: u8,
    },
    InitState {
        listing_price: u128
    },
    // FetchMarketItems {},
    // FetchMyNFTs {},
    // FetchItemsCreated {}
}

#[derive(BorshDeserialize, Debug)]
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

impl GachaMarketplaceInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let payload = GachaMarketplacePayload::try_from_slice(input).unwrap();
        // Match the variant to determine which data struct is expected by
        Ok(match payload.variant {
            0 => Self::CreateMarketItem {
                token_program_id: payload.token_program_id,
                mint_address: payload.mint_address,
                price: payload.price,
                file_name: payload.file_name,
                description: payload.description,
                cash_back: payload.cash_back,
            },
            1 => Self::PurchaseSale {
                token_program_id: payload.token_program_id,
                price: payload.price,
                item_id: payload.item_id,
            },
            2 => Self::CreateGacha {
                token_program_id: payload.token_program_id,
                qty: payload.qty,
            },
            3 => Self::Gacha {
                token_program_id: payload.token_program_id,
                qty: payload.qty,
                price: payload.price,
                fee: payload.fee,
            },
            4 => Self::InitState{
                listing_price: payload.listing_price
            },
            // 5 => Self::FetchMarketItems {},
            // 6 => Self::FetchMyNFTs {},
            // 7 => Self::FetchItemsCreated {},
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}