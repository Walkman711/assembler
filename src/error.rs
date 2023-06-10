use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyErr {
    #[error("Parse Error: {0}")]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse mnemonic from initial token: {0}")]
    BadMnemonic(String),
    #[error("Failed to parse register {0}")]
    BadRegister(String),
}
