use std::ffi::{c_void, c_long};

use nix::{
    sys::ptrace,
    unistd::Pid,
};

pub struct RealPtraceOps;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait PtraceOps {
    fn read(&self, pid: Pid, addr: *mut c_void) -> Result<i64, nix::Error>;
    fn write(&self, pid: Pid, addr: *mut c_void, data: i64);  
}

impl PtraceOps for RealPtraceOps {
    fn read(&self, pid: Pid, addr: *mut c_void) -> Result<i64, nix::Error> {
        ptrace::read(pid, addr)
    }

    fn write(&self, pid: Pid, addr: *mut c_void, data: i64) {
        ptrace::write(pid, addr, data as c_long).expect("Cannot write to this address");
    }
}

pub struct Breakpoint<T: PtraceOps> {
    pid: Pid,
    addr: *mut c_void,
    saved_data: i8,
    enabled: bool,
    ptrace_ops: T,
}

impl<T: PtraceOps> Breakpoint<T> {
    pub fn new(pid: Pid, addr: *mut c_void, ptrace_ops: T) -> Self {
        Self {
            pid,
            addr,
            saved_data: 0i8,
            enabled: false,
            ptrace_ops,
        }
    }

    pub fn enable(&mut self) {
        let Ok(old_line) = self.ptrace_ops.read(self.pid, self.addr) else {
            std::process::exit(-1);
        };

        self.saved_data = (old_line & 0xff) as i8;
        let int3: i64 = 0xcc; // the int 3 interruption signal instruction
        let data_with_int3_added = (old_line & !0xff) | int3; // set the bottom byte of the address to int3 (0xcc)

        let _ = self.ptrace_ops.write(self.pid, self.addr, data_with_int3_added);

        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let Ok(line) = self.ptrace_ops.read(self.pid, self.addr) else {
            eprintln!("Couldn't remove breakpoint at an adress that doesn't contain any data");
            std::process::exit(-1);
        };

        let restored_line = (line & !0xff) | self.saved_data as i64;
 
        let _ = self.ptrace_ops.write(self.pid, self.addr, restored_line);

        self.enabled = false;
    }
}


#[cfg(test)]
mod test {
}
