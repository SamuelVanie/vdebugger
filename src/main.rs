use std::env;
use debugger::Debugger;
use nix::unistd::{fork, ForkResult, write};

pub mod debugger;

fn main() {

    if env::args().len() < 2 {
        eprintln!("No program name were specified");
        std::process::exit(-1);
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            // execute the debugger in the parent
            println!("Starting the debugging process {child}");
            let dbg = Debugger::new(env::args().nth(1).unwrap().into(), child);
            dbg.run();
        }
        Ok(ForkResult::Child) => {
            // execute the debugee in the child
            write(std::io::stdout(), "It's unsafe for me, the child to write directly on the stdout\n".as_bytes()).ok();
            unsafe { nix::libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}
