use nix::{sys::ptrace, unistd::Pid};
use strum_macros::EnumString;

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Reg {
    Rax, Rbx, Rcx, Rdx,
    Rdi, Rsi, Rbp, Rsp,
    R8, R9, R10, R11,
    R12, R13, R14, R15,
    Rip, Rflags, Cs,
    OrigRax, FsBase,
    GsBase,
    Fs, Gs, Ss, Ds, Es
}

pub fn get_register_value(pid: Pid, r: Reg) -> Result<u64, nix::Error> {
    let regs = ptrace::getregs(pid)?;

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

    let mut regs = ptrace::getregs(pid)?;

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
