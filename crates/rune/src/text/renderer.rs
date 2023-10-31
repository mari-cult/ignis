use {super::Text, crate::util, std::fmt};

/// A text renderer backend.
pub trait Backend: fmt::Debug {
    /// Render the text.
    #[inline]
    #[track_caller]
    fn render(&mut self, text: &dyn Text) {
        let _text = text;

        util::unimplemented("This backend does not support rendering text.");
    }
}

/// A backend-agnostic text renderer.
#[derive(Debug)]
pub struct Renderer<'backend> {
    backend: &'backend mut dyn Backend,
}

impl<'backend> Renderer<'backend> {
    /// Create a new text renderer using the specified backend.
    #[inline]
    pub fn new(backend: &'backend mut dyn Backend) -> Self {
        Self { backend }
    }

    /// Render the text.
    #[inline]
    #[track_caller]
    pub fn render(&mut self, text: &dyn Text) {
        self.backend.render(text);
    }
}
