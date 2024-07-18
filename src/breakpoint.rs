use std::ffi::{c_long, c_void};

use nix::{sys::ptrace, unistd::Pid};

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
    saved_data: i64,
    enabled: bool,
    ptrace_ops: T,
}

impl<T: PtraceOps> Breakpoint<T> {
    pub fn new(pid: Pid, addr: *mut c_void, ptrace_ops: T) -> Self {
        Self {
            pid,
            addr,
            saved_data: 0i64,
            enabled: false,
            ptrace_ops,
        }
    }

    pub fn enable(&mut self) {
        let Ok(old_line) = self.ptrace_ops.read(self.pid, self.addr) else {
            std::process::exit(-1);
        };

        self.saved_data = old_line & 0xff;
        let int3: i64 = 0xcc; // the int 3 interruption signal instruction
        let data_with_int3_added = (old_line & !0xff) | int3; // set the bottom byte of the address to int3 (0xcc)

        let _ = self
            .ptrace_ops
            .write(self.pid, self.addr, data_with_int3_added);

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
    use super::*;
    use lazy_static::lazy_static;
    use std::sync::Mutex;

    lazy_static! {
        static ref ADDR: Mutex<usize> = Mutex::new(0x1000);
    }

    #[test]
    fn test_enable_disable_data_preservation() {
        let pid = Pid::from_raw(1234); // Dummy PID
        let addr = *ADDR.lock().unwrap() as *mut c_void;
        let mut mock_ops = MockPtraceOps::new();
        let initial_data = 0x1122334455667788i64;
        let expected_data_after_enable = 0x11223344556677CCi64;

        // Expectations for enable
        mock_ops
            .expect_read()
            .withf(move |&p, &a| p == pid && a as usize == *ADDR.lock().unwrap())
            .times(1)
            .return_const(Ok(initial_data));


        mock_ops
            .expect_write()
            .withf(move |&p, &a, &d| {
                p == pid && a as usize == *ADDR.lock().unwrap() && d == expected_data_after_enable
            })
            .times(1)
            .return_const(());

        // Expectations for disable
        mock_ops
            .expect_read()
            .withf(move |&p, &a| p == pid && a as usize == *ADDR.lock().unwrap())
            .times(1)
            .return_const(Ok(expected_data_after_enable));
        mock_ops
            .expect_write()
            .withf(move |&p, &a, &d| {
                p == pid && a as usize == *ADDR.lock().unwrap() && d == initial_data
            })
            .times(1)
            .return_const(());

        let mut breakpoint = Breakpoint::new(pid, addr, mock_ops);

        // Enable the breakpoint
        breakpoint.enable();

        // Check that the breakpoint is enabled
        assert!(breakpoint.enabled);
        assert_eq!(breakpoint.saved_data, 0x88);

        breakpoint.disable();
        assert!(!breakpoint.enabled);
    }
}
