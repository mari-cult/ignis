use {ffmpeg::util::frame::Video, image::DynamicImage};

/// A video frame.
pub struct Frame<'video> {
    frame: &'video mut Video,
}

impl<'video> Frame<'video> {
    /// The width of this frame.
    pub fn width(&self) -> u32 {
        self.frame.width()
    }

    /// The height of this frame.
    pub fn height(&self) -> u32 {
        self.frame.height()
    }

    ///
    pub fn to_image(&self, image: &mut DynamicImage) {}
}
