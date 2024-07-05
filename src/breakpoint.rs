use std::ffi::c_void;

use nix::{
    sys::ptrace::{read, write},
    unistd::Pid,
};

pub struct Breakpoint {
    pid: Pid,
    addr: *mut c_void,
    saved_data: i8,
    enabled: bool,
}

impl Breakpoint {
    pub fn new(pid: Pid, addr: *mut c_void) -> Self {
        Self {
            pid,
            addr,
            saved_data: 0i8,
            enabled: false,
        }
    }

    pub fn enable(&mut self) {
        let Ok(old_line) = read(self.pid, self.addr) else {
            std::process::exit(-1);
        };

        self.saved_data = (old_line & 0xff) as i8;
        let int3: i64 = 0xcc; // the int 3 interruption signal instruction
        let data_with_int3_added = (old_line & !0xff) | int3; // set the bottom byte of the address to int3 (0xcc)

        let _ = write(self.pid, self.addr, data_with_int3_added).unwrap_or_else(|_| {
            eprintln!("Cannot write at the given address the user line of code is loss");
            std::process::exit(-1);
        });

        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let Ok(line) = read(self.pid, self.addr) else {
            eprintln!("Couldn't remove breakpoint at an adress that doesn't contain any data");
            std::process::exit(-1);
        };

        let restored_line = (line & !0xff) | self.saved_data as i64;
 
        let _ = write(self.pid, self.addr, restored_line).unwrap_or_else(|_| {
            eprintln!("Cannot write at the given address the user line of code is loss");
            std::process::exit(-1);
        });

        self.enabled = false;
    }
}
