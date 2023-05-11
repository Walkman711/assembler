#[derive(Clone, Copy, Debug)]
pub enum Mnemonic {
    DataMnemonic,
    MemoryMnemonic,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum BranchMnemonic {
    B,
    BL,
}
