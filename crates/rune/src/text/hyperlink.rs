use {
    super::{
        default_text_id,
        id::{self, DynTextId},
        text_id, Renderer, Text, TextId,
    },
    crate::style::Color,
    std::borrow::Cow,
};

/// A hyperlink.
#[derive(Debug)]
pub struct Hyperlink<'label, 'url> {
    pub label: &'label dyn Text,
    pub url: &'url dyn Text,
}

impl<'label, 'url> Hyperlink<'label, 'url> {
    /// Create a new hyperlink.
    pub fn new(label: &'label dyn Text, url: &'url dyn Text) -> Self {
        Self { label, url }
    }
}

unsafe impl<'label, 'url> id::DynTextId for Hyperlink<'label, 'url> {
    const TEXT_ID: TextId = text_id!(b"HYPRLINK");
}

impl<'label, 'url> Text for Hyperlink<'label, 'url> {
    /// Returns a text color override, if present.
    #[inline]
    fn color_override(&self) -> Option<Color> {
        self.label.color_override()
    }

    #[inline]
    fn render(&self, renderer: &mut Renderer<'_>) {
        renderer.render(self.label);
    }

    default_text_id! {}

    #[inline]
    fn to_str(&self) -> Cow<'_, str> {
        self.label.to_str()
    }
}
