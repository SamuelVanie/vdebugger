use std::{collections::HashMap, ffi::c_void, str::FromStr};

use linefeed::{Interface, ReadResult};
use nix::{
    sys::{ptrace::cont, wait::waitpid},
    unistd::Pid,
};
use strum_macros::EnumString;

use crate::breakpoint::RealPtraceOps;
use crate::{
    breakpoint::Breakpoint,
    register::{get_register_value, REGISTERS_DESCRIPTORS},
};

static NO_COMMAND_PROVIDED_ERROR_MSG: &str = r#"
No command or invalid command were provided
Try using one of the following:
1. continue
2. exit
"#;

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
enum Command {
    CONTINUE,
    EXIT,
    BREAK,
}

pub struct Debugger {
    prog_name: String,
    pid: Pid,
    breakpoints: HashMap<*mut c_void, Breakpoint<RealPtraceOps>>,
}

fn str_to_c_void(s: &str) -> *mut c_void {
    let address =
        usize::from_str_radix(s.trim_start_matches("0x"), 16).expect("Failed to parse address");
    println!("The address is : {}", address);

    address as *mut c_void
}

impl Debugger {
    pub fn new(prog_name: String, pid: Pid) -> Self {
        Self {
            prog_name,
            pid,
            breakpoints: HashMap::new(),
        }
    }

    pub fn continue_execution(&self) {
        cont(self.pid, None).unwrap();
        let _wait_status = waitpid(self.pid, None);
    }

    pub fn handle_command(&mut self, command: &String) {
        let command_line = command.split(" ").collect::<Vec<&str>>();
        let Some(command) = command_line.get(0) else {
            println!("{NO_COMMAND_PROVIDED_ERROR_MSG}");
            return;
        };

        let args = command_line.get(1);

        if let Ok(ecommand) = Command::from_str(command) {
            match ecommand {
                Command::CONTINUE => self.continue_execution(),
                Command::EXIT => std::process::exit(0),
                Command::BREAK => {
                    if args.is_some() {
                        let args = args.unwrap();
                        self.set_breakpoint_at_address(str_to_c_void(args));
                    } else {
                        eprintln!("No address provided for the breakpoint");
                    }
                }
            }
        } else {
            println!("{NO_COMMAND_PROVIDED_ERROR_MSG}");
        }
    }

    pub fn run(&mut self) {
        let _wait_status = waitpid(self.pid, None);
        let reader = Interface::new("vdebugger").unwrap();
        println!("The program name is {}", self.prog_name);
        reader.set_prompt("vdebugger> ").unwrap_or_else(|_| {});

        while let ReadResult::Input(input) = reader.read_line().unwrap_or_else(|_| ReadResult::Eof)
        {
            self.handle_command(&input);
            reader.add_history_unique(input.clone());
            println!("got input {:?}", input);
        }
    }

    pub fn set_breakpoint_at_address(&mut self, address: *mut c_void) {
        println!("Set breakpoint at address {:p}", address);
        let mut b = Breakpoint::new(self.pid, address, RealPtraceOps);
        b.enable();
        self.breakpoints.insert(address, b);
    }

    pub fn dump_registers(&self) {
        REGISTERS_DESCRIPTORS.iter().for_each(|&desc| {
            let val = get_register_value(self.pid, desc.r).unwrap();
            println!("{} 0x{:016x}", desc.name, val);
        });
    }
}
