#![allow(dead_code)]

use crate::{
    cond::Cond,
    error::{MyErr, ParseError},
    mnemonics::{DataMnemonic, MemoryMnemonic, Mnemonic},
};

#[derive(Clone, Copy, Debug)]
pub enum DataType {
    SignedWord,
    UnsignedWord,
    SignedHalfWord,
    UnsignedHalfWord,
    SignedByte,
    UnsignedByte,
}

#[derive(Clone, Copy, Debug)]
pub enum Operand {
    Register,
    Immediate(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum Register {
    W(u8),
    X(u8),
}

impl Register {
    pub fn get_id(&self) -> u8 {
        match self {
            Register::W(n) => *n,
            Register::X(n) => *n,
        }
    }
}

impl TryFrom<&str> for Register {
    type Error = MyErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            panic!("empty string");
        }

        if value == "sp" {
            return Ok(Register::X(29));
        } else if value == "pc" {
            return Ok(Register::X(31));
        }

        let access_type = &value[0..1];
        let Ok(register_id) = &value[1..].parse::<u8>() else {
            return Err(ParseError::BadRegister(value.to_owned()).into());
        };

        if *register_id > 31 {
            panic!("bad reg id {register_id}");
        }

        match access_type {
            "x" | "X" => Ok(Self::X(*register_id)),
            "w" | "W" => Ok(Self::W(*register_id)),
            _ => panic!("bad register access_type {access_type:?}"),
        }
    }
}

// TODO: u4
#[derive(Clone, Copy, Debug)]
pub struct Rotation(u8);

// TODO: u4
#[derive(Clone, Copy, Debug)]
pub struct Shift(u8);

#[derive(Clone, Copy, Debug)]
pub enum FlexibleOperand {
    RegisterWithShift(Register, Shift),
    ImmediateWithRotation(u8, Rotation),
}

#[derive(Clone, Copy, Debug)]
pub enum SetConditionCodes {
    SetCodes,
    DontSetCodes,
}

#[derive(Clone, Copy, Debug)]
pub enum Offset {
    RegisterWithShift(Register, Shift, UpDown),
    // TODO: u12
    Immediate(u16, UpDown),
}

#[derive(Clone, Copy, Debug)]
pub enum UpDown {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub enum IndexMode {
    PostIndex,
    Offset,
    PreIndex,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    // TODO: src vs dest register?
    DataProcessing(
        Cond,
        DataMnemonic,
        SetConditionCodes,
        Register,
        Register,
        FlexibleOperand,
    ),
    Mem(Cond, MemoryMnemonic, IndexMode, Register, Register, Offset),
    Branch(Cond, Operand, u64),
}

impl From<&str> for Instruction {
    fn from(value: &str) -> Self {
        dbg!(value);
        let (opcode_cond, rest) = value.trim().split_once(' ').unwrap();

        dbg!(opcode_cond, rest);
        let mnemonic = DataMnemonic::try_from(opcode_cond).unwrap();
        dbg!(mnemonic);
        let cond_maybe = opcode_cond.replace(&mnemonic.to_string(), "");
        let cond = if cond_maybe.is_empty() {
            Cond::AL
        } else {
            Cond::try_from(cond_maybe.as_str()).unwrap()
        };
        dbg!(cond);

        let mut operands = rest.split(",");
        // dbg!(&operands);
        // assert!(operands.len() >= 2);
        // let digit_re = regex::Regex::new(r"\d+").unwrap();
        // let reg1 = digit_re.find(operands.next().unwrap()).map(|x| x.as_str());
        // let reg2 = digit_re.find(operands.next().unwrap()).map(|x| x.as_str());
        let reg1 = Register::try_from(operands.next().unwrap().trim()).unwrap();
        dbg!(reg1);
        let reg2 = Register::try_from(operands.next().unwrap().trim()).unwrap();
        dbg!(reg2);
        // TODO: proper flex op vs 3rd reg handling
        let last = operands.next().unwrap_or("0").trim();
        dbg!(last);
        let flexible_op: u8 = last.parse().unwrap();
        dbg!(flexible_op);

        Self::DataProcessing(
            cond,
            mnemonic,
            SetConditionCodes::SetCodes,
            reg1,
            reg2,
            FlexibleOperand::ImmediateWithRotation(flexible_op, Rotation(0)),
        )
    }
}

impl Instruction {
    pub fn to_machine_code(self) -> u32 {
        match self {
            Instruction::DataProcessing(cond, opcode, set_condition_codes, rn, rd, op2) => {
                let mut encoding: u32 = 0x0;

                let cond_mask: u32 = (cond as u8 as u32) << 28;
                encoding |= cond_mask;

                let i_mask: u32 = match op2 {
                    FlexibleOperand::RegisterWithShift(_, _) => 0,
                    FlexibleOperand::ImmediateWithRotation(_, _) => 1 << 25,
                };
                encoding |= i_mask;

                let opcode_mask: u32 = (opcode as u8 as u32) << 21;
                encoding |= opcode_mask;

                let s_mask: u32 = match set_condition_codes {
                    SetConditionCodes::SetCodes => 1 << 20,
                    SetConditionCodes::DontSetCodes => 0,
                };
                encoding |= s_mask;

                let rn_mask: u32 = (rn.get_id() as u32) << 16;
                encoding |= rn_mask;

                let rd_mask: u32 = (rd.get_id() as u32) << 12;
                encoding |= rd_mask;

                let op2_mask: u32 = match op2 {
                    FlexibleOperand::RegisterWithShift(reg, shift) => {
                        ((shift.0 as u32) << 4) | (reg.get_id() as u32)
                    }
                    FlexibleOperand::ImmediateWithRotation(imm, rotation) => {
                        ((rotation.0 as u32) << 8) | (imm as u32)
                    }
                };
                encoding |= op2_mask;

                encoding
            }
            Instruction::Mem(cond, opcode, index_mode, rn, rd, offset) => {
                let mut encoding: u32 = 0;

                let cond_mask: u32 = (cond as u8 as u32) << 28;
                encoding |= cond_mask;

                // not sure what to call this?
                let random_bit_mask: u32 = 1 << 26;
                encoding |= random_bit_mask;

                let i_mask: u32 = match offset {
                    Offset::RegisterWithShift(..) => 1 << 25,
                    Offset::Immediate(..) => 0,
                };
                encoding |= i_mask;

                let p_mask: u32 = match index_mode {
                    IndexMode::PostIndex => 0,
                    IndexMode::Offset | IndexMode::PreIndex => 1 << 24,
                };
                encoding |= p_mask;

                let u_mask: u32 = match offset {
                    Offset::RegisterWithShift(_, _, updown) | Offset::Immediate(_, updown) => {
                        match updown {
                            UpDown::Up => 1 << 23,
                            UpDown::Down => 0,
                        }
                    }
                };
                encoding |= u_mask;

                let b_mask: u32 = match opcode {
                    MemoryMnemonic::STR | MemoryMnemonic::LDR => 0,
                    MemoryMnemonic::STRB | MemoryMnemonic::LDRB => 1 << 22,
                };
                encoding |= b_mask;

                let w_mask: u32 = match index_mode {
                    IndexMode::PostIndex | IndexMode::Offset => 0,
                    IndexMode::PreIndex => 1 << 21,
                };
                encoding |= w_mask;

                let l_mask: u32 = match opcode {
                    MemoryMnemonic::STR | MemoryMnemonic::STRB => 0,
                    MemoryMnemonic::LDR | MemoryMnemonic::LDRB => 1 << 20,
                };
                encoding |= l_mask;

                let rn_mask: u32 = (rn.get_id() as u32) << 16;
                encoding |= rn_mask;

                let rd_mask: u32 = (rd.get_id() as u32) << 12;
                encoding |= rd_mask;

                let offset_mask: u32 = match offset {
                    Offset::RegisterWithShift(reg, shift, _) => {
                        ((shift.0 as u32) << 4) | (reg.get_id() as u32)
                    }
                    // Truncate immediate value to 12 bits since rust doesn't want you to have
                    // bizarre numeric types like u12 floating around.
                    Offset::Immediate(imm, _) => (imm & 0x0FFF).into(),
                };
                encoding |= offset_mask;

                encoding
            }
            Instruction::Branch(_, _, _) => todo!(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        cond::Cond,
        instructions::{Offset, UpDown},
        mnemonics::{DataMnemonic, MemoryMnemonic},
    };

    use super::{FlexibleOperand, IndexMode, Instruction, Register, SetConditionCodes, Shift};

    #[test]
    fn test_dpi() {
        let mov_inst = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::ADD,
            SetConditionCodes::DontSetCodes,
            Register(1),
            Register(0),
            FlexibleOperand::RegisterWithShift(Register(2), Shift(0)),
        );
        // taken from cs107e on github
        let expected = 0b1110_00_0_0100_0_0001_0000_0000_0000_0010;
        assert_eq!(mov_inst.to_machine_code(), expected);
    }

    #[test]
    fn test_mem() {
        let str_inst = Instruction::Mem(
            Cond::AL,
            MemoryMnemonic::STR,
            IndexMode::Offset,
            Register(0),
            Register(1),
            Offset::Immediate(0, UpDown::Up),
        );
        let machine_code = str_inst.to_machine_code();
        let actual = machine_code.swap_bytes();
        let expected = 0x1080E5;
        assert_eq!(
            actual, expected,
            "actual: {actual:#8X} | expected: {expected:#8X}"
        );
    }
}
