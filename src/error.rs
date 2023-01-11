use thiserror::Error;
use solana_program::{
    msg,
    program_error::{ProgramError, PrintProgramError},
    decode_error::DecodeError,
};
#[derive(Error, Debug, Copy, Clone)]
pub enum GachaError {
    #[error("Price must be at least 1 lamports")]
    InvalidPrice,
    #[error("Cash back should lower than 1")]
    CashbackMax,
    #[error("Please submit the asking price in order to complete the purchase")]
    InvalidPayment,
    #[error("Invalid account")]
    InvalidStateAccount,
    #[error("Account already has entry in Map")]
    AccountAlreadyHasEntry,
}
impl From<GachaError> for ProgramError {
    fn from(e: GachaError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for GachaError {
    fn type_of() -> &'static str {
        "Marketplace Error"
    }
}

impl PrintProgramError for GachaError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            GachaError::InvalidPrice => msg!("Error: Price must be at least 1 lamports"),
            GachaError::CashbackMax => msg!("Error: Cash back should lower than 1"),
            GachaError::InvalidPayment => msg!("Error: Please submit the asking price in order to complete the purchase"),
            GachaError::InvalidStateAccount => msg!("Error: Invalid account"),
            GachaError::AccountAlreadyHasEntry => msg!("Error: Account already has entry in Map")
        }
    }
}