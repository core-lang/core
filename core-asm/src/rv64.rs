use crate::{AssemblerBuffer, Label};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Register(u8);

impl Register {
    pub fn new(value: u8) -> Register {
        assert!(value < 31);
        Register(value)
    }

    fn rd(self) -> u32 {
        (self.0 as u32) << 7
    }

    fn rs1(self) -> u32 {
        (self.0 as u32) << 15
    }

    fn rs2(self) -> u32 {
        (self.0 as u32) << 20
    }

    fn rs3(self) -> u32 {
        (self.0 as u32) << 27
    }
}

pub const R0: Register = Register(0);
pub const R1: Register = Register(1);
pub const R2: Register = Register(2);
pub const R3: Register = Register(3);
pub const R4: Register = Register(4);
pub const R5: Register = Register(5);
pub const R6: Register = Register(6);
pub const R7: Register = Register(7);
pub const R8: Register = Register(8);
pub const R9: Register = Register(9);
pub const R10: Register = Register(10);
pub const R11: Register = Register(11);
pub const R12: Register = Register(12);
pub const R13: Register = Register(13);
pub const R14: Register = Register(14);
pub const R15: Register = Register(15);
pub const R16: Register = Register(16);
pub const R17: Register = Register(17);
pub const R18: Register = Register(18);
pub const R19: Register = Register(19);
pub const R20: Register = Register(20);
pub const R21: Register = Register(21);
pub const R22: Register = Register(22);
pub const R23: Register = Register(23);
pub const R24: Register = Register(24);
pub const R25: Register = Register(25);
pub const R26: Register = Register(26);
pub const R27: Register = Register(27);
pub const R28: Register = Register(28);
pub const R29: Register = Register(29);
pub const R30: Register = Register(30);
pub const R31: Register = Register(31);
pub const REG_ZERO: Register = Register(0);
pub const REG_SP: Register = Register(2);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloatRegister(u8);

impl FloatRegister {
    pub fn new(value: u8) -> FloatRegister {
        assert!(value < 32);
        FloatRegister(value)
    }

    #[allow(dead_code)]
    fn encoding(self) -> u32 {
        self.0 as u32
    }

    fn rd(self) -> u32 {
        (self.0 as u32) << 7
    }

    fn rs1(self) -> u32 {
        (self.0 as u32) << 15
    }

    fn rs2(self) -> u32 {
        (self.0 as u32) << 20
    }

    #[allow(dead_code)]
    fn rs3(self) -> u32 {
        (self.0 as u32) << 27
    }
}

pub const F0: FloatRegister = FloatRegister(0);
pub const F1: FloatRegister = FloatRegister(1);
pub const F2: FloatRegister = FloatRegister(2);
pub const F3: FloatRegister = FloatRegister(3);
pub const F4: FloatRegister = FloatRegister(4);
pub const F5: FloatRegister = FloatRegister(5);
pub const F6: FloatRegister = FloatRegister(6);
pub const F7: FloatRegister = FloatRegister(7);
pub const F8: FloatRegister = FloatRegister(8);
pub const F9: FloatRegister = FloatRegister(9);
pub const F10: FloatRegister = FloatRegister(10);
pub const F11: FloatRegister = FloatRegister(11);
pub const F12: FloatRegister = FloatRegister(12);
pub const F13: FloatRegister = FloatRegister(13);
pub const F14: FloatRegister = FloatRegister(14);
pub const F15: FloatRegister = FloatRegister(15);
pub const F16: FloatRegister = FloatRegister(16);
pub const F17: FloatRegister = FloatRegister(17);
pub const F18: FloatRegister = FloatRegister(18);
pub const F19: FloatRegister = FloatRegister(19);
pub const F20: FloatRegister = FloatRegister(20);
pub const F21: FloatRegister = FloatRegister(21);
pub const F22: FloatRegister = FloatRegister(22);
pub const F23: FloatRegister = FloatRegister(23);
pub const F24: FloatRegister = FloatRegister(24);
pub const F25: FloatRegister = FloatRegister(25);
pub const F26: FloatRegister = FloatRegister(26);
pub const F27: FloatRegister = FloatRegister(27);
pub const F28: FloatRegister = FloatRegister(28);
pub const F29: FloatRegister = FloatRegister(29);
pub const F30: FloatRegister = FloatRegister(30);
pub const F31: FloatRegister = FloatRegister(31);

#[allow(dead_code)]
struct ForwardJump {
    offset: u32,
    label: Label,
    kind: JumpKind,
}

#[allow(dead_code)]
enum JumpKind {
    Unconditional,
    Conditional, /*(Cond)*/
    NonZero(bool, Register),
    Zero(bool, Register),
}

pub struct AssemblerRv64 {
    #[allow(dead_code)]
    unresolved_jumps: Vec<ForwardJump>,
    buffer: AssemblerBuffer,
}

impl AssemblerRv64 {
    pub fn new() -> AssemblerRv64 {
        AssemblerRv64 {
            unresolved_jumps: Vec::new(),
            buffer: AssemblerBuffer::new(),
        }
    }

    pub fn create_label(&mut self) -> Label {
        self.buffer.create_label()
    }

    pub fn create_and_bind_label(&mut self) -> Label {
        self.buffer.create_and_bind_label()
    }

    pub fn bind_label(&mut self, lbl: Label) {
        self.buffer.bind_label(lbl);
    }

    #[allow(dead_code)]
    fn offset(&self, lbl: Label) -> Option<u32> {
        self.buffer.offset(lbl)
    }

    pub fn finalize(mut self, alignment: Option<usize>) -> Vec<u8> {
        //self.resolve_jumps();
        if let Some(alignment) = alignment {
            self.align_to(alignment);
        }
        self.buffer.code
    }

    fn align_to(&mut self, alignment: usize) {
        while self.buffer.code.len() % alignment != 0 {
            //self.brk(0);
        }
        assert_eq!(self.buffer.code.len() % alignment, 0);
    }

    pub fn position(&self) -> usize {
        self.buffer.position()
    }

    pub fn set_position(&mut self, pos: usize) {
        self.buffer.set_position(pos);
    }

    pub fn set_position_end(&mut self) {
        self.buffer.set_position_end();
    }

    pub fn emit_u8(&mut self, value: u8) {
        self.buffer.emit_u8(value);
    }

    pub fn emit_u32(&mut self, value: u32) {
        self.buffer.emit_u32(value);
    }

    pub fn emit_u64(&mut self, value: u64) {
        self.buffer.emit_u64(value);
    }
}

/** see unpriv-isa-asciidoc.html#opcodemap */
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
enum Opcode {
    Load = 0b0000011,
    LoadFP = 0b0000111,
    Custom0 = 0b0001011,
    MiscMem = 0b0001111,
    OpImm = 0b0010011,
    AUIPC = 0b0010111,
    OpImm32 = 0b0011011,
    Store = 0b0100011,
    StoreFp = 0b0100111,
    Custom1 = 0b0101011,
    AMO = 0b0101111,
    Op = 0b0110011,
    LUI = 0b0110111,
    Op32 = 0b0111011,
    MAdd = 0b1000011,
    MSub = 0b1000111,
    NMSub = 0b1001011,
    NMAdd = 0b1001111,
    OpFP = 0b1010011,
    Reserved0 = 0b1010111,
    RV128_0 = 0b1011011,
    Branch = 0b1100011,
    JALR = 0b1100111,
    Reserved1 = 0b1101011,
    JAL = 0b1101111,
    System = 0b1110011,
    Reserved2 = 0b1110111,
    RV128_1 = 0b1111011,
}

impl Opcode {
    fn encoding(self: Opcode) -> u32 {
        self as u32
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MemoryAccess {
    None = 0b0000,
    W = 0b0001,
    R = 0b0010,
    RW = 0b0011,
    O = 0b0100,
    I = 0b1000,
    IORW = 0b1111,
}

impl MemoryAccess {
    pub fn pred(self) -> i32 {
        (self as i32) << 20
    }
    pub fn succ(self) -> i32 {
        (self as i32) << 16
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MemoryOrdering {
    None = 0b00,
    Release = 0b01,
    Acquire = 0b10,
    AcquireRelease = 0b11,
}

impl MemoryOrdering {
    pub fn encoding(self) -> u32 {
        (self as u32) << 25
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum FloatFormat {
    S = 0b00,
    D = 0b01,
    H = 0b10,
    Q = 0b11,
}

impl FloatFormat {
    pub fn encoding(self) -> u32 {
        (self as u32) << 26
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum RoundingMode {
    RNE = 0b000,
    RTZ = 0b001,
    RDN = 0b010,
    RUP = 0b011,
    RMM = 0b100,
}

impl RoundingMode {
    pub fn value(self) -> u32 {
        self as u32
    }
    pub fn encoding(self) -> u32 {
        (self as u32) << 12
    }
}

use crate::rv64::Opcode::*;

impl AssemblerRv64 {
    fn r(
        &mut self,
        major_opcode: Opcode,
        funct3: u32,
        funct7: u32,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) {
        let instruction: u32 = (funct7 << 25)
            | rs2.rs2()
            | rs1.rs1()
            | (funct3 << 12)
            | rd.rd()
            | major_opcode.encoding();
        self.emit_u32(instruction);
    }

    fn r_aqrl(
        &mut self,
        funct3: u32,
        funct5: u32,
        mem_order: MemoryOrdering,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) {
        let instruction = (funct5 << 27)
            | mem_order.encoding()
            | rs2.rs2()
            | rs1.rs1()
            | (funct3 << 12)
            | rd.rd()
            | AMO.encoding();
        self.emit_u32(instruction);
    }

    fn r4rm(
        &mut self,
        major_opcode: Opcode,
        fmt: FloatFormat,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        let instruction: u32 = rs3.rs3()
            | fmt.encoding()
            | rs2.rs2()
            | rs1.rs1()
            | rm.encoding()
            | rd.rd()
            | major_opcode.encoding();
        self.emit_u32(instruction);
    }

    fn r_fp(
        &mut self,
        funct3: u32,
        funct7: u32,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        let instruction: u32 =
            (funct7 << 25) | rs2.rs2() | rs1.rs1() | (funct3 << 12) | rd.rd() | OpFP.encoding();
        self.emit_u32(instruction);
    }

    fn r_fp_to_int(
        &mut self,
        funct3: u32,
        funct7: u32,
        rd: Register,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        let instruction: u32 =
            (funct7 << 25) | rs2.rs2() | rs1.rs1() | (funct3 << 12) | rd.rd() | OpFP.encoding();
        self.emit_u32(instruction);
    }

    fn r_int_to_fp(
        &mut self,
        funct3: u32,
        funct7: u32,
        rd: FloatRegister,
        rs1: Register,
        rs2: Register,
    ) {
        let instruction: u32 =
            (funct7 << 25) | rs2.rs2() | rs1.rs1() | (funct3 << 12) | rd.rd() | OpFP.encoding();
        self.emit_u32(instruction);
    }

    fn i(&mut self, major_opcode: Opcode, funct3: u32, rd: Register, rs1: Register, imm12: i32) {
        let instruction =
            ((imm12 as u32) << 20) | rs1.rs1() | (funct3 << 12) | rd.rd() | major_opcode.encoding();
        self.emit_u32(instruction);
    }
    fn s(&mut self, major_opcode: Opcode, funct3: u32, rs1: Register, rs2: Register, imm12: i32) {
        let imm12 = imm12 as u32;
        let imm7 = (imm12 & 0b111111100000) << 20;
        let imm5 = (imm12 & 0b11111) << 7;
        let instruction =
            imm7 | rs2.rs2() | rs1.rs1() | (funct3 << 12) | imm5 | major_opcode.encoding();
        self.emit_u32(instruction);
    }
    fn b(&mut self, funct3: u32, rs1: Register, rs2: Register, imm12: i32) {
        // immediate is a signed offset in multiples of 2 bytes, so bit 0 is always 0
        let imm12 = imm12 as u32;
        let imm7_1 = (imm12 & 0b1000000000000) << 19;
        let imm7_2 = (imm12 & 0b0011111100000) << 20;
        let imm5_1 = (imm12 & 0b0000000011110) << 7;
        let imm5_2 = (imm12 & 0b0100000000000) >> 4;
        let instruction = imm7_1
            | imm7_2
            | rs2.rs2()
            | rs1.rs1()
            | (funct3 << 12)
            | imm5_1
            | imm5_2
            | Store.encoding();
        self.emit_u32(instruction);
    }
    fn u(&mut self, major_opcode: Opcode, rd: Register, imm20: i32) {
        let instruction = ((imm20 as u32) & 0b11111111111111111111000000000000)
            | rd.rd()
            | major_opcode.encoding();
        self.emit_u32(instruction);
    }

    #[allow(unused_parens)]
    fn j(&mut self, rd: Register, imm20: i32) {
        // immediate is a signed offset in multiples of 2 bytes, so bit 0 is always 0
        let imm20_1 = (imm20 as u32 & 0b100000000000000000000) << 11;
        let imm20_2 = (imm20 as u32 & 0b000000000011111111110) << 20;
        let imm20_3 = (imm20 as u32 & 0b000000000100000000000) << 9;
        let imm20_4 = (imm20 as u32 & 0b011111111000000000000);
        let instruction = imm20_1 | imm20_2 | imm20_3 | imm20_4 | rd.rd() | JAL.encoding();
        self.emit_u32(instruction);
    }
}

impl AssemblerRv64 {
    // ▼▼ RV32I Base Instruction Set ▼▼
    pub fn lui(&mut self, rd: Register, imm: i32) {
        self.u(LUI, rd, imm)
    }
    pub fn auipc(&mut self, rd: Register, imm: i32) {
        self.u(AUIPC, rd, imm)
    }
    pub fn jal(&mut self, rd: Register, imm: i32) {
        self.j(rd, imm)
    }
    pub fn jalr(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(JALR, 0b000, rd, rs1, imm)
    }
    pub fn beq(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b000, rs1, rs2, imm)
    }
    pub fn bne(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b001, rs1, rs2, imm)
    }
    pub fn blt(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b100, rs1, rs2, imm)
    }
    pub fn bge(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b101, rs1, rs2, imm)
    }
    pub fn bltu(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b110, rs1, rs2, imm)
    }
    pub fn bgeu(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.b(0b111, rs1, rs2, imm)
    }
    pub fn lb(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b000, rd, rs1, imm)
    }
    pub fn lh(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b001, rd, rs1, imm)
    }
    pub fn lw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b010, rd, rs1, imm)
    }
    pub fn lbu(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b100, rd, rs1, imm)
    }
    pub fn lhu(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b101, rd, rs1, imm)
    }
    pub fn sb(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(Store, 0b000, rs1, rs2, imm)
    }
    pub fn sh(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(Store, 0b001, rs1, rs2, imm)
    }
    pub fn sw(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(Store, 0b010, rs1, rs2, imm)
    }
    pub fn addi(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b000, rd, rs1, imm)
    }
    /** pseudo-instruction */
    pub fn mv(&mut self, rd: Register, rs1: Register) {
        self.addi(rd, rs1, 0)
    }
    pub fn slti(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b010, rd, rs1, imm)
    }
    pub fn sltiu(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b011, rd, rs1, imm)
    }
    /** pseudo-instruction */
    pub fn seqz(&mut self, rd: Register, rs1: Register) {
        self.sltiu(rd, rs1, 1)
    }
    pub fn xori(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b100, rd, rs1, imm)
    }
    /** pseudo-instruction */
    pub fn not(&mut self, rd: Register, rs1: Register) {
        self.xori(rd, rs1, -1)
    }
    pub fn ori(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b110, rd, rs1, imm)
    }
    pub fn andi(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b111, rd, rs1, imm)
    }
    pub fn slli(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b001, rd, rs1, imm)
    }

    pub fn srli(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b101, rd, rs1, imm)
    }

    pub fn srai(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b101, rd, rs1, imm | 0b0100000)
    }
    pub fn add(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b000, 0b0000000, rd, rs1, rs2)
    }
    pub fn sub(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b000, 0b0100000, rd, rs1, rs2)
    }
    /** pseudo-instruction */
    pub fn neg(&mut self, rd: Register, rs1: Register) {
        self.sub(rd, REG_ZERO.into(), rs1)
    }
    pub fn sll(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b001, 0b0000000, rd, rs1, rs2)
    }
    pub fn slt(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b010, 0b0000000, rd, rs1, rs2)
    }
    /** pseudo-instruction */
    pub fn sltz(&mut self, rd: Register, rs1: Register) {
        self.slt(rd, rs1, REG_ZERO)
    }
    /** pseudo-instruction */
    pub fn sgtz(&mut self, rd: Register, rs1: Register) {
        self.slt(rd, REG_ZERO, rs1)
    }
    pub fn sltu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b011, 0b0000000, rd, rs1, rs2)
    }
    /** pseudo-instruction */
    pub fn snez(&mut self, rd: Register, rs1: Register) {
        self.sltu(rd, REG_ZERO.into(), rs1);
    }
    pub fn xor(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b100, 0b0000000, rd, rs1, rs2)
    }
    pub fn srl(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b101, 0b0000000, rd, rs1, rs2)
    }
    pub fn sra(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b101, 0b0100000, rd, rs1, rs2)
    }
    pub fn or(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b110, 0b0000000, rd, rs1, rs2)
    }
    pub fn and(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b111, 0b0000000, rd, rs1, rs2)
    }
    pub fn fence(&mut self, pred: MemoryAccess, succ: MemoryAccess) {
        self.i(
            MiscMem,
            0b000,
            REG_ZERO,
            REG_ZERO,
            pred.pred() | succ.succ(),
        )
    }
    pub fn fence_tso(&mut self) {
        let fm = 0b1000 << 28;
        let pred = 0b0011 << 24;
        let succ = 0b0011 << 24;
        self.i(MiscMem, 0b000, REG_ZERO, REG_ZERO, fm | pred | succ)
    }
    pub fn ecall(&mut self) {
        self.i(System, 0b000, REG_ZERO, REG_ZERO, 0)
    }
    pub fn ebreak(&mut self) {
        self.i(System, 0b001, REG_ZERO, REG_ZERO, 1)
    }
    // ▲▲ RV32I Base Instruction Set ▲▲
    // ▼▼ RV64I Base Instruction Set (in addition to RV32I) ▼▼
    pub fn lwu(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b110, rd, rs1, imm)
    }
    pub fn ld(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(Load, 0b011, rd, rs1, imm)
    }
    pub fn sd(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(Store, 0b011, rs1, rs2, imm)
    }
    pub fn addiw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b000, rd, rs1, imm)
    }
    pub fn slliw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b001, rd, rs1, imm)
    }
    pub fn srliw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b101, rd, rs1, imm)
    }
    pub fn sraiw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b101, rd, rs1, imm | 0b0100000)
    }
    pub fn addw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b000, 0b0000000, rd, rs1, rs2)
    }
    pub fn subw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b000, 0b0100000, rd, rs1, rs2)
    }
    pub fn sllw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b001, 0b0000000, rd, rs1, rs2)
    }
    pub fn srlw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b101, 0b0000000, rd, rs1, rs2)
    }
    pub fn sraw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b101, 0b0100000, rd, rs1, rs2)
    }
    // ▲▲ RV64I Base Instruction Set (in addition to RV32I) ▲▲
    // ▼▼ RV32/RV64 Zifencei Standard Extension ▼▼
    pub fn fencei(&mut self) {
        self.i(MiscMem, 0b001, REG_ZERO, REG_ZERO, 0)
    }
    // ▲▲ RV32/RV64 Zifencei Standard Extension ▲▲
    // ▼▼ RV32/RV64 Zicsr Standard Extension ▼▼
    pub fn csrrw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(System, 0b001, rd, rs1, imm)
    }
    pub fn csrrs(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(System, 0b010, rd, rs1, imm)
    }
    pub fn csrrc(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(System, 0b011, rd, rs1, imm)
    }
    pub fn csrrwi(&mut self, rd: Register, imm: i32) {
        self.u(System, rd, 0b101 << 12 | imm << 15)
    }
    pub fn csrrsi(&mut self, rd: Register, imm: i32) {
        self.u(System, rd, 0b110 << 12 | imm << 15)
    }
    pub fn csrrci(&mut self, rd: Register, imm: i32) {
        self.u(System, rd, 0b111 << 12 | imm << 15)
    }
    // ▲▲ RV32/RV64 Zicsr Standard Extension ▲▲
    // ▼▼ RV32M Standard Extension ▼▼
    pub fn mul(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b000, 0b0000001, rd, rs1, rs2)
    }
    pub fn mulh(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b001, 0b0000001, rd, rs1, rs2)
    }
    pub fn mulhsu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b010, 0b0000001, rd, rs1, rs2)
    }
    pub fn mulhu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b011, 0b0000001, rd, rs1, rs2)
    }
    pub fn div(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b100, 0b0000001, rd, rs1, rs2)
    }
    pub fn divu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b101, 0b0000001, rd, rs1, rs2)
    }
    pub fn rem(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b110, 0b0000001, rd, rs1, rs2)
    }
    pub fn remu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b111, 0b0000001, rd, rs1, rs2)
    }
    // ▲▲ RV32M Standard Extension ▲▲
    // ▼▼ RV64M Standard Extension (in addition to RV32M) ▼▼
    pub fn mulw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b000, 0b0000001, rd, rs1, rs2)
    }
    pub fn divw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b100, 0b0000001, rd, rs1, rs2)
    }
    pub fn divuw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b101, 0b0000001, rd, rs1, rs2)
    }
    pub fn remw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b110, 0b0000001, rd, rs1, rs2)
    }
    pub fn remuw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b111, 0b0000001, rd, rs1, rs2)
    }
    // ▲▲ RV64M Standard Extension (in addition to RV32M) ▲▲
    // ▼▼ RV32A Standard Extension ▼▼
    pub fn lr_w(&mut self, rd: Register, rs1: Register, mem_order: MemoryOrdering) {
        self.r_aqrl(0b010, 0b00010, mem_order, rd, rs1, REG_ZERO)
    }
    pub fn sc_w(&mut self, rd: Register, rs1: Register, rs2: Register, mem_order: MemoryOrdering) {
        self.r_aqrl(0b010, 0b00011, mem_order, rd, rs1, rs2)
    }
    pub fn amoswap_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b00001, mem_order, rd, rs1, rs2)
    }
    pub fn amoadd_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b00000, mem_order, rd, rs1, rs2)
    }
    pub fn amoxor_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b00100, mem_order, rd, rs1, rs2)
    }
    pub fn amoand_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b01100, mem_order, rd, rs1, rs2)
    }
    pub fn amoor_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b01000, mem_order, rd, rs1, rs2)
    }
    pub fn amomin_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b10000, mem_order, rd, rs1, rs2)
    }
    pub fn amomax_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b10100, mem_order, rd, rs1, rs2)
    }
    pub fn amominu_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b11000, mem_order, rd, rs1, rs2)
    }
    pub fn amomaxu_w(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b010, 0b11100, mem_order, rd, rs1, rs2)
    }
    // ▲▲ RV32A Standard Extension ▲▲
    // ▼▼ RV64A Standard Extension (in addition to RV32A) ▼▼
    pub fn lr_d(&mut self, rd: Register, rs1: Register, mem_order: MemoryOrdering) {
        self.r_aqrl(0b011, 0b00010, mem_order, rd, rs1, REG_ZERO)
    }
    pub fn sc_d(&mut self, rd: Register, rs1: Register, rs2: Register, mem_order: MemoryOrdering) {
        self.r_aqrl(0b011, 0b00011, mem_order, rd, rs1, rs2)
    }
    pub fn amoswap_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b00001, mem_order, rd, rs1, rs2)
    }
    pub fn amoadd_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b00000, mem_order, rd, rs1, rs2)
    }
    pub fn amoxor_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b00100, mem_order, rd, rs1, rs2)
    }
    pub fn amoand_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b01100, mem_order, rd, rs1, rs2)
    }
    pub fn amoor_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b01000, mem_order, rd, rs1, rs2)
    }
    pub fn amomin_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b10000, mem_order, rd, rs1, rs2)
    }
    pub fn amomax_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b10100, mem_order, rd, rs1, rs2)
    }
    pub fn amominu_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b11000, mem_order, rd, rs1, rs2)
    }
    pub fn amomaxu_d(
        &mut self,
        rd: Register,
        rs1: Register,
        rs2: Register,
        mem_order: MemoryOrdering,
    ) {
        self.r_aqrl(0b011, 0b11100, mem_order, rd, rs1, rs2)
    }
    // ▲▲ RV64A Standard Extension (in addition to RV32A) ▲▲
    // ▼▼ RV32F Standard Extension ▼▼
    pub fn flw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(LoadFP, 0b010, rd, rs1, imm)
    }
    pub fn fsw(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(StoreFp, 0b010, rs1, rs2, imm)
    }
    pub fn fmadd_s(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(MAdd, FloatFormat::S, rm, rd, rs1, rs2, rs3)
    }
    pub fn fmsub_s(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(MSub, FloatFormat::S, rm, rd, rs1, rs2, rs3)
    }
    pub fn fnmsub_s(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(NMSub, FloatFormat::S, rm, rd, rs1, rs2, rs3)
    }
    pub fn fnmadd_s(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(NMAdd, FloatFormat::S, rm, rd, rs1, rs2, rs3)
    }
    pub fn fadd_s(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0000000, rd, rs1, rs2)
    }
    pub fn fsub_s(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0000100, rd, rs1, rs2)
    }
    pub fn fmul_s(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0001000, rd, rs1, rs2)
    }
    pub fn fdiv_s(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0001100, rd, rs1, rs2)
    }
    pub fn fsqrt_s(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: FloatRegister) {
        self.r_fp(rm.value(), 0b0101100, rd, rs1, F0)
    }
    pub fn fsgnj_s(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b000, 0b0010000, rd, rs1, rs2)
    }
    pub fn fsgnjn_s(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b001, 0b0010000, rd, rs1, rs2)
    }
    pub fn fsgnjx_s(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b010, 0b0010000, rd, rs1, rs2)
    }
    pub fn fmin_s(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b000, 0b0010100, rd, rs1, rs2)
    }
    pub fn fmax_s(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b001, 0b0010100, rd, rs1, rs2)
    }
    pub fn fcvt_w_s(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100000, rd, rs1, F0)
    }
    pub fn fcvt_wu_s(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100000, rd, rs1, F1)
    }
    pub fn fmv_x_w(&mut self, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(0b000, 0b1110000, rd, rs1, F0)
    }
    pub fn feq_s(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b010, 0b10100000, rd, rs1, rs2)
    }
    pub fn flt_s(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b001, 0b10100000, rd, rs1, rs2)
    }
    pub fn fle_s(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b000, 0b10100000, rd, rs1, rs2)
    }
    pub fn fclass_s(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b001, 0b11100000, rd, rs1, rs2)
    }
    pub fn fcvt_s_w(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101000, rd, rs1, R0)
    }
    pub fn fcvt_s_wu(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101000, rd, rs1, R1)
    }
    pub fn fmv_w_x(&mut self, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(0b000, 0b1111000, rd, rs1, R0)
    }
    // ▲▲ RV32F Standard Extension ▲▲
    // ▼▼ RV64F Standard Extension (in addition to RV32F) ▼▼
    pub fn fcvt_l_s(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100000, rd, rs1, F2)
    }
    pub fn fcvt_lu_s(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100000, rd, rs1, F3)
    }
    pub fn fcvt_s_l(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101000, rd, rs1, R2)
    }
    pub fn fcvt_s_lu(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101000, rd, rs1, R3)
    }
    // ▲▲ RV64F Standard Extension (in addition to RV32F) ▲▲
    // ▼▼ RV32D Standard Extension ▼▼
    pub fn fld(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(LoadFP, 0b011, rd, rs1, imm)
    }
    pub fn fsd(&mut self, rs1: Register, rs2: Register, imm: i32) {
        self.s(StoreFp, 0b011, rs1, rs2, imm)
    }
    pub fn fmadd_d(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(MAdd, FloatFormat::D, rm, rd, rs1, rs2, rs3)
    }
    pub fn fmsub_d(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(MSub, FloatFormat::D, rm, rd, rs1, rs2, rs3)
    }
    pub fn fnmsub_d(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(NMSub, FloatFormat::D, rm, rd, rs1, rs2, rs3)
    }
    pub fn fnmadd_d(
        &mut self,
        rm: RoundingMode,
        rd: Register,
        rs1: Register,
        rs2: Register,
        rs3: Register,
    ) {
        self.r4rm(NMAdd, FloatFormat::D, rm, rd, rs1, rs2, rs3)
    }
    pub fn fadd_d(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0000001, rd, rs1, rs2)
    }
    pub fn fsub_d(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0000101, rd, rs1, rs2)
    }
    pub fn fmul_d(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0001001, rd, rs1, rs2)
    }
    pub fn fdiv_d(
        &mut self,
        rm: RoundingMode,
        rd: FloatRegister,
        rs1: FloatRegister,
        rs2: FloatRegister,
    ) {
        self.r_fp(rm.value(), 0b0001101, rd, rs1, rs2)
    }
    pub fn fsqrt_d(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: FloatRegister) {
        self.r_fp(rm.value(), 0b0101101, rd, rs1, F0)
    }
    pub fn fsgnj_d(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b000, 0b0010001, rd, rs1, rs2)
    }
    pub fn fsgnjn_d(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b001, 0b0010011, rd, rs1, rs2)
    }
    pub fn fsgnjx_d(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b010, 0b0010010, rd, rs1, rs2)
    }
    pub fn fmin_d(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b000, 0b0010101, rd, rs1, rs2)
    }
    pub fn fmax_d(&mut self, rd: FloatRegister, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp(0b001, 0b0010101, rd, rs1, rs2)
    }
    pub fn fcvt_s_d(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: FloatRegister) {
        self.r_fp(rm.value(), 0b0100000, rd, rs1, F1)
    }
    pub fn fcvt_d_s(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: FloatRegister) {
        self.r_fp(rm.value(), 0b0100001, rd, rs1, F0)
    }
    pub fn feq_d(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b010, 0b10100001, rd, rs1, rs2)
    }
    pub fn flt_d(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b001, 0b10100001, rd, rs1, rs2)
    }
    pub fn fle_d(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b000, 0b10100001, rd, rs1, rs2)
    }
    pub fn fclass_d(&mut self, rd: Register, rs1: FloatRegister, rs2: FloatRegister) {
        self.r_fp_to_int(0b001, 0b11100001, rd, rs1, rs2)
    }
    pub fn fcvt_w_d(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100001, rd, rs1, F0)
    }
    pub fn fcvt_wu_d(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100001, rd, rs1, F1)
    }
    pub fn fcvt_d_w(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101001, rd, rs1, R0)
    }
    pub fn fcvt_d_wu(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101001, rd, rs1, R1)
    }
    // ▲▲ RV32D Standard Extension ▲▲
    // ▼▼ RV64D Standard Extension (in addition to RV32D) ▼▼
    pub fn fcvt_l_d(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100001, rd, rs1, F2)
    }
    pub fn fcvt_lu_d(&mut self, rm: RoundingMode, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(rm.value(), 0b1100001, rd, rs1, F3)
    }
    pub fn fmv_x_d(&mut self, rd: Register, rs1: FloatRegister) {
        self.r_fp_to_int(0b000, 0b1110001, rd, rs1, F0)
    }
    pub fn fcvt_d_l(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101001, rd, rs1, R2)
    }
    pub fn fcvt_d_lu(&mut self, rm: RoundingMode, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(rm.value(), 0b1101001, rd, rs1, R3)
    }
    pub fn fmv_d_x(&mut self, rd: FloatRegister, rs1: Register) {
        self.r_int_to_fp(0b000, 0b1111001, rd, rs1, R0)
    }
    // ▲▲ RV64D Standard Extension (in addition to RV32D) ▲▲
    // ▼▼ RV32B Standard Extension ▼▼
    //  ▼ Zba: address generation ▼
    /** shift left by 1 and add */
    pub fn sh1add(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b010, 0010000, rd, rs1, rs2)
    }
    /** shift left by 2 and add */
    pub fn sh2add(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b100, 0010000, rd, rs1, rs2)
    }
    /** shift left by 3 and add */
    pub fn sh3add(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b110, 0010000, rd, rs1, rs2)
    }
    //  ▲ Zba: address generation ▲
    //  ▼ Zbb: bit-manipulation ▼
    /** AND with inverted operand */
    pub fn andn(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b111, 0b0100000, rd, rs1, rs2)
    }
    /** OR with inverted operand */
    pub fn orn(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b110, 0b0100000, rd, rs1, rs2)
    }
    /** exclusive NOR */
    pub fn xnor(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b100, 0b0100000, rd, rs1, rs2)
    }
    /** count leading zero bits */
    pub fn clz(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b001, rd, rs1, 0b0110000_00000)
    }
    /** count trailing zero bits */
    pub fn ctz(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b001, rd, rs1, 0b0110000_00001)
    }
    /** count set bits in word */
    pub fn cpop(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b001, rd, rs1, 0b0110000_00010)
    }
    /** maximum */
    pub fn max(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b110, 0b0000101, rd, rs1, rs2)
    }
    /** unsigned maximum */
    pub fn maxu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b111, 0b0000101, rd, rs1, rs2)
    }
    /** minimum */
    pub fn min(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b100, 0b0000101, rd, rs1, rs2)
    }
    /** unsigned minimum */
    pub fn minu(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b101, 0b0000101, rd, rs1, rs2)
    }
    /** sign-extend byte */
    pub fn sext_b(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b001, rd, rs1, 0b0110000_00100)
    }
    /** sign-extend halfword */
    pub fn sext_h(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b001, rd, rs1, 0b0110000_00101)
    }
    /** Zero-extend halfword */
    pub fn zext_h_32(&mut self, rd: Register, rs1: Register) {
        self.i(Op, 0b100, rd, rs1, 0b0000100_00000)
    }
    /** rotate left (register) */
    pub fn rol(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b001, 0b0110000, rd, rs1, rs2)
    }
    /** rotate right (register) */
    pub fn ror(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op, 0b101, 0b0110000, rd, rs1, rs2)
    }
    /** rotate right (immediate) */
    pub fn rori(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm, 0b101, rd, rs1, (imm & 0b111111) | 0b011000_000000);
    }
    /** bitwise OR-combine, byte granule */
    pub fn orc_b(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b101, rd, rs1, 0b001010000111);
    }
    /** byte-reverse register */
    pub fn rev8_32(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b101, rd, rs1, 0b011010011000);
    }
    //  ▲ Zbb: bit-manipulation ▲
    // ▲▲ RV32B Standard Extension ▲▲
    // ▼▼ RV64B Standard Extension (in addition to RV32B) ▼▼
    //  ▼ Zba: address generation ▼
    /** add unsigned word */
    pub fn add_uw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b000, 0000100, rd, rs1, rs2)
    }
    /** shift unsigned word left by 1 and add */
    pub fn sh1add_uw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b010, 0010000, rd, rs1, rs2)
    }
    /** shift unsigned word left by 2 and add */
    pub fn sh2add_uw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b100, 0010000, rd, rs1, rs2)
    }
    /** shift unsigned word left by 3 and add */
    pub fn sh3add_uw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b110, 0010000, rd, rs1, rs2)
    }
    /** shift-left unsigned word (immediate) */
    pub fn slli_uw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b001, rd, rs1, (imm & 0b111111) | 0b000010_000000);
    }
    //  ▲ Zba: address generation ▲
    //  ▼ Zbb: Basic bit-manipulation ▼
    /** count leading zero bits in word */
    pub fn clzw(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm32, 0b001, rd, rs1, 0b0110000_00000)
    }
    /** count trailing zero bits in word */
    pub fn ctzw(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm32, 0b001, rd, rs1, 0b0110000_00001)
    }
    /** count set bits in word */
    pub fn cpopw(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm32, 0b001, rd, rs1, 0b0110000_00010)
    }
    /** Zero-extend halfword */
    pub fn zext_h_64(&mut self, rd: Register, rs1: Register) {
        self.i(Op32, 0b100, rd, rs1, 0b0000100_00000)
    }
    /** rotate left word (register) */
    pub fn rolw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b001, 0b0110000, rd, rs1, rs2)
    }
    /** rotate right word (register) */
    pub fn rorw(&mut self, rd: Register, rs1: Register, rs2: Register) {
        self.r(Op32, 0b101, 0b0110000, rd, rs1, rs2)
    }
    /** rotate right word (immediate) */
    pub fn roriw(&mut self, rd: Register, rs1: Register, imm: i32) {
        self.i(OpImm32, 0b101, rd, rs1, (imm & 0b11111) | 0b0110000_00000);
    }
    /** byte-reverse register */
    pub fn rev8_64(&mut self, rd: Register, rs1: Register) {
        self.i(OpImm, 0b101, rd, rs1, 0b011010111000);
    }
    //  ▲ Zbb: Basic bit-manipulation ▲
    // ▲▲ RV64B Standard Extension (in addition to RV32B) ▲▲
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Condition {
    EQ,
    NE,
    GTU,
    LEU,
    GEU,
    LTU,
    GT,
    LE,
    GE,
    LT,
}

impl Condition {
    pub fn invert(self) -> Self {
        match self {
            Condition::EQ => Condition::NE,
            Condition::NE => Condition::EQ,
            Condition::GTU => Condition::LEU,
            Condition::LEU => Condition::GTU,
            Condition::GEU => Condition::LTU,
            Condition::LTU => Condition::GEU,
            Condition::GT => Condition::LE,
            Condition::LE => Condition::GT,
            Condition::GE => Condition::LT,
            Condition::LT => Condition::GE,
        }
    }
}
