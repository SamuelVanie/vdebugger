use std::{collections::HashMap, ffi::c_void, str::FromStr};

use linefeed::{Interface, ReadResult};
use nix::{
    sys::{ptrace::cont, wait::waitpid},
    unistd::Pid,
};
use strum_macros::EnumString;

use crate::{
    breakpoint::Breakpoint,
    register::{get_register_value, REGISTERS_DESCRIPTORS},
};
use crate::{
    breakpoint::RealPtraceOps,
    register::{get_register_from_name, set_register_value},
};

static NO_COMMAND_PROVIDED_ERROR_MSG: &str = r#"
No command or invalid command were provided
Try using one of the following:
1. continue
2. break 0xADDRESS
3. exit
"#;

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
enum Command {
    CONTINUE,
    REGISTER,
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
        let command_line: Vec<String> = command
            .split_whitespace()
            .map(|el| el.to_string())
            .collect();

        let Some(command) = command_line.get(0) else {
            println!("{NO_COMMAND_PROVIDED_ERROR_MSG}");
            return;
        };

        let arg1 = command_line.get(1);
        let arg2 = command_line.get(2);
        let arg3 = command_line.get(3);

        if let Ok(ecommand) = Command::from_str(command) {
            match ecommand {
                Command::CONTINUE => self.continue_execution(),
                Command::EXIT => std::process::exit(0),
                Command::BREAK => {
                    if arg1.is_some() {
                        self.set_breakpoint_at_address(str_to_c_void(arg1.unwrap()));
                    } else {
                        eprintln!("No address provided for the breakpoint");
                        return;
                    };
                }
                Command::REGISTER => {
                    if arg1.is_none() {
                        eprintln!("You cannot call the register command without any arguments");
                        return;
                    }
                    let arg1 = arg1.unwrap();
                    let arg1 = arg1.to_lowercase();
                    if arg1 == "dump" {
                        self.dump_registers();
                    } else if arg1 == "read" {
                        if arg2.is_none() {
                            eprintln!("This command requires a register name");
                            return;
                        }
                        let arg2 = arg2.unwrap();
                        let Some(reg) = get_register_from_name(arg2) else {
                            eprintln!("This register doesn't exist in the table");
                            return;
                        };
                        let Ok(val) = get_register_value(self.pid, reg) else {
                            eprintln!("Cannot get the value of this register");
                            return;
                        };
                        println!("{} -> {}", arg2, val);
                    } else if arg1 == "write" {
                        if arg2.is_none() {
                            eprintln!("This command requires a register name");
                            return;
                        }
                        let arg2 = arg2.unwrap();
                        if arg3.is_none() {
                            eprintln!(
                                "This command requires a value that will be set to the register"
                            );
                            return;
                        }
                        let arg3 = arg3.unwrap();

                        let Some(reg) = get_register_from_name(arg2) else {
                            eprintln!("This register doesn't exist in the table");
                            return;
                        };

                        let val = u64::from_str_radix(arg3.trim_start_matches("0x"), 16)
                            .expect("Failed to parse address");

                        set_register_value(self.pid, reg, val).unwrap();
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
            reader.add_history_unique(input.clone());
            self.handle_command(&input.clone());
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
