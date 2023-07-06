#[derive(Clone, Copy, Debug, Eq, PartialEq, strum_macros::EnumIter, strum_macros::EnumString)]
pub enum Cond {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
    NV,
}

impl From<Cond> for u8 {
    fn from(value: Cond) -> Self {
        match value {
            Cond::EQ => 0x00,
            Cond::NE => 0x01,
            Cond::CS => 0x02,
            Cond::CC => 0x03,
            Cond::MI => 0x04,
            Cond::PL => 0x05,
            Cond::VS => 0x06,
            Cond::VC => 0x07,
            Cond::HI => 0x08,
            Cond::LS => 0x09,
            Cond::GE => 0x0A,
            Cond::LT => 0x0B,
            Cond::GT => 0x0C,
            Cond::LE => 0x0D,
            Cond::AL => 0x0E,
            Cond::NV => 0x0F,
        }
    }
}
