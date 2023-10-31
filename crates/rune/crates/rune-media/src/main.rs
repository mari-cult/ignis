use {
    image::{
        codecs::{gif, png, webp},
        AnimationDecoder, Frames,
    },
    rune_content::ContentKind,
    std::{fmt, fs::File, io::Cursor, path::Path},
};

use std::io::BufReader;

use image::DynamicImage;

pub use self::error::Error;

mod error;

pub type Result<T> = std::result::Result<T, Error>;

pub struct MediaReader {
    kind: ContentKind,
    data: memmap2::Mmap,
}

impl MediaReader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let data = unsafe { memmap2::Mmap::map(&file)? };
        let kind = ContentKind::from_bytes(&data).ok_or_else(|| Error::UnsupportedMedia)?;

        Ok(Self { kind, data })
    }

    pub fn kind(&self) -> ContentKind {
        self.kind
    }
}

pub enum DynamicMedia<'a> {
    Image(DynamicImage),
    AnimatedImage(Frames<'a>),
    //Video(VideoFrames<'a>),
}

impl<'a> fmt::Debug for DynamicMedia<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(_image) => fmt.debug_struct("Image").finish_non_exhaustive(),
            Self::AnimatedImage(_frames) => {
                fmt.debug_struct("AnimatedImage").finish_non_exhaustive()
            }
        }
    }
}

fn main() -> Result<()> {
    let mut reader = MediaReader::open("anim.webp")?;

    let media = match reader.kind() {
        ContentKind::IMAGE_WEBP => {
            let decoder = webp::WebPDecoder::new(Cursor::new(&*reader.data))?;

            if decoder.has_animation() {
                DynamicMedia::AnimatedImage(decoder.into_frames())
            } else {
                DynamicMedia::Image(DynamicImage::from_decoder(decoder)?)
            }
        }
        ContentKind::IMAGE_PNG => {
            let decoder = png::PngDecoder::new(Cursor::new(&*reader.data))?;

            if decoder.is_apng() {
                DynamicMedia::AnimatedImage(decoder.apng().into_frames())
            } else {
                DynamicMedia::Image(DynamicImage::from_decoder(decoder)?)
            }
        }
        ContentKind::IMAGE_GIF => {
            let decoder = gif::GifDecoder::new(Cursor::new(&*reader.data))?;

            DynamicMedia::AnimatedImage(decoder.into_frames())
        }
        _ => todo!(),
    };

    println!("{media:?}");

    Ok(())
}

const MAP_MAX_LEN: u64 = 1024 * 1024 * 8;

pub enum Source {
    Map(Cursor<memmap2::Mmap>),
    Buffered(BufReader<File>),
}

impl Source {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();

        if length < MAP_MAX_LEN {
            let map = unsafe { memmap2::Mmap::map(&file) };
        }
    }
}
