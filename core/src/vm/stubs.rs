use crate::compiler::core_entry_stub;
use crate::compiler::core_exit_stubs::{self, NativeFct, NativeFctKind};
use crate::compiler::lazy_compilation_stub;
use crate::gc::Address;
use crate::language::ty::SourceType;
use crate::safepoint;
use crate::stdlib;
use crate::vm::VM;

pub struct Stubs {
    compile: Option<Address>,
    core_entry: Option<Address>,
    trap: Option<Address>,
    stack_overflow: Option<Address>,
    safepoint: Option<Address>,
}

impl Stubs {
    pub fn new() -> Stubs {
        Stubs {
            compile: None,
            core_entry: None,
            trap: None,
            stack_overflow: None,
            safepoint: None,
        }
    }

    pub fn lazy_compilation(&self) -> Address {
        self.compile.expect("uninitialized field")
    }

    pub fn core_entry(&self) -> Address {
        self.core_entry.expect("uninitialized field")
    }

    pub fn trap(&self) -> Address {
        self.trap.expect("uninitialized field")
    }

    pub fn stack_overflow(&self) -> Address {
        self.stack_overflow.expect("uninitialized field")
    }

    pub fn safepoint(&self) -> Address {
        self.safepoint.expect("uninitialized field")
    }
}

pub fn setup_stubs(vm: &mut VM) {
    vm.stubs.core_entry = Some(core_entry_stub::install(vm).instruction_start());

    let ifct = NativeFct {
        fctptr: Address::from_ptr(stdlib::trap as *const u8),
        args: &[SourceType::Int32],
        return_type: SourceType::Unit,
        desc: NativeFctKind::TrapStub,
    };
    let code = core_exit_stubs::generate(vm, ifct, false);
    vm.stubs.trap = Some(code.instruction_start());

    vm.stubs.compile = Some(lazy_compilation_stub::generate(vm).instruction_start());

    let ifct = NativeFct {
        fctptr: Address::from_ptr(safepoint::stack_overflow as *const u8),
        args: &[],
        return_type: SourceType::Unit,
        desc: NativeFctKind::GuardCheckStub,
    };
    let code = core_exit_stubs::generate(vm, ifct, false);
    vm.stubs.stack_overflow = Some(code.instruction_start());

    let ifct = NativeFct {
        fctptr: Address::from_ptr(safepoint::safepoint_slow as *const u8),
        args: &[],
        return_type: SourceType::Unit,
        desc: NativeFctKind::SafepointStub,
    };
    let code = core_exit_stubs::generate(vm, ifct, false);
    vm.stubs.safepoint = Some(code.instruction_start());
}
