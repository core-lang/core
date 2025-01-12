use crate::vm::{Code, VM};
use core_parser::interner::Name;

#[cfg(target_os = "linux")]
pub fn register_with_perf(code: &Code, vm: &VM, name: Name) {
    use std::fs::OpenOptions;
    use std::io::prelude::*;

    let pid = unsafe { libc::getpid() };
    let fname = format!("/tmp/perf-{}.map", pid);

    let mut options = OpenOptions::new();
    let mut file = options.create(true).append(true).open(&fname).unwrap();

    let code_start = code.instruction_start().to_usize();
    let code_end = code.instruction_end().to_usize();
    let name = vm.interner.str(name);

    let line = format!(
        "{:x} {:x} core::{}\n",
        code_start,
        code_end - code_start,
        name
    );
    file.write_all(line.as_bytes()).unwrap();
}

#[cfg(not(target_os = "linux"))]
pub fn register_with_perf(_: &Code, _: &VM, _: Name) {
    // nothing to do
}
