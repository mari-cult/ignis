use {crate::text::Text, std::fmt};

#[cfg(feature = "backend-egui")]
pub mod egui;

#[cfg(debug_assertions)]
pub mod mock;

/// Interface backend.
pub trait Backend: fmt::Debug {
    /// Render the specified text.
    fn render_text(&mut self, text: &dyn Text);
}

/// Backend-agnostic interface.
pub struct Interface<'backend> {
    backend: &'backend mut dyn Backend,
}

impl<'backend> Interface<'backend> {
    /// Create a new interface using the specified backend.
    #[inline]
    pub fn new(backend: &'backend mut dyn Backend) -> Self {
        Self { backend }
    }
}

impl<'backend> Interface<'backend> {
    /// Render the specified text.
    #[inline]
    #[track_caller]
    pub fn render_text(&mut self, text: &dyn Text) {
        self.backend.render_text(text)
    }
}
