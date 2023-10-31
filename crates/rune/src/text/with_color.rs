use {
    super::{
        default_render, default_text_id,
        id::{self, DynTextId},
        text_id, Renderer, Text, TextId,
    },
    crate::style::Color,
    std::borrow::Cow,
};

/// Text with an accent colour.
#[derive(Debug)]
pub struct WithColor<'text> {
    pub text: &'text dyn Text,
    pub color: Color,
}

impl<'text> WithColor<'text> {
    pub fn new(text: &'text dyn Text, color: Color) -> Self {
        Self { text, color }
    }
}

unsafe impl<'text> id::DynTextId for WithColor<'text> {
    const TEXT_ID: TextId = text_id!(b"WITHCOLR");
}

impl<'text> Text for WithColor<'text> {
    #[inline]
    fn color_override(&self) -> Option<Color> {
        Some(self.color)
    }

    default_render! {}
    default_text_id! {}

    #[inline]
    fn to_str(&self) -> Cow<'_, str> {
        self.text.to_str()
    }
}
