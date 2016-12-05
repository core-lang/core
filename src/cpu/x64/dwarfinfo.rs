use libc::c_char;

use std::env;
use std::ffi::CString;
use std::os::unix::ffi::OsStringExt;

extern "C" {
    fn dwarfinfo_init(name: *const c_char, reg_table_size: i32) -> *const u8;
    fn dwarfinfo_at(info: *const u8, pc: *const u8);
    fn dwarfinfo_free(info: *const u8);
}

pub struct DwarfInfo {
    cptr: *const u8,
}

impl DwarfInfo {
    fn new(reg_table_size: i32) -> DwarfInfo {
        let name = env::current_exe().expect("Couldn't get full exe name.");
        let osstr = name.into_os_string();
        let cstr = CString::new(osstr.into_vec()).expect("Couldn't create CString.");

        unsafe {
            DwarfInfo {
                cptr: dwarfinfo_init(cstr.as_ptr(), reg_table_size)
            }
        }
    }
}

impl Drop for DwarfInfo {
    fn drop(&mut self) {
        unsafe {
            dwarfinfo_free(self.cptr);
        }
    }
}