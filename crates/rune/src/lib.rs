use std::borrow::Cow;

pub use self::backend::Interface;

pub(crate) mod util;

pub mod backend;
//pub mod element;
pub mod style;
pub mod text;

/// The backend to use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    Egui,

    #[cfg(debug_assertions)]
    Mock,
}

/// An application.
pub trait App: Send + Sync + 'static {
    /// Application title.
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(env!("CARGO_PKG_NAME"))
    }

    // Render the application.
    fn render(&self, interface: &mut Interface<'_>);
}

/// Run the provided application.
pub fn run<A: App>(app: &mut A, backend: Backend) -> Result<(), ()> {
    match backend {
        #[cfg(feature = "backend-egui")]
        Backend::Egui => backend::egui::run(app),
        #[cfg(debug_assertions)]
        Backend::Mock => backend::mock::run(app),
        #[allow(unreachable_patterns)]
        backend => {
            let _app = app;

            unimplemented!("Support for {backend:?} was not enabled.");
        }
    }
}
