use std::str::FromStr;

use linefeed::{Interface, ReadResult};
use nix::{
    sys::{ptrace::cont, wait::waitpid},
    unistd::Pid,
};
use strum_macros::EnumString;

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
}

pub struct Debugger {
    prog_name: String,
    pid: Pid,
}

impl Debugger {
    pub fn new(prog_name: String, pid: Pid) -> Self {
        Self { prog_name, pid }
    }

    pub fn continue_execution(&self) {
        cont(self.pid, None).unwrap();
        let _wait_status = waitpid(self.pid, None);
    }

    pub fn handle_command(&self, command: &String) {
        let command_line = command.split(" ").collect::<Vec<&str>>();
        let Some(command) = command_line.get(0) else {
            println!("{NO_COMMAND_PROVIDED_ERROR_MSG}");
            return;
        };

        if let Ok(ecommand) = Command::from_str(command) {
            match ecommand {
                Command::CONTINUE => self.continue_execution(),
                Command::EXIT => std::process::exit(0)
            }
        } else {
            println!("{NO_COMMAND_PROVIDED_ERROR_MSG}");
        }
    }

    pub fn run(&self) {
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
}
