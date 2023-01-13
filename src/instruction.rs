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
        nft_contract: Pubkey, // program id,
        token_id: Pubkey,     // ATA
        price: u128,
        file_name: String,
        description: String,
        cash_back: u8,
    },
    PurchaseSale {
        nft_contract: Pubkey, // program id
        price: u128,
        item_id: u128,
    },
    Gacha {
        nft_contract: Pubkey,
        qty: u8,
        price: u128,
        fee: u128,
    },
    CreateGacha {
        nft_contract: Pubkey,
        qty: u8,
    },
    InitState {
        listing_price: u128
    },
}

#[derive(BorshDeserialize, Debug)]
struct GachaMarketplacePayload {
    nft_contract: Pubkey, // program id,
    token_id: Pubkey,     // ATA
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
                nft_contract: payload.nft_contract,
                token_id: payload.token_id,
                price: payload.price,
                file_name: payload.file_name,
                description: payload.description,
                cash_back: payload.cash_back,
            },
            1 => Self::PurchaseSale {
                nft_contract: payload.nft_contract,
                price: payload.price,
                item_id: payload.item_id,
            },
            2 => Self::CreateGacha {
                nft_contract: payload.nft_contract,
                qty: payload.qty,
            },
            3 => Self::Gacha {
                nft_contract: payload.nft_contract,
                qty: payload.qty,
                price: payload.price,
                fee: payload.fee,
            },
            4 => Self::InitState{
                listing_price: payload.listing_price
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}