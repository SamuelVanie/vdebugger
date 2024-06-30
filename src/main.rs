use std::{env, ffi::CString};
use debugger::Debugger;
use nix::{sys::ptrace::traceme, unistd::{execv, fork, write, ForkResult}};

pub mod debugger;

fn main() {

    if env::args().len() < 2 {
        eprintln!("No program name were specified");
        std::process::exit(-1);
    }
    let program = env::args().nth(1).unwrap();
    let c_prog = CString::new(program.clone()).unwrap();
    let c_prog = c_prog.as_c_str();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            // execute the debugger in the parent
            println!("Starting the debugging process {child}");
            let dbg = Debugger::new(program, child);
            dbg.run();
        }
        Ok(ForkResult::Child) => {
            // execute the debugee in the child
            write(std::io::stdout(), "It's unsafe for me, the child to write directly on the stdout\n".as_bytes()).ok();
            traceme().unwrap();
            let _ = execv(c_prog, &[c_prog]);
            unsafe { nix::libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}
