use {
    mocha_utils::{Category, Command},
    std::io::ErrorKind,
};

fn main() {
    let error = Command::new("/usr/bin/id")
        .execution_policy((Category::Users, ErrorKind::PermissionDenied))
        .spawn_in_place();

    eprintln!("error: {error}");
}
