use debugger::Debugger;

use nix::{
    sys::{personality::{self, Persona}, ptrace::traceme},
    unistd::{execv, fork, write, ForkResult},
};
use std::{env, ffi::CString};

pub mod debugger;
pub mod breakpoint;

fn main() {
    if env::args().len() < 2 {
        eprintln!("No program name were specified");
        std::process::exit(-1);
    }

    let Some(program) = env::args().nth(1) else {
        eprintln!("Error the program name should be given as program arg");
        std::process::exit(-1);
    };

    let Ok( c_prog ) = CString::new(program.clone()) else {
        eprintln!("Could not use the program name given");
        std::process::exit(-1);
    };

    let c_prog = c_prog.as_c_str();
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            // execute the debugger in the parent
            println!("Starting the debugging process {child}");
            let mut dbg = Debugger::new(program, child);
            dbg.run();
        }
        Ok(ForkResult::Child) => {
            // execute the debugee in the child
            write(
                std::io::stdout(),
                "It's unsafe for me, the child to write directly on the stdout\n".as_bytes(),
            )
                .ok();
            
            // Disable address space randomization
            personality::set(Persona::ADDR_NO_RANDOMIZE).unwrap_or_else(|_| {
                eprintln!("Cannot remove address randomization from the child process");
                std::process::exit(-1);
            });
            
            traceme().unwrap_or_else(|_| {
                eprintln!("Cannot trace the child process");
                std::process::exit(-1);
            });
            
            let _ = execv(c_prog, &[c_prog]);
            std::process::exit(0);
        }
        Err(_) => println!("Fork failed"),
    }
}
