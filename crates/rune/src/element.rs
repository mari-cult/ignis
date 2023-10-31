use super::Renderer;

pub use self::span::Span;

mod span;

/// An element.
pub trait Element {
    fn render(&self, renderer: &mut Renderer);
}
