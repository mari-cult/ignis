//! Simple `write(2)` hello world.

use linux_syscall::{syscall, Error};

fn main() {
    let stdout = 1_usize;
    let buffer = "Hello, World!\n";
    let result: Result<usize, Error> =
        unsafe { syscall!(Write, stdout, buffer.as_ptr(), buffer.len()) };

    println!("{result:?}");
}
