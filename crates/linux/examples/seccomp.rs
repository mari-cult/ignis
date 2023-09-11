use {
    clap::Parser,
    linux::{
        fd,
        seccomp::{
            bpf::{Action, Instruction, Program},
            Listener,
        },
        syscall::{Error, Id},
    },
    rustix::thread,
    std::{
        ffi::OsString,
        io,
        os::unix::{
            io::{FromRawFd, IntoRawFd},
            process::CommandExt,
        },
        process::Command,
    },
};

const PROGRAM: &[Instruction] = &[
    Instruction::load_syscall_id(),
    Instruction::syscall(Id::Exit),
    Instruction::action(Action::UserNotify),
    Instruction::syscall(Id::ExitGroup),
    Instruction::action(Action::UserNotify),
    Instruction::syscall(Id::GetUid),
    Instruction::action(Action::UserNotify),
    Instruction::syscall(Id::GetGid),
    Instruction::action(Action::UserNotify),
    Instruction::action(Action::Allow),
];

#[derive(Parser)]
struct Args {
    command: OsString,
    args: Vec<OsString>,
}

fn main() -> io::Result<()> {
    let Args { command, args } = Args::parse();

    thread::set_no_new_privs(true)?;

    let program = Program::new(PROGRAM);

    println!("BPF program to install: {program:?}");

    let mut command = Command::new(command);
    let (sender, receiver) = fd::channel()?;

    command.args(args);

    unsafe {
        command.pre_exec(move || {
            let listener = Listener::install(&program)?;

            sender.send(listener)?;

            Ok(())
        });
    }

    let mut child = command.spawn()?;
    let fd = receiver.recv()?;
    let mut listener = unsafe { Listener::from_raw_fd(fd.into_raw_fd()) };

    while let Ok(mut notification) = listener.recv() {
        let process_id = notification.process_id();

        match notification.syscall() {
            Ok(Id::Exit) | Ok(Id::ExitGroup) => {
                println!("[{process_id}] exited");

                unsafe {
                    notification.send_continue()?;
                }

                break;
            }
            Ok(Id::GetGid) => {
                println!("[{process_id}] denied obtaining group id");

                notification.send_error(Error::PermissionDenied)?;
            }
            Ok(Id::GetUid) => {
                println!("[{process_id}] denied obtaining user id");

                notification.send_error(Error::PermissionDenied)?;
            }
            _ => unsafe { notification.send_continue()? },
        }
    }

    child.wait()?;

    Ok(())
}
