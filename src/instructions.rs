#![allow(dead_code)]

use crate::{
    cond::Cond,
    error::{MyErr, ParseError},
    mnemonics::{DataMnemonic, MemoryMnemonic, Mnemonic, MultiplyMnemonic},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataType {
    SignedWord,
    UnsignedWord,
    SignedHalfWord,
    UnsignedHalfWord,
    SignedByte,
    UnsignedByte,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operand {
    Register,
    Immediate(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Register {
    R(u8),
    W(u8),
    X(u8),
}

impl Register {
    pub fn get_id(&self) -> u8 {
        match self {
            Register::R(n) => *n,
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
            return Ok(Register::R(13));
        } else if value == "pc" {
            return Ok(Register::R(15));
        }

        let access_type = &value[0..1];
        let Ok(register_id) = &value[1..].parse::<u8>() else {
            return Err(ParseError::BadRegister(value.to_owned()).into());
        };

        if *register_id > 31 {
            panic!("bad reg id {register_id}");
        }

        match access_type {
            "r" | "R" => Ok(Self::R(*register_id)),
            "x" | "X" => Ok(Self::X(*register_id)),
            "w" | "W" => Ok(Self::W(*register_id)),
            _ => Err(ParseError::BadRegister(value.to_owned()).into()),
        }
    }
}

// TODO: u4
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rotation(u8);

// TODO: u4
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Shift(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlexibleOperand {
    RegisterWithShift(Register, Shift),
    ImmediateWithRotation(u8, Rotation),
}

impl TryFrom<&str> for FlexibleOperand {
    type Error = MyErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(reg) = Register::try_from(value) {
            return Ok(FlexibleOperand::RegisterWithShift(reg, Shift(0)));
        } else {
            Ok(FlexibleOperand::ImmediateWithRotation(
                value.parse::<u8>().unwrap(),
                Rotation(0),
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetConditionCodes {
    SetCodes,
    DontSetCodes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Offset {
    RegisterWithShift(Register, Shift, UpDown),
    // TODO: u12
    Immediate(u16, UpDown),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpDown {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexMode {
    PostIndex,
    Offset,
    PreIndex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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
    Mul(Cond, MultiplyMnemonic, Register, Register, Register),
}

impl From<&str> for Instruction {
    fn from(value: &str) -> Self {
        let (opcode_cond, rest) = value.trim().split_once(' ').unwrap();

        if let Ok(mnemonic) = DataMnemonic::try_from(opcode_cond) {
            let cond_maybe = opcode_cond.replace(&mnemonic.to_string(), "");
            let cond = if cond_maybe.is_empty() {
                Cond::AL
            } else {
                Cond::try_from(cond_maybe.as_str()).unwrap()
            };

            let mut operands = rest.split(",");

            // There will always be a destination register
            let rd = Register::try_from(operands.next().unwrap().trim()).unwrap();

            let (rn, flex_op) = if mnemonic == DataMnemonic::MOV {
                (
                    Register::R(0),
                    FlexibleOperand::try_from(operands.next().unwrap().trim()).unwrap(),
                )
            } else {
                (
                    Register::try_from(operands.next().unwrap().trim()).unwrap(),
                    FlexibleOperand::try_from(operands.next().unwrap().trim()).unwrap(),
                )
            };

            Self::DataProcessing(
                cond,
                mnemonic,
                SetConditionCodes::DontSetCodes,
                rn,
                rd,
                flex_op,
            )
        } else if let Ok(mnemonic) = MemoryMnemonic::try_from(opcode_cond) {
            let cond_maybe = opcode_cond.replace(&mnemonic.to_string(), "");
            let cond = if cond_maybe.is_empty() {
                Cond::AL
            } else {
                Cond::try_from(cond_maybe.as_str()).unwrap()
            };
            let mut operands = rest.split(",");

            // There will always be a destination register
            let rd = Register::try_from(operands.next().unwrap().trim()).unwrap();
            // TODO: can this be anything but the stack pointer?
            let rn = Register::R(13);

            let mut offset = 0;
            while let Some(maybe_offset) = operands.next() {
                // Should do something more robust, but this will do for handling brackets for now.
                // Should use something like the compiler lexer where i expand out special syms
                if let Ok(parsed_offset) = maybe_offset.replace(']', "").trim().parse::<u16>() {
                    offset = parsed_offset;
                }
            }

            Self::Mem(
                cond,
                mnemonic,
                IndexMode::Offset,
                rn,
                rd,
                Offset::Immediate(offset, UpDown::Up),
            )
        } else {
            let mnemonic = MultiplyMnemonic::try_from(opcode_cond).unwrap();
            let cond_maybe = opcode_cond.replace(&mnemonic.to_string(), "");
            let cond = if cond_maybe.is_empty() {
                Cond::AL
            } else {
                Cond::try_from(cond_maybe.as_str()).unwrap()
            };
            let mut operands = rest.split(",");
            let rd = Register::try_from(operands.next().unwrap().trim()).unwrap();
            let rn = Register::try_from(operands.next().unwrap().trim()).unwrap();
            let rs = Register::try_from(operands.next().unwrap().trim()).unwrap();
            Self::Mul(cond, mnemonic, rn, rd, rs)
        }
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

                encoding.swap_bytes()
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

                encoding.swap_bytes()
            }
            Instruction::Mul(..) => todo!(),
            Instruction::Branch(..) => todo!(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        cond::Cond,
        instructions::{Offset, Rotation, UpDown},
        mnemonics::{DataMnemonic, MemoryMnemonic},
    };

    use super::{FlexibleOperand, IndexMode, Instruction, Register, SetConditionCodes, Shift};

    #[test]
    fn test_add() {
        let add_inst_str = "add r4, r3, r5";
        let add_inst_expected = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::ADD,
            SetConditionCodes::DontSetCodes,
            Register::R(3),
            Register::R(4),
            FlexibleOperand::RegisterWithShift(Register::R(5), Shift(0)),
        );

        assert_eq!(
            Instruction::try_from(add_inst_str).unwrap(),
            add_inst_expected,
        );

        let expected_be: u32 = 0x05_40_83_E0;
        let encoding = add_inst_expected.to_machine_code();

        assert_eq!(
            encoding, expected_be,
            "\nactual: {encoding:#08x} | expected: {expected_be:#08x}"
        );
    }

    #[test]
    fn test_sub() {
        let sub_inst_str = "sub r1, r0, r2";
        let sub_inst_expected = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::SUB,
            SetConditionCodes::DontSetCodes,
            Register::R(0),
            Register::R(1),
            FlexibleOperand::RegisterWithShift(Register::R(2), Shift(0)),
        );

        assert_eq!(
            Instruction::try_from(sub_inst_str).unwrap(),
            sub_inst_expected,
        );

        let expected_be: u32 = 0x02_10_40_E0;
        let expected_le = expected_be.to_le();
        let encoding = sub_inst_expected.to_machine_code();

        assert_eq!(
            encoding, expected_le,
            "\nactual: {encoding:#08x} | expected: {expected_le:#08x}"
        );
    }

    #[test]
    fn test_mov() {
        let mov_inst_str = "mov r0, 1";
        let mov_inst_expected = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::MOV,
            SetConditionCodes::DontSetCodes,
            Register::R(0),
            Register::R(0),
            FlexibleOperand::ImmediateWithRotation(1, Rotation(0)),
        );

        assert_eq!(
            Instruction::try_from(mov_inst_str).unwrap(),
            mov_inst_expected,
        );

        let expected_le = 0x01_00_a0_e3;
        let encoding = mov_inst_expected.to_machine_code();

        assert_eq!(
            encoding, expected_le,
            "\nactual: {encoding:#08x} | expected: {expected_le:#08x}"
        );
    }

    #[test]
    fn test_mem() {
        let str_inst_str = "str r0, [sp, 8]";
        let str_inst_expected = Instruction::Mem(
            Cond::AL,
            MemoryMnemonic::STR,
            IndexMode::Offset,
            Register::R(13),
            Register::R(0),
            Offset::Immediate(8, UpDown::Up),
        );

        // TODO: handle parsing mem instructions
        assert_eq!(
            Instruction::try_from(str_inst_str).unwrap(),
            str_inst_expected,
        );

        let encoding = str_inst_expected.to_machine_code();
        let expected = 0x08_00_8d_e5;
        assert_eq!(
            encoding, expected,
            "actual: {encoding:#8X} | expected: {expected:#8X}"
        );
    }
}
