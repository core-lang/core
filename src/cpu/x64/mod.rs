use cpu::x64::dwarfinfo::{DwarfInfo, MachineContext};
use ctxt::{Context, FctId, FctKind, get_ctxt};
use execstate::ExecState;
use jit::fct::CatchType;
use object::{Handle, Obj};
use os::MemMapTable;
use stacktrace::Stacktrace;

pub use self::param::*;
pub use self::reg::*;

pub mod dwarfinfo;
pub mod emit;
pub mod instr;
pub mod param;
pub mod reg;
pub mod trap;

pub fn handle_exception(exception: Handle<Obj>, es: &mut ExecState) -> bool {
    let mut pc : usize = es.pc;
    let mut fp : usize = es.regs[RBP.int() as usize];

    loop {
        let found = find_handler(exception, es, pc, fp);

        match found {
            HandlerFound::Yes => { return true; }
            HandlerFound::Stop => { return false; }
            HandlerFound::No => {
                if fp == 0 { return false; }
            }
        }

        pc = unsafe { *((fp + 8) as *const usize) };
        fp = unsafe { *(fp as *const usize) };
    }
}

#[derive(PartialEq, Eq, Debug)]
enum HandlerFound { Yes, No, Stop }

fn find_handler(exception: Handle<Obj>, es: &mut ExecState, pc: usize, fp: usize) -> HandlerFound {
    let ctxt = get_ctxt();
    let fct_id = {
        let code_map = ctxt.code_map.lock().unwrap();
        code_map.get(pc)
    };

    // println!("------------");
    // println!("find {:x}", pc);

    if let Some(fct_id) = fct_id {
        let fct = ctxt.fct_by_id(fct_id);

        if let FctKind::Source(ref src) = fct.kind {
            let src = src.lock().unwrap();

            if let Some(ref jit_fct) = src.jit_fct {
                let cls_id = exception.header().vtbl().class().id;

                for entry in &jit_fct.exception_handlers {
                    // println!("entry = {:x} to {:x} for {:?}",
                    //          entry.try_start, entry.try_end, entry.catch_type);

                    if entry.try_start < pc && pc <= entry.try_end
                        && (entry.catch_type == CatchType::Any
                            || entry.catch_type == CatchType::Class(cls_id)) {
                        if let Some(offset) = entry.offset {
                            let arg = (fp as isize + offset as isize) as usize;

                            unsafe {
                                *(arg as *mut usize) = exception.raw() as usize;
                            }
                        }

                        es.regs[RSP.int() as usize] = fp - src.stacksize() as usize;
                        es.regs[RBP.int() as usize] = fp;
                        es.pc = entry.catch;

                        return HandlerFound::Yes;

                    } else if pc > entry.try_end {
                        // exception handlers are sorted, no more possible handlers
                        // in this function

                        return HandlerFound::No;
                    }
                }
            }

            // exception can only bubble up in stacktrace if current function
            // is allowed to throw exceptions
            if !fct.ast.throws {
                return HandlerFound::Stop;
            }
        }
    }

    HandlerFound::No
}

pub fn get_rootset(ctxt: &Context) -> Vec<usize> {
    let mut rootset = Vec::new();
    let mut mctxt = MachineContext::new();

    unsafe { asm!("lea (%rip), $0": "=r"(mctxt.pc)) }
    unsafe { asm!("mov %rbp, $0": "=r"(mctxt.fp)) }
    unsafe { asm!("mov %rsp, $0": "=r"(mctxt.sp)) }

    let table = MemMapTable::new();

    while mctxt.fp != 0 {
        pop_frame(ctxt, &mut rootset, &table, &mut mctxt);
    }

    rootset
}

fn pop_frame(ctxt: &Context, rootset: &mut Vec<usize>, table: &MemMapTable,
             mctxt: &mut MachineContext) {
    let code_map = ctxt.code_map.lock().unwrap();
    let fct_id = code_map.get(mctxt.pc);

    if let Some(fct_id) = fct_id {
        determine_rootset(rootset, ctxt, mctxt, fct_id);

        unsafe {
            mctxt.pc = *((mctxt.fp + 8) as *const usize);
            mctxt.fp = *(mctxt.fp as *const usize);
        }

    } else if let Some(entry) = table.find_entry(mctxt.pc) {
        let pc = entry.address(mctxt.pc) as *const u8;

        let mut info = DwarfInfo::new(&entry.pathname, 17);
        let found = info.at(mctxt, pc);

        if !entry.pathname.contains("dora") {
            panic!("does this happen?");
        }

        if !found {
            panic!("no dwarf info found for pc.");
        }

    } else {
        panic!("pc does not point into valid memory area.");
    }
}

fn determine_rootset(rootset: &mut Vec<usize>, ctxt: &Context,
                     mctxt: &MachineContext, fct_id: FctId) {
    let fct = ctxt.fct_by_id(fct_id);

    if let FctKind::Source(ref src) = fct.kind {
        let src = src.lock().unwrap();
        let jit_fct = src.jit_fct.as_ref().expect("no jit information");
        let offset = mctxt.pc - (jit_fct.fct_ptr().raw() as usize);
        let gcpoint = jit_fct.gcpoint_for_offset(offset as i32).expect("no gcpoint");

        for &offset in &gcpoint.offsets {
            let addr = (mctxt.fp as isize + offset as isize) as usize;
            let obj = unsafe { *(addr as *const usize) };

            rootset.push(obj);
        }
    } else {
        panic!("should be FctKind::Source");
    }
}

pub fn get_stacktrace(ctxt: &Context, es: &ExecState) -> Stacktrace {
    let mut stacktrace = Stacktrace::new();
    determine_stack_entry(&mut stacktrace, ctxt, es.pc);

    let mut rbp = es.regs[reg::RBP.int() as usize];

    while rbp != 0 {
        let ra = unsafe { *((rbp + 8) as *const usize) };
        let cont = determine_stack_entry(&mut stacktrace, ctxt, ra);

        if !cont { break; }

        rbp = unsafe { *(rbp as *const usize) };
    }

    return stacktrace;
}

fn determine_stack_entry(stacktrace: &mut Stacktrace, ctxt: &Context, pc: usize) -> bool {
    let code_map = ctxt.code_map.lock().unwrap();
    let fct_id = code_map.get(pc);

    if let Some(fct_id) = fct_id {
        let mut lineno = 0;
        let fct = ctxt.fct_by_id(fct_id);
        if let FctKind::Source(ref src) = fct.kind {
            let src = src.lock().unwrap();
            let jit_fct = src.jit_fct.as_ref().unwrap();
            let offset = pc - (jit_fct.fct_ptr().raw() as usize);
            lineno = jit_fct.lineno_for_offset(offset as i32);

            if lineno == 0 {
                panic!("lineno not found for program point");
            }
        }

        stacktrace.push_entry(fct_id, lineno);

        true
    } else {
        // only continue if we still haven't reached jitted functions
        stacktrace.len() == 0
    }
}
