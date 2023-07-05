use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyErr {
    #[error("Parse Error: {0}")]
    Parse(#[from] ParseError),
    #[error("ParseIntError: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("StrumError: {0}")]
    Strum(#[from] strum::ParseError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse mnemonic from initial token: {0}")]
    BadMnemonic(String),
    #[error("Failed to parse register {0}")]
    BadRegister(String),
    #[error("Ran out of operands")]
    RanOutOfOperands,
}
