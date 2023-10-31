#[cold]
#[inline(never)]
#[track_caller]
pub fn unimplemented(message: &'static str) -> ! {
    unimplemented!("{message}")
}
