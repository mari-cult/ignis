use image::ImageFormat;

use {image::io::Reader as ImageReader, std::io::Cursor};

/// Create a newtype for `&'static [u8]`.
macro_rules! static_bytes_newtype {
    ($ident:ident) => {
        #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(not(doc), repr(transparent))]
        pub struct $ident {
            bytes: &'static [u8],
        }

        impl $ident {
            #[inline(always)]
            pub(crate) const fn from_static(bytes: &'static [u8]) -> Self {
                Self { bytes }
            }

            #[inline(always)]
            pub const fn as_bytes(&self) -> &'static [u8] {
                self.bytes
            }

            #[inline(always)]
            pub const fn as_str(&self) -> &'static str {
                unsafe { ::core::str::from_utf8_unchecked(self.bytes) }
            }
        }

        impl ::core::fmt::Debug for $ident {
            #[inline(always)]
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self.as_str(), fmt)
            }
        }

        impl ::core::fmt::Display for $ident {
            #[inline(always)]
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(self.as_str(), fmt)
            }
        }
    };
}

/// Create a `&'static [T]`.
///
/// To silence `error[E0515]: cannot return value referencing temporary value`.
macro_rules! static_slice {
    ($type:ty: $($value:expr,)*) => {{
        const SLICE: &[$type] = &[$($value,)*];

        SLICE
    }};
}

macro_rules! content_kind {
    ($(const $ident:ident = ([$($mime:literal),*], [$($extension:literal),*]);)*) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum Repr {
            $($ident,)*
        }

        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct ContentKind {
            pub(crate) repr: Repr,
        }

        impl ContentKind {
            $(pub const $ident: Self = Self { repr: Repr::$ident };)*

            pub const fn from_extension_bytes(extension: &[u8]) -> Option<Self> {
                let kind = match extension {
                    $($($extension => Self::$ident,)*)*
                    _ => return None,
                };

                Some(kind)
            }

            #[inline(always)]
            pub const fn from_extension_str(extension: &str) -> Option<Self> {
                Self::from_extension_bytes(extension.as_bytes())
            }

            pub const fn extensions(self) -> &'static [Extension] {
                match self {
                    $(Self::$ident => static_slice![Extension: $(Extension::from_static($extension),)*],)*
                }
            }

            pub const fn from_mime_bytes(mime: &[u8]) -> Option<Self> {
                let kind = match mime {
                    $($($mime => Self::$ident,)*)*
                    _ => return None,
                };

                Some(kind)
            }

            #[inline(always)]
            pub const fn from_mime_str(mime: &str) -> Option<Self> {
                Self::from_mime_bytes(mime.as_bytes())
            }

            pub const fn mimes(self) -> &'static [Mime] {
                match self {
                    $(Self::$ident => static_slice![Mime: $(Mime::from_static($mime),)*],)*
                }
            }

            #[inline(always)]
            pub const fn is_audio(self) -> bool {
                is_audio(self)
            }

            #[inline(always)]
            pub const fn is_image(self) -> bool {
                is_image(self)
            }

            #[inline(always)]
            pub const fn is_video(self) -> bool {
                is_video(self)
            }

            #[inline(always)]
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                from_bytes(bytes)
            }
        }

        impl ::core::fmt::Debug for ContentKind {
            #[inline(always)]
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(&self.repr, fmt)
            }
        }
    };
}

macro_rules! matches_kind {
    ($value:expr, $($ident:ident)|*) => {
        matches!($value, $(ContentKind::$ident)|*)
    };
}

static_bytes_newtype!(Mime);
static_bytes_newtype!(Extension);

content_kind! {
    const AUDIO_FLAC = ([b"audio/flac"], [b"flac"]);
    const AUDIO_MP3 = ([b"audio/mp3"], [b"mp3"]);
    const AUDIO_OGG = ([b"audio/ogg"], [b"ogg"]);
    const AUDIO_OPUS = ([b"audio/opus"], [b"opus"]);
    const AUDIO_WAV = ([b"audio/wav"], [b"wav"]);
    const IMAGE_AVIF = ([b"image/avif"], [b"avif"]);
    const IMAGE_BMP = ([b"image/bmp", b"image/x-bmp"], [b"bmp", b"dib"]);
    const IMAGE_DDS = ([b"image/vnd-ms.dds"], [b"dds"]);
    const IMAGE_EXR = ([b"image/x-exr"], [b"exr"]);
    const IMAGE_GIF = ([b"image/gif"], [b"gif"]);
    const IMAGE_HDR = ([b"image/vnd.radiance"], [b"hdr"]);
    const IMAGE_ICO = ([b"image/x-icon"], [b"ico"]);
    const IMAGE_JPEG = ([b"image/jpg"], [b"jfi", b"jfif", b"jif", b"jpe", b"jpeg", b"jpg"]);
    const IMAGE_PNG = ([b"image/apng", b"image/png", b"image/vnd.mozilla.apng"], [b"apng", b"png"]);
    const IMAGE_PNM = ([b"image/x-portable-anymap", b"image/x-portable-bitmap", b"image/x-portable-graymap", b"image/x-portable-pixmap"], [b"pbm", b"pgm", b"pnm", b"ppm"]);
    const IMAGE_QOI = ([b"image/x-qoi"], [b"qoi"]);
    const IMAGE_TGA = ([b"image/x-targa", b"image/x-tga"], [b"icb", b"tga", b"vda", b"vst"]);
    const IMAGE_TIFF = ([b"image/tiff", b"image/tiff-fx"], [b"tif", b"tiff"]);
    const IMAGE_WEBP = ([b"image/webp"], [b"webp"]);
    const VIDEO_AVI = ([b"video/x-msvideo"], [b"avi"]);
    const VIDEO_GP3 = ([b"video/3gpp"], [b"3gp"]);
    const VIDEO_M3U8 = ([b"application/vnd.apple.mpegurl"], [b"m3u8"]);
    const VIDEO_MKV = ([b"video/x-matroska"], [b"mkv"]);
    const VIDEO_MOV = ([b"video/quicktime"], [b"mov"]);
    const VIDEO_MP4 = ([b"video/mp4"], [b"mp4"]);
    const VIDEO_TS = ([b"video/mp2t"], [b"ts"]);
    const VIDEO_WEBM = ([b"video/webm"], [b"webm"]);
    const VIDEO_WMV = ([b"video/x-ms-wmv"], [b"wmv"]);
}

const fn is_audio(kind: ContentKind) -> bool {
    matches_kind!(
        kind,
        AUDIO_FLAC | AUDIO_MP3 | AUDIO_OGG | AUDIO_OPUS | AUDIO_WAV
    )
}

const fn is_image(kind: ContentKind) -> bool {
    matches_kind!(
        kind,
        IMAGE_AVIF
            | IMAGE_BMP
            | IMAGE_DDS
            | IMAGE_EXR
            | IMAGE_GIF
            | IMAGE_HDR
            | IMAGE_ICO
            | IMAGE_JPEG
            | IMAGE_PNG
            | IMAGE_PNM
            | IMAGE_QOI
            | IMAGE_TGA
            | IMAGE_TIFF
            | IMAGE_WEBP
    )
}

const fn is_video(kind: ContentKind) -> bool {
    matches_kind!(
        kind,
        VIDEO_AVI
            | VIDEO_GP3
            | VIDEO_M3U8
            | VIDEO_MKV
            | VIDEO_MOV
            | VIDEO_MP4
            | VIDEO_TS
            | VIDEO_WEBM
            | VIDEO_WMV
    )
}

fn from_bytes(bytes: &[u8]) -> Option<ContentKind> {
    image_from_bytes(bytes).or_else(|| generic_from_bytes(bytes))
}

fn image_from_bytes(bytes: &[u8]) -> Option<ContentKind> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;

    let kind = match reader.format()? {
        ImageFormat::Avif => ContentKind::IMAGE_AVIF,
        ImageFormat::Bmp => ContentKind::IMAGE_BMP,
        ImageFormat::Dds => ContentKind::IMAGE_DDS,
        ImageFormat::Gif => ContentKind::IMAGE_GIF,
        ImageFormat::Hdr => ContentKind::IMAGE_HDR,
        ImageFormat::Ico => ContentKind::IMAGE_ICO,
        ImageFormat::Jpeg => ContentKind::IMAGE_JPEG,
        ImageFormat::Png => ContentKind::IMAGE_PNG,
        ImageFormat::Pnm => ContentKind::IMAGE_PNM,
        ImageFormat::Qoi => ContentKind::IMAGE_QOI,
        ImageFormat::Tga => ContentKind::IMAGE_TGA,
        ImageFormat::Tiff => ContentKind::IMAGE_TIFF,
        ImageFormat::WebP => ContentKind::IMAGE_WEBP,
        _ => return None,
    };

    Some(kind)
}

fn generic_from_bytes(bytes: &[u8]) -> Option<ContentKind> {
    let kind = infer::get(bytes)?;

    ContentKind::from_mime_str(kind.mime_type())
}
