use {
    clap::Parser,
    linux::{
        seccomp::{
            bpf::{Action, Instruction, Program},
            Listener,
        },
        syscall::{Error, Id},
    },
    rustix::thread::set_no_new_privs,
    std::{ffi::OsString, io, process::Command, thread},
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

    set_no_new_privs(true)?;

    let program = Program::new(PROGRAM);

    println!("BPF program to install: {program:?}");

    let mut listener = Listener::install(&program)?;
    let child = Command::new(command).args(args).spawn()?;

    let handle = thread::spawn(move || {
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("stdout: {stdout}");

        io::Result::Ok(())
    });

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

    let _result = handle.join();

    Ok(())
}
