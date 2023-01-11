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
    InitState {},
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
    variant: u8,
}

impl GachaMarketplaceInstruction {
    // Unpack inbound buffer to associated Instruction

    // The expected format for input is a Borsh serialized vector

    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // Take the first byte as the variant to

        // determine which instruction to execute

        // let (&variant, rest) = input
        //     .split_first()
        //     .ok_or(ProgramError::InvalidInstructionData)?;
        // Use the temporary payload struct to deserialize

        let payload = GachaMarketplacePayload::try_from_slice(input).unwrap();
        // let payload = try_from_slice_unchecked(input)?;
        // Match the variant to determine which data struct is expected by
        println!("{:?}", payload);
        Ok(Self::InitState {
        })
        // the function and return the TestStruct or an error
        // Ok(match payload.variant {
        //     0 => Self::CreateMarketItem {
        //         nft_contract: payload.nft_contract,
        //         token_id: payload.token_id,
        //         price: payload.price,
        //         file_name: payload.file_name,
        //         description: payload.description,
        //         cash_back: payload.cash_back,
        //     },
        //     1 => Self::PurchaseSale {
        //         nft_contract: payload.nft_contract,
        //         price: payload.price,
        //         item_id: payload.item_id,
        //     },
        //     2 => Self::CreateGacha {
        //         nft_contract: payload.nft_contract,
        //         qty: payload.qty,
        //     },
        //     3 => Self::Gacha {
        //         nft_contract: payload.nft_contract,
        //         qty: payload.qty,
        //         price: payload.price,
        //         fee: payload.fee,
        //     },
        //     4 => Self::InitState{},
        //     _ => return Err(ProgramError::InvalidInstructionData),
        // })
    }
}

#[derive(Debug, Clone, BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct InitStateArgs {
    pub admin_authority: Pubkey,
}

pub fn init_state(args: &InitStateArgs) -> Instruction {
    let accounts = vec![AccountMeta::new(args.admin_authority, true)];

    let instruction = GachaMarketplaceInstruction::InitState {};

    Instruction {
        program_id: crate::ID,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

#[derive(Debug, Clone, BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct CreateMarketItemArgs {
    pub user_authority: Pubkey,
    pub state_account: Pubkey,
    pub token_program: Pubkey,

    pub nft_contract: Pubkey, // program id,
    pub token_id: Pubkey,     // ATA
    pub price: u128,
    pub file_name: String,
    pub description: String,
    pub cash_back: u8,
}

pub fn create_market_item(args: &CreateMarketItemArgs) -> Instruction {
    println!("mmmmmmmmmmmmmmmm");
    let accounts = vec![
        AccountMeta::new(args.user_authority, true),
        AccountMeta::new(args.state_account, false),
        AccountMeta::new(args.token_program, false),
    ];

    let instruction = GachaMarketplaceInstruction::CreateMarketItem {
        nft_contract: args.nft_contract, // program id,
        token_id: args.token_id,         // ATA
        price: args.price,
        file_name: args.file_name.clone(),
        description: args.description.clone(),
        cash_back: args.cash_back,
    };

    Instruction {
        program_id: crate::ID,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

#[derive(Debug, Clone, BorshSchema, BorshSerialize, BorshDeserialize)]
pub struct PurchaseSaleArgs {
    pub user_authority: Pubkey,
    pub state_account: Pubkey,
    pub token_program: Pubkey,

    pub _nft_contract: Pubkey,
    pub _item_id: u128,
    pub _price: u128,
}

pub fn purchase_sale(args: &PurchaseSaleArgs) -> Instruction {
    let accounts = vec![
        AccountMeta::new(args.user_authority, true),
        AccountMeta::new(args.state_account, false),
        AccountMeta::new(args.token_program, false),
    ];

    let instruction = GachaMarketplaceInstruction::PurchaseSale {
        nft_contract: args._nft_contract,
        price: args._price,
        item_id: args._item_id,
    };

    Instruction {
        program_id: crate::ID,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

// pub struct GachaArgs {
//     pub user_authority: Pubkey,
//     pub state_account: Pubkey,
//     pub token_program: Pubkey,

//     pub _nft_contract: Pubkey,
//     pub _qty: u8,
//     pub _price: u128,
//     pub _fee: u128,
// }

// pub fn gacha(args: &GachaArgs) -> Instruction {
//     let accounts = vec![
//         AccountMeta::new(args.user_authority, true),
//         AccountMeta::new(args.state_account, false),
//         AccountMeta::new(args.token_program, false),
//     ];

//     let instruction = GachaMarketplaceInstruction::Gacha {
//         nft_contract: args._nft_contract,
//         qty: args._qty,
//         price: args._price,
//         fee: args._fee,
//     };

//     Instruction {
//         program_id: crate::ID,
//         accounts,
//         data: instruction.try_to_vec().unwrap(),
//     }
// }

// pub struct CreateGachaArgs {
//     pub user_authority: Pubkey,
//     pub state_account: Pubkey,
//     pub token_program: Pubkey,

//     pub _nft_contract: Pubkey,
//     pub _qty: u8,
// }

// pub fn create_gacha(args: &GachaArgs) -> Instruction {
//     let accounts = vec![
//         AccountMeta::new(args.user_authority, true),
//         AccountMeta::new(args.state_account, false),
//         AccountMeta::new(args.token_program, false),
//     ];

//     let instruction = GachaMarketplaceInstruction::CreateGacha {
//         nft_contract: args._nft_contract,
//         qty: args._qty,
//     };

//     Instruction {
//         program_id: crate::ID,
//         accounts,
//         data: instruction.try_to_vec().unwrap(),
//     }
// }
