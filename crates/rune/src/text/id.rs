use {super::Text, std::fmt};

/// A unique identifier for the underlying type of a `dyn Text`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TextId {
    id: u64,
}

/// A trait to associate a [`TextId`] to a type.
///
/// # Safety
///
/// `Text::id()` must return the same [`TextId`] as `TEXT_ID`.
pub unsafe trait DynTextId: Text {
    const TEXT_ID: TextId;
}

impl TextId {
    /// Create a new text ID.
    ///
    /// # Safety
    ///
    /// `id` must be unique for the implemented type.
    #[inline]
    #[must_use]
    pub const unsafe fn new(id: u64) -> Self {
        Self { id }
    }

    /// Create a new text ID from eight bytes.
    ///
    /// # Safety
    ///
    /// `id` must be unique for the implemented type.
    #[inline]
    #[must_use]
    pub const unsafe fn new_bytes(id: [u8; 8]) -> Self {
        Self::new(u64::from_ne_bytes(id))
    }
}

impl fmt::Debug for TextId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("TextId").finish_non_exhaustive()
    }
}
