use std::{collections::HashMap, ffi::c_void, str::FromStr};

use linefeed::{Interface, ReadResult};
use nix::{
    sys::{ptrace::{self, cont}, wait::waitpid},
    unistd::Pid,
};
use strum_macros::EnumString;

use crate::{
    breakpoint::Breakpoint,
    register::{get_register_value, Reg, REGISTERS_DESCRIPTORS},
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
3. memory [read/write] 0xADDRESS
4. register [dump/read/write] [0xADDRESS]
5. exit
"#;

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
enum Command {
    CONTINUE,
    REGISTER,
    MEMORY,
    EXIT,
    BREAK,
}

pub struct Debugger {
    prog_name: String,
    pid: Pid,
    breakpoints: HashMap<u64, Breakpoint<RealPtraceOps>>,
}

fn str_addr_to_c_void(s: &str) -> *mut c_void {
    let address =
        i64::from_str_radix(s.trim_start_matches("0x"), 16).expect("Failed to parse address");
    println!("The address is : {}", address);

    address as *mut c_void
}

fn str_to_addr(s: &str) -> i64 {
    i64::from_str_radix(s.trim_start_matches("0x"), 16).expect("Failed to parse address")
}

fn str_to_reg_value(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("Failed to parse the string to an u64 value")
}


impl Debugger {
    pub fn new(prog_name: String, pid: Pid) -> Self {
        Self {
            prog_name,
            pid,
            breakpoints: HashMap::new(),
        }
    }

    pub fn continue_execution(&mut self) {
        self.step_over_breakpoint();
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
                        self.set_breakpoint_at_address(arg1.unwrap());
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

                        let val = str_to_reg_value(arg3);
                        set_register_value(self.pid, reg, val).unwrap();
                    }
                },
                Command::MEMORY => {
                    if arg1.is_none() {
                        eprintln!("Command memory cannot be called without any arguments");
                        return;
                    }
                    let arg1 = arg1.unwrap();
                    if arg2.is_none() {
                        eprintln!("You should precise the address you want to manipulate");
                        return;
                    }
                    let arg2 = arg2.unwrap();

                    if arg1 == "read" {
                        let Ok(val) = ptrace::read(self.pid, str_addr_to_c_void(arg2)) else {
                            eprintln!("Cannot read data at this memory address");
                            return;
                        };
                        println!("{} --> {}", arg2, val);
                    } else if arg1 == "write" {
                        if arg3.is_none() {
                            eprintln!("You should precise the value that will be set to the register");
                            return;
                        }
                        let arg3 = arg3.unwrap();
                        let val = str_to_addr(arg3);
                        let Ok(_) = ptrace::write(self.pid, str_addr_to_c_void(arg2), val) else {
                            eprintln!("Cannot write to that address");
                            return;
                        };
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

    pub fn set_breakpoint_at_address(&mut self, address: &str) {
        println!("Set breakpoint at address {}", address);
        let addr = str_addr_to_c_void(address);
        let mut b = Breakpoint::new(self.pid, addr, RealPtraceOps);
        b.enable();
        self.breakpoints.insert(str_to_reg_value(address), b);
    }



    fn step_over_breakpoint(&mut self) {
        let current_line = self.get_pc() - 1; // because execution will be past the breakpoint
        if self.breakpoints.contains_key(&current_line) {
            let bp = self.breakpoints.get_mut(&current_line).unwrap();

            if bp.enabled {
                let prev = current_line;
                self.set_pc(prev);

                let bp = self.breakpoints.get_mut(&current_line).unwrap();
                bp.disable();

                let Ok(_) = ptrace::step(self.pid, None) else {
                    eprintln!("Cannot go to the next line");
                    std::process::exit(-1);
                };

                let Ok(_) = waitpid(self.pid, None) else {
                    eprintln!("Cannot communicate with the debuggee process");
                    std::process::exit(-1);
                };

                bp.enable();
            }
        }
    }

    pub fn dump_registers(&self) {
        REGISTERS_DESCRIPTORS.iter().for_each(|&desc| {
            let Ok(val) = get_register_value(self.pid, desc.r) else {
                eprintln!("Cannot get value of the register {:?}. Verify that the debuggee's process hasn't ended", desc.r);
                std::process::exit(-1)
            };
            println!("{} 0x{:016x}", desc.name, val);
        });
    }

    fn get_pc(&self) -> u64 {
        let Ok(pc) = get_register_value(self.pid, Reg::Rip) else {
            eprintln!("Cannot get the program counter");
            std::process::exit(-1);
        };
        pc
    }

    fn set_pc(&self, pc: u64) {
        let Ok(_) = set_register_value(self.pid, Reg::Rip, pc) else {
            eprintln!("Cannot move the program counter");
            std::process::exit(-1);
        };
    }
}
