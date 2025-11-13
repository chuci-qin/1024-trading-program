//! Trading Program Error Definitions

use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum TradingError {
    #[error("Invalid Authority")]
    InvalidAuthority,
    
    #[error("Invalid Vault Account")]
    InvalidVaultAccount,
    
    #[error("Invalid Position Account")]
    InvalidPositionAccount,
    
    #[error("Invalid Protection Pool Account")]
    InvalidProtectionPoolAccount,
    
    #[error("Account Already Exists")]
    AccountAlreadyExists,
    
    #[error("Serialization Error")]
    SerializationError,
    
    #[error("Deserialization Error")]
    DeserializationError,
    
    #[error("Insufficient Balance")]
    InsufficientBalance,
    
    #[error("Invalid Margin")]
    InvalidMargin,
    
    #[error("Invalid Leverage")]
    InvalidLeverage,
    
    #[error("Invalid Price")]
    InvalidPrice,
    
    #[error("Invalid Size")]
    InvalidSize,
    
    #[error("Position Not Found")]
    PositionNotFound,
    
    #[error("Position Not Liquidatable")]
    PositionNotLiquidatable,
    
    #[error("Margin Ratio Too Low")]
    MarginRatioTooLow,
    
    #[error("Arithmetic Overflow")]
    ArithmeticOverflow,
    
    #[error("Arithmetic Underflow")]
    ArithmeticUnderflow,
    
    #[error("Division By Zero")]
    DivisionByZero,
    
    #[error("Invalid Token Transfer")]
    InvalidTokenTransfer,
    
    #[error("Invalid Market")]
    InvalidMarket,
    
    #[error("Invalid Account ID")]
    InvalidAccountId,
    
    #[error("Smart Hedge Not Enabled")]
    SmartHedgeNotEnabled,
    
    #[error("Reentry Already Triggered")]
    ReentryAlreadyTriggered,
    
    #[error("Reentry Not Enabled")]
    ReentryNotEnabled,
    
    #[error("Price Drop Insufficient")]
    PriceDropInsufficient,
    
    #[error("Protection Pool Expired")]
    ProtectionPoolExpired,
    
    #[error("Invalid Close Ratio")]
    InvalidCloseRatio,
    
    #[error("Insurance Fund Insufficient")]
    InsuranceFundInsufficient,
}

impl From<TradingError> for ProgramError {
    fn from(e: TradingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

