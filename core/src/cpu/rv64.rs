use crate::cpu::{FReg, Reg};
use crate::masm::CondCode;
use core_asm::rv64 as rv64_asm;
use core_asm::rv64::{Condition, Register};

pub static REG_PARAMS: [Reg; 8] = [R10, R11, R12, R13, R14, R15, R16, R17];
pub static FREG_PARAMS: [FReg; 12] = [F0, F1, F2, F3, F4, F5, F6, F7, F28, F29, F30, F31];

pub static CCALL_REG_PARAMS: [Reg; 8] = [R10, R11, R12, R13, R14, R15, R16, R17];
pub static CCALL_FREG_PARAMS: [FReg; 12] = [F0, F1, F2, F3, F4, F5, F6, F7, F28, F29, F30, F31];

pub static SCRATCH: [Reg; 7] = [R5, R6, R7, R28, R29, R30, R31];

pub const REG_RESULT: Reg = R10;
pub const REG_TMP1: Reg = R5;
pub const REG_TMP2: Reg = R6;
pub const REG_TMP3: Reg = R7;
pub const REG_ZERO: Reg = Reg(0);
pub const REG_RA: Reg = Reg(1);
pub const REG_SP: Reg = Reg(2);
pub const REG_GP: Reg = Reg(3);
pub const REG_THREAD: Reg = Reg(4);
pub const REG_FP: Reg = Reg(8);

pub const FREG_RESULT: FReg = F10;
pub const FREG_TMP1: FReg = F0;

pub const STACK_FRAME_ALIGNMENT: usize = 16;

pub const R0: Reg = Reg(0);
pub const R1: Reg = Reg(1);
pub const R2: Reg = Reg(2);
pub const R3: Reg = Reg(3);
pub const R4: Reg = Reg(4);
pub const R5: Reg = Reg(5);
pub const R6: Reg = Reg(6);
pub const R7: Reg = Reg(7);
#[allow(dead_code)]
pub const R8: Reg = Reg(8);
pub const R9: Reg = Reg(9);
pub const R10: Reg = Reg(10);
pub const R11: Reg = Reg(11);
pub const R12: Reg = Reg(12);
pub const R13: Reg = Reg(13);
pub const R14: Reg = Reg(14);
pub const R15: Reg = Reg(15);
#[allow(dead_code)]
pub const R16: Reg = Reg(16);
#[allow(dead_code)]
pub const R17: Reg = Reg(17);
#[allow(dead_code)]
pub const R18: Reg = Reg(18);
#[allow(dead_code)]
pub const R19: Reg = Reg(19);
#[allow(dead_code)]
pub const R20: Reg = Reg(20);
#[allow(dead_code)]
pub const R21: Reg = Reg(21);
#[allow(dead_code)]
pub const R22: Reg = Reg(22);
#[allow(dead_code)]
pub const R23: Reg = Reg(23);
#[allow(dead_code)]
pub const R24: Reg = Reg(24);
#[allow(dead_code)]
pub const R25: Reg = Reg(25);
#[allow(dead_code)]
pub const R26: Reg = Reg(26);
#[allow(dead_code)]
pub const R27: Reg = Reg(27);
pub const R28: Reg = Reg(28);
pub const R29: Reg = Reg(29);
pub const R30: Reg = Reg(30);
pub const R31: Reg = Reg(31);

pub const F0: FReg = FReg(0);
pub const F1: FReg = FReg(1);
pub const F2: FReg = FReg(2);
pub const F3: FReg = FReg(3);
pub const F4: FReg = FReg(4);
pub const F5: FReg = FReg(5);
pub const F6: FReg = FReg(6);
pub const F7: FReg = FReg(7);
pub const F8: FReg = FReg(8);
pub const F9: FReg = FReg(9);
pub const F10: FReg = FReg(10);
pub const F11: FReg = FReg(11);
pub const F12: FReg = FReg(12);
pub const F13: FReg = FReg(13);
pub const F14: FReg = FReg(14);
pub const F15: FReg = FReg(15);
pub const F16: FReg = FReg(16);
pub const F17: FReg = FReg(17);
pub const F18: FReg = FReg(18);
pub const F19: FReg = FReg(19);
pub const F20: FReg = FReg(20);
pub const F21: FReg = FReg(21);
pub const F22: FReg = FReg(22);
pub const F23: FReg = FReg(23);
pub const F24: FReg = FReg(24);
pub const F25: FReg = FReg(25);
pub const F26: FReg = FReg(26);
pub const F27: FReg = FReg(27);
pub const F28: FReg = FReg(28);
pub const F29: FReg = FReg(29);
pub const F30: FReg = FReg(30);
pub const F31: FReg = FReg(31);

pub fn flush_icache(_: *const u8, _: usize) {}

pub fn has_round() -> bool {
    true
}

pub fn has_popcnt() -> bool {
    false
}

pub fn has_lzcnt() -> bool {
    false
}

pub fn has_tzcnt() -> bool {
    false
}

impl From<Reg> for Register {
    fn from(reg: Reg) -> Register {
        if reg.0 < 31 {
            Register::new(reg.0)
        } else if reg == REG_ZERO {
            rv64_asm::REG_ZERO
        } else {
            assert_eq!(reg, REG_SP);
            rv64_asm::REG_SP
        }
    }
}

pub static PARAM_OFFSET: i32 = 16;

impl From<CondCode> for Condition {
    fn from(c: CondCode) -> Condition {
        match c {
            CondCode::Zero => Condition::EQ,
            CondCode::NonZero => Condition::NE,
            CondCode::Equal => Condition::EQ,
            CondCode::NotEqual => Condition::NE,
            CondCode::Greater => Condition::GT,
            CondCode::GreaterEq => Condition::GE,
            CondCode::Less => Condition::LT,
            CondCode::LessEq => Condition::LE,
            CondCode::UnsignedGreater => Condition::GTU,
            CondCode::UnsignedGreaterEq => Condition::GEU,
            CondCode::UnsignedLess => Condition::LTU,
            CondCode::UnsignedLessEq => Condition::LEU,
        }
    }
}
