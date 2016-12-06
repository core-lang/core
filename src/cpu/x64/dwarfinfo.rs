extern "C" {
    fn dwarfinfo_init(name: *const u8, reg_table_size: i32) -> *const u8;
    fn dwarfinfo_at(info: *const u8, mctxt: *mut MachineContext, pc: *const u8) -> bool;
    fn dwarfinfo_free(info: *const u8);
}

pub struct DwarfInfo {
    cptr: *const u8,
}

impl DwarfInfo {
    pub fn new(pathname: &str, reg_table_size: i32) -> DwarfInfo {
        unsafe {
            DwarfInfo {
                cptr: dwarfinfo_init(pathname.as_ptr(), reg_table_size)
            }
        }
    }

    pub fn at(&mut self, mctxt: &mut MachineContext, pc: *const u8) -> bool {
        unsafe {
            dwarfinfo_at(self.cptr, mctxt as *mut MachineContext, pc)
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

#[repr(C)]
pub struct MachineContext {
    pub pc: usize,
    pub fp: usize,
    pub sp: usize,
}

impl MachineContext {
    pub fn new() -> MachineContext {
        MachineContext {
            pc: 0,
            fp: 0,
            sp: 0,
        }
    }
}
