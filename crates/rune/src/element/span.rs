use {super::Element, crate::Renderer, std::borrow::Cow};

/// A span of text.
pub struct Span<'a> {
    span: Cow<'a, str>,
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(span: &'a str) -> Self {
        Self {
            span: Cow::Borrowed(span),
        }
    }
}

impl From<String> for Span<'_> {
    fn from(span: String) -> Self {
        Self {
            span: Cow::Owned(span),
        }
    }
}

impl<'a> From<Cow<'a, str>> for Span<'a> {
    fn from(span: Cow<'a, str>) -> Self {
        Self { span }
    }
}

impl<'a> Element for Span<'a> {
    fn render(&self, renderer: &mut Renderer<'_>) {
        renderer.text(&self.span);
    }
}
