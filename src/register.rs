use nix::{sys::ptrace, unistd::Pid};
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Reg {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rdi,
    Rsi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Rip,
    Rflags,
    Cs,
    OrigRax,
    FsBase,
    GsBase,
    Fs,
    Gs,
    Ss,
    Ds,
    Es,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RegDescriptor {
    pub r: Reg,
    dwarf_r: i32,
    pub name: &'static str,
}

pub static REGISTERS_DESCRIPTORS: &[RegDescriptor] = &[
    RegDescriptor {
        r: Reg::Rax,
        dwarf_r: 0,
        name: "rax",
    },
    RegDescriptor {
        r: Reg::Rbx,
        dwarf_r: 3,
        name: "rbx",
    },
    RegDescriptor {
        r: Reg::R15,
        dwarf_r: 15,
        name: "r15",
    },
    RegDescriptor {
        r: Reg::R14,
        dwarf_r: 14,
        name: "r14",
    },
    RegDescriptor {
        r: Reg::R13,
        dwarf_r: 13,
        name: "r13",
    },
    RegDescriptor {
        r: Reg::R12,
        dwarf_r: 12,
        name: "r12",
    },
    RegDescriptor {
        r: Reg::Rbp,
        dwarf_r: 6,
        name: "rbp",
    },
    RegDescriptor {
        r: Reg::R11,
        dwarf_r: 11,
        name: "r11",
    },
    RegDescriptor {
        r: Reg::R10,
        dwarf_r: 10,
        name: "r10",
    },
    RegDescriptor {
        r: Reg::R9,
        dwarf_r: 9,
        name: "r9",
    },
    RegDescriptor {
        r: Reg::R8,
        dwarf_r: 8,
        name: "r8",
    },
    RegDescriptor {
        r: Reg::Rcx,
        dwarf_r: 2,
        name: "rcx",
    },
    RegDescriptor {
        r: Reg::Rdx,
        dwarf_r: 1,
        name: "rdx",
    },
    RegDescriptor {
        r: Reg::Rsi,
        dwarf_r: 4,
        name: "rsi",
    },
    RegDescriptor {
        r: Reg::Rdi,
        dwarf_r: 5,
        name: "rdi",
    },
    RegDescriptor {
        r: Reg::OrigRax,
        dwarf_r: -1,
        name: "orig_rax",
    },
    RegDescriptor {
        r: Reg::Rip,
        dwarf_r: -1,
        name: "rip",
    },
    RegDescriptor {
        r: Reg::Cs,
        dwarf_r: 51,
        name: "cs",
    },
    RegDescriptor {
        r: Reg::Rflags,
        dwarf_r: 49,
        name: "eflags",
    },
    RegDescriptor {
        r: Reg::Rsp,
        dwarf_r: 7,
        name: "rsp",
    },
    RegDescriptor {
        r: Reg::Ss,
        dwarf_r: 52,
        name: "ss",
    },
    RegDescriptor {
        r: Reg::FsBase,
        dwarf_r: 58,
        name: "fs_base",
    },
    RegDescriptor {
        r: Reg::GsBase,
        dwarf_r: 59,
        name: "gs_base",
    },
    RegDescriptor {
        r: Reg::Ds,
        dwarf_r: 53,
        name: "ds",
    },
    RegDescriptor {
        r: Reg::Es,
        dwarf_r: 50,
        name: "es",
    },
    RegDescriptor {
        r: Reg::Fs,
        dwarf_r: 54,
        name: "fs",
    },
    RegDescriptor {
        r: Reg::Gs,
        dwarf_r: 55,
        name: "gs",
    },
];

pub fn get_register_name(r: Reg) -> Option<&'static str> {
    REGISTERS_DESCRIPTORS
        .iter()
        .find(|&desc| desc.r == r)
        .map(|desc| desc.name)
}

pub fn get_register_from_name(name: &String) -> Option<Reg> {
    Reg::from_str(name).ok()
}

pub fn get_register_value_from_dwarf_register(pid: Pid, reg_num: i32) -> Result<u64, nix::Error> {
    let Some(reg) = REGISTERS_DESCRIPTORS
        .iter()
        .find(|&desc| desc.dwarf_r == reg_num)
        .map(|desc| desc.r.clone())
    else {
        println!("Unknow dwarf register number");
        return Err(nix::Error::ENODATA);
    };

    get_register_value(pid, reg)
}

pub fn get_register_value(pid: Pid, r: Reg) -> Result<u64, nix::Error> {
    let regs = ptrace::getregs(pid).unwrap_or_else(|_| {
        eprintln!("Cannot get this process' registers");
        std::process::exit(-1);
    });

    let val = match r {
        Reg::Rax => regs.rax,
        Reg::Rbx => regs.rbx,
        Reg::Rcx => regs.rcx,
        Reg::Rdx => regs.rdx,
        Reg::Rdi => regs.rdi,
        Reg::Rsi => regs.rsi,
        Reg::Rbp => regs.rbp,
        Reg::Rsp => regs.rsp,
        Reg::R8 => regs.r8,
        Reg::R9 => regs.r9,
        Reg::R10 => regs.r10,
        Reg::R11 => regs.r11,
        Reg::R12 => regs.r12,
        Reg::R13 => regs.r13,
        Reg::R14 => regs.r14,
        Reg::R15 => regs.r15,
        Reg::Rip => regs.rip,
        Reg::Rflags => regs.eflags,
        Reg::Cs => regs.cs,
        Reg::OrigRax => regs.orig_rax,
        Reg::FsBase => regs.fs_base,
        Reg::GsBase => regs.gs_base,
        Reg::Fs => regs.fs,
        Reg::Gs => regs.gs,
        Reg::Es => regs.es,
        Reg::Ds => regs.ds,
        Reg::Ss => regs.ss,
    };

    Ok(val)
}

pub fn set_register_value(pid: Pid, r: Reg, value: u64) -> Result<(), nix::Error> {
    let mut regs = ptrace::getregs(pid).unwrap_or_else(|_| {
        eprintln!("Cannot get this process' registers");
        std::process::exit(-1);
    });

    match r {
        Reg::Rax => regs.rax = value,
        Reg::Rbx => regs.rbx = value,
        Reg::Rcx => regs.rcx = value,
        Reg::Rdx => regs.rdx = value,
        Reg::Rdi => regs.rdi = value,
        Reg::Rsi => regs.rsi = value,
        Reg::Rbp => regs.rbp = value,
        Reg::Rsp => regs.rsp = value,
        Reg::R8 => regs.r8 = value,
        Reg::R9 => regs.r9 = value,
        Reg::R10 => regs.r10 = value,
        Reg::R11 => regs.r11 = value,
        Reg::R12 => regs.r12 = value,
        Reg::R13 => regs.r13 = value,
        Reg::R14 => regs.r14 = value,
        Reg::R15 => regs.r15 = value,
        Reg::Rip => regs.rip = value,
        Reg::Rflags => regs.eflags = value,
        Reg::Cs => regs.cs = value,
        Reg::OrigRax => regs.orig_rax = value,
        Reg::FsBase => regs.fs_base = value,
        Reg::GsBase => regs.gs_base = value,
        Reg::Fs => regs.fs = value,
        Reg::Gs => regs.gs = value,
        Reg::Es => regs.es = value,
        Reg::Ds => regs.ds = value,
        Reg::Ss => regs.ss = value,
    };

    ptrace::setregs(pid, regs)
}
