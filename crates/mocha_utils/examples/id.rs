use {
    mocha_utils::{Category, Command},
    std::io::ErrorKind,
};

fn main() {
    Command::new("/usr/bin/id")
        .execution_policy((Category::Users, ErrorKind::PermissionDenied))
        .spawn_blocking()
        .expect("no id")
        .wait()
        .expect("it died");
}
