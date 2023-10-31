use {std::io, thiserror::Error};

#[derive(Debug, Error)]
pub enum Error {
    #[error("ffmpeg: {error}")]
    Ffmpeg {
        #[from]
        error: ffmpeg_next::Error,
    },
    #[error("image: {error}")]
    Image {
        #[from]
        error: image::ImageError,
    },
    #[error("io: {error}")]
    Io {
        #[from]
        error: io::Error,
    },

    #[error("unsupported media")]
    UnsupportedMedia,
}
