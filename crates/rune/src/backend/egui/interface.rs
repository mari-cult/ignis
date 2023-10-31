use {
    super::EguiText,
    crate::{
        backend::Backend,
        text::{self, Text},
    },
    eframe::egui,
    std::fmt,
};

/// [`egui`] interface.
pub struct EguiInterface<'backend> {
    pub(crate) ui: &'backend mut egui::Ui,
}

impl<'backend> EguiInterface<'backend> {
    #[inline]
    pub fn new(ui: &'backend mut egui::Ui) -> Self {
        Self { ui }
    }
}

impl<'backend> Backend for EguiInterface<'backend> {
    #[inline]
    fn render_text(&mut self, text: &dyn Text) {
        let mut backend = EguiText::new(self);
        let mut renderer = text::Renderer::new(&mut backend);

        renderer.render(text);
    }
}

impl<'backend> fmt::Debug for EguiInterface<'backend> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("EguiInterface").finish_non_exhaustive()
    }
}
