use {super::Interface, crate::App, eframe::egui, std::mem};

pub use self::{interface::EguiInterface, text::EguiText};

pub mod conversion;
pub mod interface;
pub mod text;

/// Adapt [`App`] to [`eframe::App`]
pub struct EguiApp<A: App> {
    app: &'static mut A,
}

impl<A: App> EguiApp<A> {
    #[inline]
    pub fn new(app: &'static mut A) -> Self {
        Self { app }
    }
}

impl<A: App> eframe::App for EguiApp<A> {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            let mut backend = EguiInterface::new(ui);
            let mut interface = Interface::new(&mut backend);

            self.app.render(&mut interface);
        });
    }
}

pub fn run<A: App>(app: &mut A) -> Result<(), ()> {
    let title = app.title().to_string();
    let options = eframe::NativeOptions::default();

    // SAFETY: `app` is valid for the duration of this function.
    let app = unsafe { extend_lifetime_mut(app) };
    let result = eframe::run_native(
        &title,
        options,
        Box::new(move |_creation_context| Box::new(EguiApp::new(app))),
    );

    match result {
        Ok(()) => Ok(()),
        Err(_error) => Err(()),
    }
}

/// Extend the lifetime of `&mut App` to `&'static mut App`.
///
/// # Safety
///
/// Caller must ensure `&'static mut App` is valid for the duration of `&mut App`.
#[inline(always)]
#[must_use]
unsafe fn extend_lifetime_mut<A: App>(app: &mut A) -> &'static mut A {
    mem::transmute(app)
}
