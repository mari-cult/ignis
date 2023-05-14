use crate::{artifact::Artifact, atom::Atom, error::Error};

mod artifact;
mod atom;
mod error;
mod package;
mod tui;

type Result<T> = std::result::Result<T, Error>;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tui::Milk::run().await;
}
