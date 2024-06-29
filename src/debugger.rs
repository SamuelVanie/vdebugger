use std::str::FromStr;

use linefeed::{Interface, ReadResult};
use nix::{sys::{ptrace::cont, wait::waitpid}, unistd::Pid};
use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
enum Command {
    CONTINUE,
}

pub struct Debugger {
    m_prog_name: String,
    m_pid: Pid,
}

impl Debugger {
    pub fn new(m_prog_name: String, m_pid: Pid) -> Self {
        Self { m_prog_name, m_pid }
    }

    pub fn continue_execution(&self) {
        cont(self.m_pid, None).unwrap();
        let _wait_status = waitpid(self.m_pid, None);
    }

    pub fn handle_command(&self, command: &String) {
        let command_line = command.split(" ").collect::<Vec<&str>>();
        let command = command_line.get(0).unwrap();

        if let Ok(ecommand) = Command::from_str(command) {
            match ecommand {
                Command::CONTINUE => self.continue_execution(),
            }
        } else {
            println!("Invalid command entered");
            unsafe { nix::libc::_exit(0) };
        }
    }

    pub fn run(&self) {
        let _wait_status = waitpid(self.m_pid, None);
        let reader = Interface::new("vdebugger").unwrap();
        println!("The program name is {}", self.m_prog_name);
        reader.set_prompt("vdebugger> ").unwrap();

        while let ReadResult::Input(input) = reader.read_line().unwrap() {
            self.handle_command(&input);
            reader.add_history_unique(input.clone());
            println!("got input {:?}", input);
        }
    }
}
