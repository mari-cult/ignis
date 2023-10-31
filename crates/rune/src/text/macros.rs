macro_rules! default_text_id {
    () => {
        #[inline]
        fn text_id(&self) -> TextId {
            <Self as DynTextId>::TEXT_ID
        }
    };
}

macro_rules! default_to_str {
    () => {
        #[inline]
        fn to_str(&self) -> Cow<'_, str> {
            Cow::Borrowed(self)
        }
    };
}

macro_rules! default_render {
    () => {
        #[inline]
        fn render(&self, renderer: &mut Renderer<'_>) {
            renderer.render(self)
        }
    };
}

macro_rules! impl_primitive {
    ($($ty:ty = $id:literal,)*) => {$(
        $crate::text::macros::impl_text_id! {
            $ty = $id,
        }

        impl Text for $ty {
            $crate::text::macros::default_render! {}
            $crate::text::macros::default_text_id! {}
            $crate::text::macros::default_to_str! {}
        }
    )*};
}

macro_rules! text_id {
    ($id:literal) => {{
        const TEXT_ID: TextId = unsafe { TextId::new_bytes(*$id) };

        TEXT_ID
    }};
}

macro_rules! impl_text_id {
    ($($ty:ty = $id:literal,)*) => {$(
        unsafe impl DynTextId for $ty {
            const TEXT_ID: TextId = $crate::text::macros::text_id!($id);
        }
    )*};
}

pub(crate) use {
    default_render, default_text_id, default_to_str, impl_primitive, impl_text_id, text_id,
};
