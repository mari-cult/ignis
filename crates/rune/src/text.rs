//! Functionality for rendering text.

use {
    crate::style::Color,
    std::{borrow::Cow, fmt},
};

pub(crate) use self::macros::{
    default_render, default_text_id, impl_primitive, impl_text_id, text_id,
};

pub use self::{
    hyperlink::Hyperlink,
    id::{DynTextId, TextId},
    renderer::{Backend, Renderer},
    with_color::WithColor,
};

mod hyperlink;
mod renderer;
mod with_color;

pub(crate) mod macros;

pub mod id;

/// A piece of text.
pub trait Text: fmt::Debug {
    /// Returns a text color override, if present.
    #[inline]
    fn color_override(&self) -> Option<Color> {
        None
    }

    /// Render this text.
    fn render(&self, renderer: &mut Renderer<'_>);

    /// A unique identifier for the underlying type of this text.
    fn text_id(&self) -> TextId;

    /// Convert this text to a string.
    fn to_str(&self) -> Cow<'_, str>;
}

impl_primitive! {
    &str = b"str\0\0\0\0\0",
    String = b"String\0\0",
    Cow<'_, str> = b"Cow<str>",
}

impl_text_id! {
    &[&dyn Text] = b"&[&dyn]\0",
}

impl Text for &[&dyn Text] {
    default_render! {}
    default_text_id! {}

    #[inline]
    fn to_str(&self) -> Cow<'_, str> {
        Cow::Owned(self.iter().map(|text| text.to_str()).collect())
    }
}

impl dyn Text + '_ {
    /// Downcast this text into the specified underlying type.
    ///
    /// This is for specialization in backend implementations.
    pub fn downcast<T: DynTextId>(&self) -> Option<&T> {
        if self.text_id() == T::TEXT_ID {
            Some(unsafe { self.downcast_unchecked::<T>() })
        } else {
            None
        }
    }

    /// Downcast this text into the specified underlying type.
    ///
    /// This is for specialization in backend implementations.
    ///
    /// # Safety
    ///
    /// The underlying value must be of type `T`.
    #[inline]
    pub unsafe fn downcast_unchecked<T: DynTextId>(&self) -> &T {
        &*(self as *const dyn Text as *const T)
    }
}
