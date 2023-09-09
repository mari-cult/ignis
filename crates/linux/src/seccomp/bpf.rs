//! Berkley Packet Filters.
//!
//! # References
//!
//!  - [samples/seccomp/bpf-fancy.c](https://github.com/torvalds/linux/blob/master/samples/seccomp/bpf-fancy.c)

use {
    super::c::seccomp_data,
    crate::syscall::Id,
    core::{fmt, marker::PhantomData, mem, slice},
    libc::{
        BPF_ABS as ABS, BPF_JEQ as JEQ, BPF_JMP as JMP, BPF_K as K, BPF_LD as LD, BPF_RET as RET,
        BPF_W as W,
    },
    memoffset::offset_of,
};

const ACTION: u16 = (RET + K) as u16;
const SYSCALL: u16 = (JMP + JEQ + K) as u16;
const LOAD_SYSCALL_ID: u16 = (LD + W + ABS) as u16;

const NR_OFFSET: u32 = offset_of!(seccomp_data, nr) as u32;

/// A BPF action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Action {
    /// Allow execution.
    Allow = 0x7fff0000,
    /// Provide an error.
    Error = 0x00050000,
    /// Kill the process.
    KillProcess = 0x80000000,
    /// Kill the thread.
    KillThread = 0x00000000,
    /// Log the event.
    Log = 0x7ffc0000,
    /// Trace the event.
    Trace = 0x7ff00000,
    /// Cause a trap.
    Trap = 0x00030000,
    /// Notify a userspace listener.
    UserNotify = 0x7fc00000,
}

/// A BPF instruction.
#[cfg_attr(not(doc), repr(transparent))]
pub struct Instruction {
    instruction: libc::sock_filter,
}

/// A BPF program.
#[cfg_attr(not(doc), repr(transparent))]
#[must_use = "a BPF program does nothing by itself"]
pub struct Program<'bpf> {
    program: libc::sock_fprog,
    _phantom: PhantomData<&'bpf [Instruction]>,
}

impl Instruction {
    /// A jump.
    const fn jump(code: u16, k: u32, jt: u8, jf: u8) -> Self {
        Self {
            instruction: libc::sock_filter { code, jt, jf, k },
        }
    }

    /// A statement.
    const fn statement(code: u16, k: u32) -> Self {
        Self::jump(code, k, 0, 0)
    }

    /// Trigger an action.
    pub const fn action(action: Action) -> Self {
        Self::statement(ACTION, action as u32)
    }

    /// Load system call number.
    pub const fn load_syscall_id() -> Self {
        Self::statement(LOAD_SYSCALL_ID, NR_OFFSET)
    }

    /// Jump if equal to the specified system call.
    pub const fn syscall(id: Id) -> Self {
        Self::jump(SYSCALL, id as u32, 0, 1)
    }

    // Is a load syscall id instruction.
    const fn is_load_syscall_id(&self) -> bool {
        let libc::sock_filter { code, k, jt, jf } = self.instruction;

        code == LOAD_SYSCALL_ID && k == NR_OFFSET && jt == 0 && jf == 0
    }

    // Is a syscall instruction.
    const fn is_syscall(&self) -> Option<Id> {
        let libc::sock_filter { code, k, jt, jf } = self.instruction;

        if code == SYSCALL && jt == 0 && jf == 1 {
            // SAFETY: `k` is from `Instruction::syscall`.
            Some(unsafe { mem::transmute(k) })
        } else {
            None
        }
    }

    /// Is an action instruction.
    const fn is_action(&self) -> Option<Action> {
        let libc::sock_filter { code, k, jt, jf } = self.instruction;

        if code == ACTION && jt == 0 && jf == 0 {
            // SAFETY: `k` is from `Instruction::action`.
            Some(unsafe { mem::transmute(k) })
        } else {
            None
        }
    }
}

impl<'bpf> Program<'bpf> {
    /// Create a new `Program` from the provided instructions.
    ///
    /// # Panics
    ///
    /// If `program` length exceeds 65535.
    pub const fn new(program: &'bpf [Instruction]) -> Self {
        assert!(
            program.len() < 65535,
            "program length cannot exceed 65535 instructions."
        );

        // The assertion guarentees the length does not exceed `u16::MAX`.
        let len = program.len() as u16;

        // `Instruction` is `repr(transparent)`.
        let filter = program.as_ptr().cast::<libc::sock_filter>().cast_mut();

        Self {
            program: libc::sock_fprog { len, filter },
            _phantom: PhantomData,
        }
    }

    pub(crate) const fn as_ptr(&self) -> *mut libc::sock_fprog {
        // `Program` is `repr(transparent)`.
        (self as *const Program<'bpf>)
            .cast::<libc::sock_fprog>()
            .cast_mut()
    }

    pub(crate) const fn as_instructions(&self) -> &'bpf [Instruction] {
        // SAFETY: Simply reversing the creation process.
        unsafe {
            slice::from_raw_parts(
                self.program.filter.cast_const().cast::<Instruction>(),
                self.program.len as usize,
            )
        }
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_load_syscall_id() {
            fmt.write_str("LoadSyscallId")
        } else if let Some(id) = self.is_syscall() {
            fmt.debug_tuple("Syscall").field(&id).finish()
        } else if let Some(action) = self.is_action() {
            fmt.debug_tuple("Action").field(&action).finish()
        } else {
            unreachable!()
        }
    }
}

impl<'bpf> fmt::Debug for Program<'bpf> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_instructions(), fmt)
    }
}
