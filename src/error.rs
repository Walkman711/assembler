use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssemblerError {
    #[error("Parse Error: {0}")]
    Parse(#[from] ParseError),
    #[error("ParseIntError: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("StrumError: {0}")]
    Strum(#[from] strum::ParseError),
    #[error("IOError: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse mnemonic from initial token: {0}")]
    BadMnemonic(String),
    #[error("Failed to parse register {0}")]
    BadRegister(String),
    #[error("Ran out of operands")]
    RanOutOfOperands,
    #[error("Bad flex operand {0}")]
    BadFlexOperand(String),
}
