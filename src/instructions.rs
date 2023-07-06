use crate::{
    cond::Cond,
    error::{AssemblerError, ParseError},
    mnemonics::{BranchMnemonic, DataMnemonic, MemoryMnemonic, Mnemonic, MultiplyMnemonic},
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
pub struct Rd(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rn(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rs(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rm(pub u8);

fn parse_reg_id(value: &str) -> Result<u8, AssemblerError> {
    if value.is_empty() {
        panic!("empty string");
    }

    if value.contains("fp") {
        return Ok(11);
    } else if value.contains("ip") {
        return Ok(12);
    } else if value.contains("sp") {
        return Ok(13);
    } else if value.contains("lr") {
        return Ok(14);
    } else if value.contains("pc") {
        return Ok(15);
    }

    if !["r", "R"].contains(&(&value[0..1])) {
        return Err(ParseError::BadRegister(value.to_owned()).into());
    }

    if let Ok(register_id) = &value[1..].parse::<u8>() {
        Ok(*register_id)
    } else {
        Err(ParseError::BadRegister(value.to_owned()).into())
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
    RegisterWithShift(u8, Shift),
    ImmediateWithRotation(u8, Rotation),
}

impl TryFrom<&str> for FlexibleOperand {
    type Error = AssemblerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(reg_id) = parse_reg_id(value) {
            return Ok(FlexibleOperand::RegisterWithShift(reg_id, Shift(0)));
        } else if value.contains('#') {
            Ok(FlexibleOperand::ImmediateWithRotation(
                value.replace('#', "").parse::<u8>()?,
                Rotation(0),
            ))
        } else {
            Err(AssemblerError::Parse(ParseError::BadFlexOperand(
                value.to_string(),
            )))
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
    RegisterWithShift(u8, Shift, UpDown),
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
        Rd,
        Rn,
        FlexibleOperand,
    ),
    Mem(Cond, MemoryMnemonic, IndexMode, Rn, Rd, Offset),
    Branch(Cond, BranchMnemonic, u32),
    BranchExec(Cond, Rn),
    Mul(Cond, MultiplyMnemonic, SetConditionCodes, Rd, Rn, Rs, Rm),
}

impl TryFrom<&str> for Instruction {
    type Error = AssemblerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        dbg!(value);
        let (opcode_cond, rest) = value
            .trim()
            .split_once(' ')
            .ok_or(ParseError::RanOutOfOperands)?;
        let mut operands = rest.split(",");
        let mnemonic = Mnemonic::try_from(opcode_cond)?;
        let cond_maybe = opcode_cond.replace(&mnemonic.to_string(), "");
        let cond = if cond_maybe.is_empty() {
            Cond::AL
        } else {
            Cond::try_from(cond_maybe.as_str())?
        };

        let mut get_next_op = || -> Result<&str, Self::Error> {
            Ok(operands
                .next()
                .ok_or(AssemblerError::Parse(ParseError::RanOutOfOperands))?
                .trim())
        };

        let mut get_reg_id = || -> Result<u8, Self::Error> { parse_reg_id(get_next_op()?) };

        match mnemonic {
            Mnemonic::Data(data_mnemonic) => {
                let rd = Rd(get_reg_id()?);

                let (rn, flex_op) = if data_mnemonic == DataMnemonic::MOV {
                    (Rn(0), FlexibleOperand::try_from(get_next_op()?)?)
                } else {
                    (
                        Rn(get_reg_id()?),
                        FlexibleOperand::try_from(get_next_op()?)?,
                    )
                };

                Ok(Self::DataProcessing(
                    cond,
                    data_mnemonic,
                    SetConditionCodes::DontSetCodes,
                    rd,
                    rn,
                    flex_op,
                ))
            }
            Mnemonic::Mem(mem_mnemonic) => {
                let rd = Rd(get_reg_id()?);
                let rn = Rn(get_reg_id()?);

                let mut offset = 0;
                while let Some(maybe_offset) = operands.next() {
                    // Should do something more robust, but this will do for handling brackets for now.
                    // Should use something like the compiler lexer where i expand out special syms
                    if let Ok(parsed_offset) = maybe_offset.replace(']', "").trim().parse::<u16>() {
                        offset = parsed_offset;
                    }
                }

                Ok(Self::Mem(
                    cond,
                    mem_mnemonic,
                    IndexMode::Offset,
                    rn,
                    rd,
                    Offset::Immediate(offset, UpDown::Up),
                ))
            }
            Mnemonic::Mul(mul_mnemonic) => {
                let rd = Rd(get_reg_id()?);
                let reg_2_id = get_reg_id()?;
                let reg_3_id = get_reg_id()?;
                let (rn, rm, rs) = if let Ok(reg_4_id) = get_reg_id() {
                    let rn = Rn(reg_2_id);
                    let rm = Rm(reg_3_id);
                    let rs = Rs(reg_4_id);
                    (rn, rm, rs)
                } else {
                    let rn = Rn(0);
                    let rm = Rm(reg_2_id);
                    let rs = Rs(reg_3_id);
                    (rn, rm, rs)
                };

                Ok(Self::Mul(
                    cond,
                    mul_mnemonic,
                    SetConditionCodes::DontSetCodes,
                    rd,
                    rn,
                    rs,
                    rm,
                ))
            }
            Mnemonic::Branch(b_mnemonic) => {
                let offset = (get_next_op()?).parse::<u32>()?;
                Ok(Self::Branch(cond, b_mnemonic, offset))
            }
            Mnemonic::BranchExec(_bx_mnemonic) => {
                let rn = Rn(get_reg_id()?);
                Ok(Self::BranchExec(cond, rn))
            }
        }
    }
}

impl Instruction {
    pub fn to_machine_code(self) -> u32 {
        let encoding = match self {
            Instruction::DataProcessing(cond, dp_mnemonic, set_condition_codes, rd, rn, op2) => {
                Self::encode_dp_inst(cond, dp_mnemonic, set_condition_codes, rd, rn, op2)
            }
            Instruction::Mem(cond, opcode, index_mode, rn, rd, offset) => {
                Self::encode_mem_inst(cond, opcode, index_mode, rn, rd, offset)
            }
            Instruction::Mul(cond, mul_mnemonic, set_condition_codes, rd, rn, rs, rm) => {
                Self::encode_mul_inst(cond, mul_mnemonic, set_condition_codes, rd, rn, rs, rm)
            }
            Instruction::Branch(cond, b_mnemonic, offset) => {
                Self::encode_branch_inst(cond, b_mnemonic, offset)
            }
            Instruction::BranchExec(cond, rn) => Self::encode_branch_exec_inst(cond, rn),
        };

        encoding.swap_bytes()
    }

    fn encode_dp_inst(
        cond: Cond,
        dp_mnemonic: DataMnemonic,
        set_condition_codes: SetConditionCodes,
        rd: Rd,
        rn: Rn,
        op2: FlexibleOperand,
    ) -> u32 {
        let mut encoding: u32 = 0;

        let cond_mask = (cond as u8 as u32) << 28;
        encoding |= cond_mask;

        let i_mask = match op2 {
            FlexibleOperand::RegisterWithShift(_, _) => 0,
            FlexibleOperand::ImmediateWithRotation(_, _) => 1 << 25,
        };
        encoding |= i_mask;

        let dp_mnemonic_mask = (dp_mnemonic as u8 as u32) << 21;
        encoding |= dp_mnemonic_mask;

        let s_mask = match set_condition_codes {
            SetConditionCodes::SetCodes => 1 << 20,
            SetConditionCodes::DontSetCodes => 0,
        };
        encoding |= s_mask;

        let rn_mask = (rn.0 as u32) << 16;
        encoding |= rn_mask;

        let rd_mask = (rd.0 as u32) << 12;
        encoding |= rd_mask;

        let op2_mask = match op2 {
            FlexibleOperand::RegisterWithShift(reg, shift) => {
                ((shift.0 as u32) << 4) | (reg as u32)
            }
            FlexibleOperand::ImmediateWithRotation(imm, rotation) => {
                ((rotation.0 as u32) << 8) | (imm as u32)
            }
        };
        encoding |= op2_mask;

        encoding
    }

    fn encode_mem_inst(
        cond: Cond,
        mem_mnemonic: MemoryMnemonic,
        index_mode: IndexMode,
        rn: Rn,
        rd: Rd,
        offset: Offset,
    ) -> u32 {
        let mut encoding: u32 = 0;

        let cond_mask = (cond as u8 as u32) << 28;
        encoding |= cond_mask;

        // not sure what to call this?
        let random_bit_mask = 1 << 26;
        encoding |= random_bit_mask;

        let i_mask = match offset {
            Offset::RegisterWithShift(..) => 1 << 25,
            Offset::Immediate(..) => 0,
        };
        encoding |= i_mask;

        let p_mask = match index_mode {
            IndexMode::PostIndex => 0,
            IndexMode::Offset | IndexMode::PreIndex => 1 << 24,
        };
        encoding |= p_mask;

        let u_mask = match offset {
            Offset::RegisterWithShift(_, _, updown) | Offset::Immediate(_, updown) => {
                match updown {
                    UpDown::Up => 1 << 23,
                    UpDown::Down => 0,
                }
            }
        };
        encoding |= u_mask;

        let b_mask = match mem_mnemonic {
            MemoryMnemonic::STR | MemoryMnemonic::LDR => 0,
            MemoryMnemonic::STRB | MemoryMnemonic::LDRB => 1 << 22,
        };
        encoding |= b_mask;

        let w_mask = match index_mode {
            IndexMode::PostIndex | IndexMode::Offset => 0,
            IndexMode::PreIndex => 1 << 21,
        };
        encoding |= w_mask;

        let l_mask = match mem_mnemonic {
            MemoryMnemonic::STR | MemoryMnemonic::STRB => 0,
            MemoryMnemonic::LDR | MemoryMnemonic::LDRB => 1 << 20,
        };
        encoding |= l_mask;

        let rn_mask = (rn.0 as u32) << 16;
        encoding |= rn_mask;

        let rd_mask = (rd.0 as u32) << 12;
        encoding |= rd_mask;

        let offset_mask = match offset {
            Offset::RegisterWithShift(reg, shift, _) => ((shift.0 as u32) << 4) | (reg as u32),
            // Truncate immediate value to 12 bits
            Offset::Immediate(imm, _) => (imm & 0x0FFF).into(),
        };
        encoding |= offset_mask;

        encoding
    }

    fn encode_mul_inst(
        cond: Cond,
        mul_mnemonic: MultiplyMnemonic,
        set_condition_codes: SetConditionCodes,
        rd: Rd,
        rn: Rn,
        rs: Rs,
        rm: Rm,
    ) -> u32 {
        let mut encoding: u32 = 0;

        let cond_mask = (cond as u8 as u32) << 28;
        encoding |= cond_mask;

        let a_mask = match mul_mnemonic {
            MultiplyMnemonic::MUL => 0,
        };
        encoding |= a_mask;

        let s_mask = match set_condition_codes {
            SetConditionCodes::SetCodes => 1 << 20,
            SetConditionCodes::DontSetCodes => 0,
        };
        encoding |= s_mask;

        let rd_mask = (rd.0 as u32) << 16;
        encoding |= rd_mask;

        let rn_mask = (rn.0 as u32) << 12;
        encoding |= rn_mask;

        let rs_mask = (rs.0 as u32) << 8;
        encoding |= rs_mask;

        let magic_bits = (0b1001 as u32) << 4;
        encoding |= magic_bits;

        let rm_mask = rm.0 as u32;
        encoding |= rm_mask;

        encoding
    }

    fn encode_branch_inst(cond: Cond, b_mnemonic: BranchMnemonic, offset: u32) -> u32 {
        let mut encoding: u32 = 0;

        let cond_mask = (cond as u8 as u32) << 28;
        encoding |= cond_mask;

        let magic_bits = (0b101 as u32) << 25;
        encoding |= magic_bits;

        let link_mask = match b_mnemonic {
            BranchMnemonic::B => 0,
            BranchMnemonic::BL => 1 << 24,
        };
        encoding |= link_mask;

        // Take lower 24 bits of offset
        let offset_mask = offset & 0x00_FF_FF_FF;
        encoding |= offset_mask;

        encoding
    }

    fn encode_branch_exec_inst(cond: Cond, rn: Rn) -> u32 {
        let mut encoding: u32 = 0;

        let cond_mask = (cond as u8 as u32) << 28;
        encoding |= cond_mask;

        let magic_bits = 0b0001_0010_1111_1111_1111_0001 << 4;
        encoding |= magic_bits;

        encoding |= rn.0 as u32;

        encoding
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        cond::Cond,
        instructions::{Offset, Rd, Rn, Rotation, UpDown},
        mnemonics::{DataMnemonic, MemoryMnemonic, MultiplyMnemonic},
    };

    use super::{FlexibleOperand, IndexMode, Instruction, Rm, Rs, SetConditionCodes, Shift};

    #[test]
    fn test_add() {
        let add_inst_str = "add r4, r3, r5";
        let add_inst_expected = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::ADD,
            SetConditionCodes::DontSetCodes,
            Rd(4),
            Rn(3),
            FlexibleOperand::RegisterWithShift(5, Shift(0)),
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
            Rd(1),
            Rn(0),
            FlexibleOperand::RegisterWithShift(2, Shift(0)),
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
        let mov_inst_str = "mov r0, #1";
        let mov_inst_expected = Instruction::DataProcessing(
            Cond::AL,
            DataMnemonic::MOV,
            SetConditionCodes::DontSetCodes,
            Rd(0),
            Rn(0),
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
            Rn(13),
            Rd(0),
            Offset::Immediate(8, UpDown::Up),
        );

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

    #[test]
    fn test_mul() {
        let mul_inst_str = "mul r0, r1, r2";
        let mul_inst_expected = Instruction::Mul(
            Cond::AL,
            MultiplyMnemonic::MUL,
            SetConditionCodes::DontSetCodes,
            Rd(0),
            Rn(0),
            Rs(2),
            Rm(1),
        );

        assert_eq!(
            Instruction::try_from(mul_inst_str).unwrap(),
            mul_inst_expected,
        );

        let encoding = mul_inst_expected.to_machine_code();
        let expected = 0x91_02_00_e0;
        assert_eq!(
            encoding, expected,
            "actual: {encoding:#8X} | expected: {expected:#8X}"
        );
    }
}
