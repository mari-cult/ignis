use {
    mocha_utils::{Category, Command, Rule},
    std::io::ErrorKind,
};

fn main() {
    let error = Command::new("/usr/bin/curl")
        .arg("https://google.com")
        .execution_policy((Category::Network, Rule::Kill))
        .execution_policy((Category::Users, ErrorKind::PermissionDenied))
        .execution_policy((Category::SetUsers, ErrorKind::PermissionDenied))
        .spawn_in_place();

    eprintln!("error: {error}");
}
