#![allow(dead_code)]

use crate::{cond::Cond, mnemonics::DataMnemonic};

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

// TODO: u4
#[derive(Clone, Copy, Debug)]
pub struct Register(u8);

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
    Mem(Cond, Operand),
    Branch(Cond, Operand, u64),
}

impl Instruction {
    pub fn to_machine_code(self) -> u32 {
        match self {
            Instruction::DataProcessing(cond, mnemonic, set_condition_codes, rn, rd, op2) => {
                let mut encoding: u32 = 0x0;

                let cond_mask: u32 = (cond as u8 as u32) << 28;
                encoding |= cond_mask;

                let imm_operand_mask: u32 = match op2 {
                    FlexibleOperand::RegisterWithShift(_, _) => 0,
                    FlexibleOperand::ImmediateWithRotation(_, _) => 1 << 25,
                };
                encoding |= imm_operand_mask;

                let opcode_mask: u32 = (mnemonic as u8 as u32) << 21;
                encoding |= opcode_mask;

                let s_mask: u32 = match set_condition_codes {
                    SetConditionCodes::SetCodes => 1 << 20,
                    SetConditionCodes::DontSetCodes => 0,
                };
                encoding |= s_mask;

                let rn_mask: u32 = (rn.0 as u32) << 16;
                encoding |= rn_mask;

                let rd_mask: u32 = (rd.0 as u32) << 12;
                encoding |= rd_mask;

                let op2_mask: u32 = match op2 {
                    FlexibleOperand::RegisterWithShift(reg, shift) => {
                        ((shift.0 as u32) << 4) | (reg.0 as u32)
                    }
                    FlexibleOperand::ImmediateWithRotation(imm, rotation) => {
                        ((rotation.0 as u32) << 8) | (imm as u32)
                    }
                };
                encoding |= op2_mask;

                encoding
            }
            Instruction::Mem(_, _) => todo!(),
            Instruction::Branch(_, _, _) => todo!(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{cond::Cond, mnemonics::DataMnemonic};

    use super::{FlexibleOperand, Instruction, Register, SetConditionCodes, Shift};

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
}
