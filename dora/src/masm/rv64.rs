use dora_parser::lexer::position::Position;

use crate::compiler::codegen::AnyReg;
use crate::cpu::*;
use crate::gc::swiper::CARD_SIZE_BITS;
use crate::gc::Address;
use crate::language::sem_analysis::FctDefinitionId;
use crate::language::ty::SourceTypeArray;
use crate::masm::{CondCode, Label, MacroAssembler, Mem};
use crate::mem::ptr_width;
use crate::mode::MachineMode;
use crate::object::{
    offset_of_array_data, offset_of_array_length, offset_of_string_data, Header, STR_LEN_MASK,
};
use crate::threads::ThreadLocalData;
use crate::vm::{get_vm, LazyCompilationSite, Trap};
use crate::vtable::VTable;
pub use dora_asm::rv64::AssemblerRv64 as Assembler;
use dora_asm::rv64::{self as asm, FloatRegister, RoundingMode};

impl MacroAssembler {
    pub fn prolog(&mut self, stacksize: i32) {
        self.asm.addi(REG_SP.into(), REG_SP.into(), -16);
        self.asm.sd(REG_SP.into(), REG_FP.into(), 0);
        self.asm.sd(REG_SP.into(), REG_RA.into(), 8);
        self.asm.addi(REG_SP.into(), REG_FP.into(), 0);
    }

    pub fn check_stack_pointer(&mut self, lbl_overflow: Label) {}

    pub fn safepoint(&mut self, lbl_safepoint: Label) {}

    pub fn fix_result(&mut self, _result: Reg, _mode: MachineMode) {
        // nothing to do on RiscV, see version for x64 for more info.
    }

    pub fn epilog(&mut self) {
        self.asm.addi(REG_FP.into(), REG_SP.into(), 0);
        self.asm.ld(REG_FP.into(), REG_SP.into(), 0);
        self.asm.ld(REG_RA.into(), REG_SP.into(), 8);
        self.asm.addi(REG_SP.into(), REG_SP.into(), 16);
    }

    pub fn epilog_without_return(&mut self) {}

    pub fn increase_stack_frame(&mut self, size: i32) {}

    pub fn decrease_stack_frame(&mut self, size: i32) {}

    pub fn direct_call(
        &mut self,
        fct_id: FctDefinitionId,
        ptr: Address,
        type_params: SourceTypeArray,
    ) {
    }

    pub fn raw_call(&mut self, ptr: Address) {}

    pub fn virtual_call(
        &mut self,
        pos: Position,
        vtable_index: u32,
        self_index: u32,
        lazy_compilation_site: LazyCompilationSite,
    ) {
    }

    pub fn load_array_elem(&mut self, mode: MachineMode, dest: AnyReg, array: Reg, index: Reg) {}

    pub fn load_string_elem(&mut self, mode: MachineMode, dest: AnyReg, array: Reg, index: Reg) {}

    pub fn set(&mut self, dest: Reg, op: CondCode) {}

    pub fn cmp_mem(&mut self, mode: MachineMode, mem: Mem, rhs: Reg) {}

    pub fn cmp_mem_imm(&mut self, mode: MachineMode, mem: Mem, imm: i32) {}

    pub fn cmp_reg(&mut self, mode: MachineMode, lhs: Reg, rhs: Reg) {}

    pub fn cmp_reg_imm(&mut self, mode: MachineMode, lhs: Reg, imm: i32) {}

    pub fn cmp_zero(&mut self, mode: MachineMode, lhs: Reg) {
        self.asm.sltiu(lhs.into(), lhs.into(), 1)
    }

    pub fn cmp_int(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {}

    pub fn test_and_jump_if(&mut self, cond: CondCode, reg: Reg, lbl: Label) {}

    pub fn jump_if(&mut self, cond: CondCode, target: Label) {}

    pub fn jump(&mut self, target: Label) {}

    pub fn jump_reg(&mut self, reg: Reg) {}

    pub fn int_div(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg, pos: Position) {
        match mode {
            MachineMode::Int64 => self.asm.div(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.divw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_mod(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg, pos: Position) {
        // todo: check semantics
        match mode {
            MachineMode::Int64 => self.asm.rem(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.remw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_mul(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.mul(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.mulw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_mul_checked(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: Reg,
        rhs: Reg,
        pos: Position,
    ) {
        // fixme: implement check
        match mode {
            MachineMode::Int64 => self.asm.mul(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.mulw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_add(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.add(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.addw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_add_checked(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: Reg,
        rhs: Reg,
        pos: Position,
    ) {
        // fixme: implement check
        match mode {
            MachineMode::Int64 => self.asm.add(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.addw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_add_imm(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, value: i64) {
        match mode {
            MachineMode::Int64 => self.asm.addi(dest.into(), lhs.into(), value as i32),
            MachineMode::Int32 => self.asm.addiw(dest.into(), lhs.into(), value as i32),
            _ => unreachable!(),
        }
    }

    pub fn int_sub(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.sub(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.subw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_sub_checked(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: Reg,
        rhs: Reg,
        pos: Position,
    ) {
        // fixme: implement check
        match mode {
            MachineMode::Int64 => self.asm.sub(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.subw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_shl(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.sll(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.sllw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_shr(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.srl(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.srlw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_sar(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.sra(dest.into(), lhs.into(), rhs.into()),
            MachineMode::Int32 => self.asm.sraw(dest.into(), lhs.into(), rhs.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_rol(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            // fixme: only use if bitmanip available
            MachineMode::Int64 | MachineMode::Int32 => {
                self.asm.rol(dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_ror(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            // fixme: only use if bitmanip available
            MachineMode::Int64 | MachineMode::Int32 => {
                self.asm.ror(dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_or(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 | MachineMode::Int32 => {
                self.asm.or(dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_and(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 | MachineMode::Int32 => {
                self.asm.and(dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_xor(&mut self, mode: MachineMode, dest: Reg, lhs: Reg, rhs: Reg) {
        match mode {
            MachineMode::Int64 | MachineMode::Int32 => {
                self.asm.xor(dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_reverse_bits(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        unimplemented!()
    }

    pub fn int_reverse_bytes(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        match mode {
            // fixme: only use if bitmanip available
            MachineMode::Int64 | MachineMode::Int32 => self.asm.rev8_64(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn count_bits(&mut self, mode: MachineMode, dest: Reg, src: Reg, count_one_bits: bool) {
        if count_one_bits {
            self.asm.xori(src.into(), src.into(), -1)
        }
        match mode {
            // fixme: only use if bitmanip available
            MachineMode::Int64 => self.asm.cpop(dest.into(), src.into()),
            // fixme: only use if bitmanip available
            MachineMode::Int32 => self.asm.cpopw(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn count_bits_leading(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        src: Reg,
        count_one_bits: bool,
    ) {
        if count_one_bits {
            self.asm.xori(src.into(), src.into(), -1)
        }
        match mode {
            MachineMode::Int64 => {
                // fixme: only use if bitmanip available
                self.asm.clz(dest.into(), src.into())
            }
            MachineMode::Int32 => {
                // fixme: only use if bitmanip available
                self.asm.clzw(dest.into(), src.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn count_bits_trailing(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        src: Reg,
        count_one_bits: bool,
    ) {
        if count_one_bits {
            self.asm.xori(src.into(), src.into(), -1)
        }
        match mode {
            MachineMode::Int64 => {
                // fixme: only use if bitmanip available
                self.asm.ctz(dest.into(), src.into())
            }
            MachineMode::Int32 => {
                // fixme: only use if bitmanip available
                self.asm.ctzw(dest.into(), src.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn int_to_float(
        &mut self,
        dest_mode: MachineMode,
        dest: FReg,
        src_mode: MachineMode,
        src: Reg,
    ) {
        match (dest_mode, src_mode) {
            (MachineMode::Float64, MachineMode::Int64) => {
                self.asm
                    .fcvt_d_l(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float32, MachineMode::Int64) => {
                self.asm
                    .fcvt_d_w(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float64, MachineMode::Int32) => {
                self.asm
                    .fcvt_s_l(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float32, MachineMode::Int32) => {
                self.asm
                    .fcvt_s_w(RoundingMode::RNE, dest.into(), src.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_to_int(
        &mut self,
        dest_mode: MachineMode,
        dest: Reg,
        src_mode: MachineMode,
        src: FReg,
    ) {
        match (dest_mode, src_mode) {
            (MachineMode::Float64, MachineMode::Int64) => {
                self.asm
                    .fcvt_l_d(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float32, MachineMode::Int64) => {
                self.asm
                    .fcvt_w_d(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float64, MachineMode::Int32) => {
                self.asm
                    .fcvt_l_s(RoundingMode::RNE, dest.into(), src.into())
            }
            (MachineMode::Float32, MachineMode::Int32) => {
                self.asm
                    .fcvt_w_s(RoundingMode::RNE, dest.into(), src.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float32_to_float64(&mut self, dest: FReg, src: FReg) {
        self.asm
            .fcvt_s_d(RoundingMode::RNE, dest.into(), src.into())
    }

    pub fn float64_to_float32(&mut self, dest: FReg, src: FReg) {
        self.asm
            .fcvt_d_s(RoundingMode::RNE, dest.into(), src.into())
    }

    pub fn int_as_float(
        &mut self,
        dest_mode: MachineMode,
        dest: FReg,
        src_mode: MachineMode,
        src: Reg,
    ) {
        match (dest_mode, src_mode) {
            (MachineMode::Float64, MachineMode::Int64) => self.asm.fmv_d_x(dest.into(), src.into()),
            (MachineMode::Float32, MachineMode::Int32) => self.asm.fmv_w_x(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_as_int(
        &mut self,
        dest_mode: MachineMode,
        dest: Reg,
        src_mode: MachineMode,
        src: FReg,
    ) {
        match (dest_mode, src_mode) {
            (MachineMode::Int64, MachineMode::Float64) => self.asm.fmv_x_d(dest.into(), src.into()),
            (MachineMode::Int32, MachineMode::Float32) => self.asm.fmv_x_w(dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_add(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float64 => {
                self.asm
                    .fadd_d(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            MachineMode::Float32 => {
                self.asm
                    .fadd_s(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_sub(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float64 => {
                self.asm
                    .fsub_d(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            MachineMode::Float32 => {
                self.asm
                    .fsub_s(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_mul(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float64 => {
                self.asm
                    .fmul_d(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            MachineMode::Float32 => {
                self.asm
                    .fmul_s(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_div(&mut self, mode: MachineMode, dest: FReg, lhs: FReg, rhs: FReg) {
        match mode {
            MachineMode::Float64 => {
                self.asm
                    .fdiv_d(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            MachineMode::Float32 => {
                self.asm
                    .fdiv_s(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_abs(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float64 => self.asm.fsgnjx_d(dest.into(), src.into(), src.into()),
            MachineMode::Float32 => self.asm.fsgnjx_s(dest.into(), src.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_neg(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float64 => self.asm.fsgnjn_d(dest.into(), src.into(), src.into()),
            MachineMode::Float32 => self.asm.fsgnjn_s(dest.into(), src.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_round_tozero(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float64 => {
                //self.asm
                //    .fround_d(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            MachineMode::Float32 => {
                //self.asm
                //    .fround_s(RoundingMode::RNE, dest.into(), lhs.into(), rhs.into())
            }
            _ => unreachable!(),
        }
    }

    pub fn float_round_up(&mut self, mode: MachineMode, dest: FReg, src: FReg) {}

    pub fn float_round_down(&mut self, mode: MachineMode, dest: FReg, src: FReg) {}

    pub fn float_round_halfeven(&mut self, mode: MachineMode, dest: FReg, src: FReg) {}

    pub fn float_sqrt(&mut self, mode: MachineMode, dest: FReg, src: FReg) {
        match mode {
            MachineMode::Float64 => self.asm.fsqrt_d(RoundingMode::RNE, dest.into(), src.into()),
            MachineMode::Float32 => self.asm.fsqrt_s(RoundingMode::RNE, dest.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn float_cmp_int(&mut self, mode: MachineMode, dest: Reg, lhs: FReg, rhs: FReg) {}

    pub fn float_cmp(
        &mut self,
        mode: MachineMode,
        dest: Reg,
        lhs: FReg,
        rhs: FReg,
        cond: CondCode,
    ) {
    }

    pub fn float_cmp_nan(&mut self, mode: MachineMode, dest: Reg, src: FReg) {}

    pub fn float_srt(&mut self, _mode: MachineMode, _dest: Reg, _lhs: FReg, _rhs: FReg) {}

    pub fn load_float_const(&mut self, mode: MachineMode, dest: FReg, imm: f64) {}

    pub fn determine_array_size(
        &mut self,
        dest: Reg,
        length: Reg,
        element_size: i32,
        with_header: bool,
    ) {
    }

    pub fn array_address(&mut self, dest: Reg, obj: Reg, index: Reg, element_size: i32) {}

    pub fn check_index_out_of_bounds(&mut self, pos: Position, array: Reg, index: Reg) {}

    pub fn check_string_index_out_of_bounds(&mut self, pos: Position, array: Reg, index: Reg) {}

    pub fn load_nil(&mut self, dest: Reg) {}

    pub fn load_int32_synchronized(&mut self, dest: Reg, address: Reg) {}

    pub fn load_int64_synchronized(&mut self, dest: Reg, address: Reg) {}

    pub fn store_int32_synchronized(&mut self, dest: Reg, address: Reg) {}

    pub fn store_int64_synchronized(&mut self, dest: Reg, address: Reg) {}

    pub fn exchange_int32_synchronized(&mut self, old: Reg, new: Reg, address: Reg) {}

    pub fn exchange_int64_synchronized(&mut self, old: Reg, new: Reg, address: Reg) {}

    pub fn compare_exchange_int32_synchronized(
        &mut self,
        expected: Reg,
        new: Reg,
        address: Reg,
    ) -> Reg {
        panic!()
    }

    pub fn compare_exchange_int64_synchronized(
        &mut self,
        expected: Reg,
        new: Reg,
        address: Reg,
    ) -> Reg {
        panic!()
    }

    pub fn fetch_add_int32_synchronized(&mut self, previous: Reg, value: Reg, address: Reg) -> Reg {
        panic!()
    }

    pub fn fetch_add_int64_synchronized(&mut self, previous: Reg, value: Reg, address: Reg) -> Reg {
        panic!()
    }

    pub fn load_mem(&mut self, mode: MachineMode, dest: AnyReg, mem: Mem) {}

    fn common_load_base_with_offset(
        &mut self,
        mode: MachineMode,
        dest: AnyReg,
        base: Reg,
        disp: i32,
    ) {
    }

    pub fn lea(&mut self, dest: Reg, mem: Mem) {}

    pub fn emit_barrier(&mut self, src: Reg, card_table_offset: usize) {}

    pub fn store_mem(&mut self, mode: MachineMode, mem: Mem, src: AnyReg) {}

    pub fn store_zero(&mut self, mode: MachineMode, mem: Mem) {}

    pub fn common_store_base_with_offset(
        &mut self,
        mode: MachineMode,
        src: AnyReg,
        base: Reg,
        offset: i32,
    ) {
    }

    pub fn copy_reg(&mut self, mode: MachineMode, dest: Reg, src: Reg) {}

    pub fn copy_sp(&mut self, dest: Reg) {}

    pub fn set_sp(&mut self, src: Reg) {}

    pub fn copy_pc(&mut self, dest: Reg) {}

    pub fn copy_ra(&mut self, dest: Reg) {}

    pub fn copy_freg(&mut self, mode: MachineMode, dest: FReg, src: FReg) {}

    pub fn int32_to_int64(&mut self, dest: Reg, src: Reg) {}

    pub fn extend_byte(&mut self, mode: MachineMode, dest: Reg, src: Reg) {}

    pub fn load_constpool(&mut self, dest: Reg, disp: i32) {}

    pub fn call_reg(&mut self, reg: Reg) {}

    pub fn debug(&mut self) {}

    pub fn load_int_const(&mut self, mode: MachineMode, dest: Reg, imm: i64) {}

    pub fn load_true(&mut self, dest: Reg) {
        self.asm.addi(dest.into(), REG_ZERO.into(), 1);
    }

    pub fn load_false(&mut self, dest: Reg) {
        self.asm.add(dest.into(), REG_ZERO.into(), REG_ZERO.into())
    }

    pub fn int_neg(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.sub(dest.into(), REG_ZERO.into(), src.into()),
            MachineMode::Int32 => self.asm.subw(dest.into(), REG_ZERO.into(), src.into()),
            _ => unreachable!(),
        }
    }

    pub fn int_not(&mut self, mode: MachineMode, dest: Reg, src: Reg) {
        match mode {
            MachineMode::Int64 => self.asm.xori(dest.into(), src.into(), -1),
            MachineMode::Int32 => self.asm.xori(dest.into(), src.into(), -1),
            _ => unreachable!(),
        }
    }

    pub fn bool_not(&mut self, dest: Reg, src: Reg) {
        self.asm.xori(dest.into(), src.into(), 1);
    }

    pub fn trap(&mut self, trap: Trap, pos: Position) {}

    pub fn nop(&mut self) {
        self.asm.addi(REG_ZERO.into(), REG_ZERO.into(), 0)
    }
}

#[derive(Debug)]
pub struct ForwardJump {
    at: usize,
    to: Label,
    ty: JumpType,
}

#[derive(Debug)]
enum JumpType {
    Jump,
    JumpIf(CondCode),
}

fn size_flag(mode: MachineMode) -> u32 {
    match mode {
        MachineMode::Int8 | MachineMode::Int32 => 0,
        MachineMode::IntPtr | MachineMode::Ptr | MachineMode::Int64 => 1,
        MachineMode::Float32 | MachineMode::Float64 => unimplemented!(),
    }
}

impl From<FReg> for FloatRegister {
    fn from(reg: FReg) -> FloatRegister {
        FloatRegister::new(reg.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::MachineMode::{Int32, Ptr};
    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! assert_emit {
        (
            $($expr:expr),*;
            $name:ident
        ) => {{
            $name.finish();
            let expected: Vec<u32> = vec![$($expr,)*];
            let mut buffer: Vec<u8> = Vec::new();

            for insn in expected {
                buffer.write_u32::<LittleEndian>(insn).unwrap();
            }

            assert_eq!(buffer, $name.data());
        }};
    }

    #[test]
    fn test_jump_forward() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.jump(lbl);
        masm.bind_label(lbl);

        assert_emit!(0x14000001; masm);
    }

    #[test]
    fn test_jump_if_forward() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.jump_if(CondCode::Zero, lbl);
        masm.bind_label(lbl);

        assert_emit!(0x54000020; masm);
    }

    #[test]
    fn test_jump_forward_with_gap() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.jump(lbl);
        masm.emit_u32(0);
        masm.bind_label(lbl);

        assert_emit!(0x14000002, 0; masm);
    }

    #[test]
    fn test_jump_if_forward_with_gap() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.jump_if(CondCode::NonZero, lbl);
        masm.emit_u32(0);
        masm.bind_label(lbl);

        assert_emit!(0x54000041, 0; masm);
    }

    #[test]
    fn test_jump_backward() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.bind_label(lbl);
        masm.jump(lbl);

        assert_emit!(0x14000000; masm);
    }

    #[test]
    fn test_jump_if_backward() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.bind_label(lbl);
        masm.jump_if(CondCode::Less, lbl);

        assert_emit!(0x5400000B; masm);
    }

    #[test]
    fn test_jump_backward_with_gap() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.bind_label(lbl);
        masm.emit_u32(0);
        masm.jump(lbl);

        assert_emit!(0, 0x17FFFFFF; masm);
    }

    #[test]
    fn test_jump_if_backward_with_gap() {
        let mut masm = MacroAssembler::new();
        let lbl = masm.create_label();
        masm.bind_label(lbl);
        masm.emit_u32(0);
        masm.jump_if(CondCode::LessEq, lbl);

        assert_emit!(0, 0x54FFFFED; masm);
    }

    #[test]
    fn test_load_int_const() {
        let mut masm = MacroAssembler::new();
        masm.load_int_const(Int32, R0, 0);
        assert_emit!(0x52800000; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Int32, R0, 0xFFFF);
        assert_emit!(0x529FFFE0; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Int32, R0, 1i64 << 16);
        assert_emit!(0x52a00020; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Ptr, R0, 0);
        assert_emit!(0xD2800000; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Int32, R0, -1);
        assert_emit!(0x12800000; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Ptr, R0, -1);
        assert_emit!(0x92800000; masm);
    }

    #[test]
    fn test_load_int_const_multiple_halfwords() {
        let mut masm = MacroAssembler::new();
        masm.load_int_const(Int32, R0, 0x10001);
        assert_emit!(0x52800020, 0x72a00020; masm);

        let mut masm = MacroAssembler::new();
        masm.load_int_const(Ptr, R0, !0x10001);
        assert_emit!(0x92800020, 0xF2BFFFC0; masm);
    }
}
