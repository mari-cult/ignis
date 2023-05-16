use mocha_utils::{Category, Command, Rule};

fn main() {
    let error = Command::new("/usr/bin/curl")
        .arg("https://google.com")
        .execution_policy((Category::Network, Rule::Kill))
        .spawn_in_place();

    eprintln!("error: {error}");
}
