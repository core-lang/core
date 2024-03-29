use std::convert::From;

#[cfg(target_arch = "x86_64")]
pub use self::x64::*;

#[cfg(target_arch = "x86_64")]
pub mod x64;

#[cfg(target_arch = "aarch64")]
pub use self::arm64::*;

#[cfg(target_arch = "aarch64")]
pub mod arm64;

#[cfg(target_arch = "riscv64")]
pub use self::rv64::*;

#[cfg(target_arch = "riscv64")]
pub mod rv64;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Reg(pub u8);

impl From<Reg> for u32 {
    fn from(reg: Reg) -> u32 {
        reg.0 as u32
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FReg(pub u8);

impl From<FReg> for u32 {
    fn from(reg: FReg) -> u32 {
        reg.0 as u32
    }
}
