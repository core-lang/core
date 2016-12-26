use std;
use libc;

use baseline;
use baseline::fct::BailoutInfo;
use cpu;
use ctxt::{Context, CTXT, FctId, get_ctxt};
use execstate::ExecState;
use os_cpu::*;
use stacktrace::{handle_exception, get_stacktrace};

pub fn register_signals(ctxt: &Context) {
    unsafe {
        let ptr = ctxt as *const Context as *const u8;
        CTXT = Some(ptr);

        let mut sa: libc::sigaction = std::mem::uninitialized();

        sa.sa_sigaction = handler as usize;
        libc::sigemptyset(&mut sa.sa_mask as *mut libc::sigset_t);
        sa.sa_flags = libc::SA_SIGINFO;

        if libc::sigaction(libc::SIGSEGV,
                           &sa as *const libc::sigaction,
                           0 as *mut libc::sigaction) == -1 {
            libc::perror("sigaction for SIGSEGV failed".as_ptr() as *const libc::c_char);
        }

        if libc::sigaction(libc::SIGILL,
                           &sa as *const libc::sigaction,
                           0 as *mut libc::sigaction) == -1 {
            libc::perror("sigaction for SIGILL failed".as_ptr() as *const libc::c_char);
        }
    }
}

// signal handler function
fn handler(signo: libc::c_int, _: *const u8, ucontext: *const u8) {
    let mut es = read_execstate(ucontext);
    let ctxt = get_ctxt();

    if let Some(trap) = detect_trap(signo as i32, &es) {
        match trap {
            Trap::COMPILER => compile_request(ctxt, &mut es, ucontext),

            Trap::DIV0 => {
                println!("division by 0");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(101);
                }
            }

            Trap::ASSERT => {
                println!("assert failed");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(101);
                }
            }

            Trap::INDEX_OUT_OF_BOUNDS => {
                println!("array index out of bounds");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(102);
                }
            }

            Trap::NIL => {
                println!("nil check failed");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(103);
                }
            }

            Trap::THROW => {
                let handler_found = handle_exception(&mut es);

                if handler_found {
                    write_execstate(&es, ucontext as *mut u8);
                } else {
                    println!("uncaught exception");
                    unsafe {
                        libc::_exit(104);
                    }
                }
            }

            Trap::CAST => {
                println!("cast failed");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(105);
                }
            }

            Trap::UNEXPECTED => {
                println!("unexpected exception");
                let stacktrace = get_stacktrace(ctxt, &es);
                stacktrace.dump(ctxt);
                unsafe {
                    libc::_exit(106);
                }
            }
        }

        // could not recognize trap -> crash vm
    } else {
        println!("error: trap not detected (signal {}).", signo);
        unsafe {
            libc::_exit(1);
        }
    }
}

fn compile_request(ctxt: &Context, es: &mut ExecState, ucontext: *const u8) {
    let fct_id = {
        let code_map = ctxt.code_map.lock().unwrap();
        code_map.get(es.pc as *const u8)
    };

    if let Some(fct_id) = fct_id {
        let mut sfi = cpu::sfi_from_execution_state(es);

        ctxt.use_sfi(&mut sfi, || {
            let jit_fct = baseline::generate(ctxt, fct_id);
            let fct = ctxt.fct_by_id(fct_id);

            if fct.is_virtual() {
                patch_vtable_call(ctxt, es, fct_id, jit_fct);
            } else {
                patch_fct_call(ctxt, es, jit_fct);
            }

            write_execstate(es, ucontext as *mut u8);
        });
    } else {
        println!("error: code not found for address {:x}", es.pc);
        unsafe {
            libc::_exit(200);
        }
    }
}

fn patch_vtable_call(ctxt: &Context, es: &mut ExecState, fid: FctId, fct_ptr: *const u8) {
    let fct = ctxt.fct_by_id(fid);
    let vtable_index = fct.vtable_index.unwrap();
    let cls_id = fct.owner_class.unwrap();

    let cls = ctxt.cls_by_id(cls_id);
    let vtable = cls.vtable.as_ref().unwrap();

    let methodtable = vtable.table_mut();
    methodtable[vtable_index as usize] = fct_ptr as usize;

    // execute fct call again
    es.pc = fct_ptr as usize;
}

pub fn patch_fct_call(ctxt: &Context, es: &mut ExecState, fct_ptr: *const u8) {
    // get return address from top of stack
    let mut ra = cpu::ra_from_execstate(es);

    let fct_id = {
        let code_map = ctxt.code_map.lock().unwrap();
        code_map.get(ra as *const u8).expect("return address not found")
    };

    let fct = ctxt.fct_by_id(fct_id);
    let src = fct.src();
    let mut src = src.lock().unwrap();
    let jit_fct = src.jit_fct.as_ref().expect("jitted fct not found");

    let offset = ra - jit_fct.fct_ptr() as usize;
    let bailout = jit_fct.bailouts.get(offset as i32).expect("bailout info not found");

    // get address of function pointer
    let disp = match bailout {
        &BailoutInfo::Compile(disp) => disp as isize,
    };

    let fct_addr: *mut usize = (ra as isize - disp) as *mut _;

    // write function pointer
    unsafe {
        *fct_addr = fct_ptr as usize;
    }

    // execute fct call again
    es.pc = fct_ptr as usize;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    COMPILER,
    DIV0,
    ASSERT,
    INDEX_OUT_OF_BOUNDS,
    NIL,
    THROW,
    CAST,
    UNEXPECTED,
}

impl Trap {
    pub fn int(self) -> u32 {
        match self {
            Trap::COMPILER => 0,
            Trap::DIV0 => 1,
            Trap::ASSERT => 2,
            Trap::INDEX_OUT_OF_BOUNDS => 3,
            Trap::NIL => 4,
            Trap::THROW => 5,
            Trap::CAST => 6,
            Trap::UNEXPECTED => 7,
        }
    }

    pub fn from(value: u32) -> Option<Trap> {
        match value {
            0 => Some(Trap::COMPILER),
            1 => Some(Trap::DIV0),
            2 => Some(Trap::ASSERT),
            3 => Some(Trap::INDEX_OUT_OF_BOUNDS),
            4 => Some(Trap::NIL),
            5 => Some(Trap::THROW),
            6 => Some(Trap::CAST),
            7 => Some(Trap::UNEXPECTED),
            _ => None,
        }
    }
}
