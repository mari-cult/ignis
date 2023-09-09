use {
    linux::seccomp::{
        bpf::{Action, Instruction, Program},
        Listener,
    },
    rustix::thread::set_no_new_privs,
    std::io,
};

const PROGRAM: &[Instruction] = &[
    Instruction::load_syscall_id(),
    //Instruction::syscall(Id::Read),
    Instruction::action(Action::UserNotify),
    //Instruction::action(Action::Allow),
];

fn main() -> io::Result<()> {
    set_no_new_privs(true)?;

    let program = Program::new(PROGRAM);

    println!("BPF program: {program:?}");

    let mut listener = Listener::install(&program)?;

    /*thread::spawn(move || {
        /*let output = Command::new("cat").arg("/etc/passwd").output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        println!("{stdout}");*/

        std::fs::read("/etc/passwd")?;

        io::Result::Ok(())
    });*/

    while let Ok(mut notification) = listener.recv() {
        notification.send_error(22)?;
    }

    Ok(())
}
