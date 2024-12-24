use crate::driver::cmd::AsmSyntax;
use crate::language::sem_analysis::FctDefinition;
use crate::language::ty::SourceTypeArray;
use crate::vm::{Code, VM};

pub fn supported() -> bool {
    false
}

pub fn disassemble(
    _vm: &VM,
    _fct: &FctDefinition,
    _type_params: &SourceTypeArray,
    _code: &Code,
    _asm_syntax: AsmSyntax,
) {
    unreachable!();
}
