use linefeed::{Interface, ReadResult};
use nix::{sys::wait::waitpid, unistd::Pid};


pub struct Debugger {
    m_prog_name: String,
    m_pid: Pid
}

impl Debugger {
    pub fn new(m_prog_name: String, m_pid: Pid) -> Self {
        Self { m_prog_name, m_pid }
    }

    pub fn run(&self) {
        let wait_status = waitpid(self.m_pid, None);
        let reader = Interface::new("vdebugger").unwrap();
        println!("The program name is {}", self.m_prog_name);
        reader.set_prompt("vdebugger> ").unwrap();

        while let ReadResult::Input(input) = reader.read_line().unwrap() {
            println!("got input {:?}", input);
        }
    }
}
