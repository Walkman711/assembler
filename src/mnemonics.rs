use strum::IntoEnumIterator;

use crate::error::{MyErr, ParseError};

#[derive(Clone, Copy, Debug)]
pub enum Mnemonic {
    Data(DataMnemonic),
    Mem(MemoryMnemonic),
}

#[derive(Clone, Copy, Debug, strum_macros::EnumIter)]
pub enum DataMnemonic {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

impl std::fmt::Display for DataMnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataMnemonic::AND => write!(f, "and"),
            DataMnemonic::EOR => write!(f, "eor"),
            DataMnemonic::SUB => write!(f, "sub"),
            DataMnemonic::RSB => write!(f, "rsb"),
            DataMnemonic::ADD => write!(f, "add"),
            DataMnemonic::ADC => write!(f, "adc"),
            DataMnemonic::SBC => write!(f, "sbc"),
            DataMnemonic::RSC => write!(f, "rsc"),
            DataMnemonic::TST => write!(f, "tst"),
            DataMnemonic::TEQ => write!(f, "teq"),
            DataMnemonic::CMP => write!(f, "cmp"),
            DataMnemonic::CMN => write!(f, "cmn"),
            DataMnemonic::ORR => write!(f, "orr"),
            DataMnemonic::MOV => write!(f, "mov"),
            DataMnemonic::BIC => write!(f, "bic"),
            DataMnemonic::MVN => write!(f, "mvn"),
        }
    }
}

impl TryFrom<&str> for DataMnemonic {
    type Error = MyErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for mnemonic in DataMnemonic::iter() {
            let mnemonic_upper = mnemonic.to_string().to_uppercase();
            let mnemonic_lower = mnemonic.to_string().to_lowercase();
            if value.starts_with(&mnemonic_upper) || value.starts_with(&mnemonic_lower) {
                return Ok(mnemonic);
            }
        }

        Err(ParseError::BadMnemonic(value.to_owned()).into())
    }
}

impl From<DataMnemonic> for u8 {
    fn from(value: DataMnemonic) -> Self {
        match value {
            DataMnemonic::AND => 0b0000,
            DataMnemonic::EOR => 0b0001,
            DataMnemonic::SUB => 0b0010,
            DataMnemonic::RSB => 0b0011,
            DataMnemonic::ADD => 0b0100,
            DataMnemonic::ADC => 0b0101,
            DataMnemonic::SBC => 0b0110,
            DataMnemonic::RSC => 0b0111,
            DataMnemonic::TST => 0b1000,
            DataMnemonic::TEQ => 0b1001,
            DataMnemonic::CMP => 0b1010,
            DataMnemonic::CMN => 0b1011,
            DataMnemonic::ORR => 0b1100,
            DataMnemonic::MOV => 0b1101,
            DataMnemonic::BIC => 0b1110,
            DataMnemonic::MVN => 0b1111,
        }
    }
}

// TODO: L, B bits
#[derive(Clone, Copy, Debug)]
pub enum MemoryMnemonic {
    STR,
    STRB,
    LDR,
    LDRB,
}

impl std::fmt::Display for MemoryMnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryMnemonic::STR => write!(f, "str"),
            MemoryMnemonic::STRB => write!(f, "strb"),
            MemoryMnemonic::LDR => write!(f, "ldr"),
            MemoryMnemonic::LDRB => write!(f, "ldrb"),
        }
    }
}

impl TryFrom<&str> for MemoryMnemonic {
    type Error = MyErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: there should be a better way of doing this that doesn't rely on the correct
        // ordering of the MemoryMnemonic enum variants. Probably just store the length of the
        // matching str, but I don't want to do that rn.
        let strb = MemoryMnemonic::STRB.to_string();
        let str = MemoryMnemonic::STR.to_string();
        let ldrb = MemoryMnemonic::LDRB.to_string();
        let ldr = MemoryMnemonic::LDR.to_string();

        if value.starts_with(&strb.to_uppercase()) || value.starts_with(&strb.to_lowercase()) {
            Ok(MemoryMnemonic::STRB)
        } else if value.starts_with(&str.to_uppercase()) || value.starts_with(&str.to_lowercase()) {
            Ok(MemoryMnemonic::STR)
        } else if value.starts_with(&ldrb.to_uppercase()) || value.starts_with(&ldrb.to_lowercase())
        {
            Ok(MemoryMnemonic::LDRB)
        } else if value.starts_with(&ldr.to_uppercase()) || value.starts_with(&ldr.to_lowercase()) {
            Ok(MemoryMnemonic::LDR)
        } else {
            Err(ParseError::BadMnemonic(value.to_owned()).into())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BranchMnemonic {
    B,
    BL,
}

impl std::fmt::Display for BranchMnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchMnemonic::B => write!(f, "b"),
            BranchMnemonic::BL => write!(f, "bl"),
        }
    }
}

impl TryFrom<&str> for BranchMnemonic {
    type Error = MyErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bl = BranchMnemonic::BL.to_string();
        let b = BranchMnemonic::B.to_string();

        if value.starts_with(&bl.to_uppercase()) || value.starts_with(&bl.to_lowercase()) {
            Ok(BranchMnemonic::BL)
        } else if value.starts_with(&b.to_uppercase()) || value.starts_with(&b.to_lowercase()) {
            Ok(BranchMnemonic::B)
        } else {
            Err(ParseError::BadMnemonic(value.to_owned()).into())
        }
    }
}
