use mocha_utils::{Category, Command, Rule};

fn main() {
    Command::new("/usr/bin/curl")
        .arg("https://google.com")
        .execution_policy((Category::Network, Rule::Kill))
        .spawn_blocking()
        .expect("no curl")
        .wait()
        .expect("it died");
}
