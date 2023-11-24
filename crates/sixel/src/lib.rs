use {
    self::{internal::InternalEncoder, rows_n::RowsN},
    bitvec::vec::BitVec,
    image::{
        error::{
            ImageFormatHint, LimitError, LimitErrorKind, UnsupportedError, UnsupportedErrorKind,
        },
        imageops::{self, ColorMap},
        ColorType, ImageBuffer, ImageEncoder, ImageError, ImageResult, Pixel, Rgba, RgbaImage,
    },
    itertools::Itertools,
    std::{
        io::{self, Write},
        num::{NonZeroU16, NonZeroU8},
    },
};

pub use self::widget::{Image, ImageState};

mod internal;
mod rows_n;
mod widget;

pub struct SixelEncoder<W: Write> {
    encoder: InternalEncoder<W>,
    alpha_threshold: NonZeroU8,
    palette_size: NonZeroU16,
    quality: NonZeroU8,
}

impl<W: Write> SixelEncoder<W> {
    #[inline]
    pub fn new(writer: W) -> Self {
        Self {
            encoder: InternalEncoder::new(writer),
            alpha_threshold: NonZeroU8::new(1).unwrap(),
            palette_size: NonZeroU16::new(256).unwrap(),
            quality: NonZeroU8::new(30).unwrap(),
        }
    }

    /// Set the alpha threshold, a percentage between 1 and 100.
    ///
    /// This is due to sixel RGB components being percentages.
    ///
    /// # Panics
    ///
    /// If `alpha_threshold` is not between 1 and 100.
    #[inline]
    #[track_caller]
    pub fn alpha_threshold(mut self, alpha_threshold: u8) -> Self {
        assert!(
            (1..=100).contains(&alpha_threshold),
            "alpha threshold must be between 1 and 100"
        );

        self.alpha_threshold = NonZeroU8::new(alpha_threshold).unwrap();
        self
    }

    /// Set the dither palette size.
    ///
    /// # Panics
    ///
    /// If `palette_size` is not between 1 and 256.
    #[inline]
    #[track_caller]
    pub fn palette_size(mut self, palette_size: u16) -> Self {
        assert!(
            (1..=256).contains(&palette_size),
            "palette size must be between 1 and 256"
        );

        self.palette_size = NonZeroU16::new(palette_size).unwrap();
        self
    }

    /// Set the dithe palette quality.
    ///
    /// # Panics
    ///
    /// If `palette_size` is not between 1 and 30.
    #[inline]
    #[track_caller]
    pub fn quality(mut self, quality: u8) -> Self {
        assert!(
            (1..=30).contains(&quality),
            "quality must be between 1 and 30"
        );

        self.quality = NonZeroU8::new(quality).unwrap();
        self
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.encoder.into_inner()
    }
}

impl<W: Write> ImageEncoder for SixelEncoder<W> {
    fn write_image(
        mut self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: ColorType,
    ) -> ImageResult<()> {
        if color_type != ColorType::Rgba8 {
            return Err(ImageError::Unsupported(
                UnsupportedError::from_format_and_kind(
                    ImageFormatHint::Name(String::from("sixel")),
                    UnsupportedErrorKind::Color(color_type.into()),
                ),
            ));
        }

        let width = map_dimension(width)?;
        let height = map_dimension(height)?;

        let mut image = ImageBuffer::<Rgba<u8>, _>::from_raw(
            u32::from(width.get()),
            u32::from(height.get()),
            buf.to_vec(),
        )
        .unwrap();

        for rgba in image.pixels_mut() {
            rgba.apply_with_alpha(map_channel, map_channel);
        }

        let color_map = color_quant::NeuQuant::new(
            i32::from(self.quality.get()),
            usize::from(self.palette_size.get()),
            image.as_raw(),
        );

        imageops::dither(&mut image, &color_map);

        self.encoder
            .write_enter_sixel_mode(width.get(), height.get())?;

        for (index, [red, green, blue, alpha]) in color_map
            .color_map_rgba()
            .chunks_exact(4)
            .map(|components| <[u8; 4]>::try_from(components).unwrap())
            .enumerate()
        {
            if alpha <= self.alpha_threshold.get() {
                continue;
            }

            self.encoder
                .write_set_color_register(index as u16, [red, green, blue])?;
        }

        let mut chunks =
            RowsN::<Rgba<u8>>::with_image(image.as_raw(), image.width(), image.height(), 6)
                .peekable();

        let mut render_bits = BitVec::with_capacity(width.get() as usize);

        while let Some(chunk) = chunks.next() {
            let mut rotated = ImageBuffer::new(chunk.height(), chunk.width());

            for (x, y, pixel) in chunk.enumerate_pixels() {
                rotated.put_pixel(y, x, *pixel);
            }

            let mut unique_pixels = rotated.pixels().unique().peekable();

            while let Some(unique_pixel) = unique_pixels.next() {
                render_bits.clear();
                render_bits.extend(
                    rotated.pixels().map(|pixel| {
                        pixel[3] > self.alpha_threshold.get() && pixel == unique_pixel
                    }),
                );

                self.encoder.write_render_line(
                    ColorMap::index_of(&color_map, unique_pixel) as u16,
                    &render_bits,
                )?;

                if unique_pixels.peek().is_some() {
                    self.encoder.write_move_to_start_of_line()?;
                }
            }

            if chunks.peek().is_some() {
                self.encoder.write_move_to_next_line()?;
            }
        }

        self.encoder.write_exit_sixel_mode()?;

        Ok(())
    }
}

#[inline]
fn map_channel(channel: u8) -> u8 {
    (channel as u16 * 100 / 255) as u8
}

#[inline]
fn map_dimension(dimension: u32) -> ImageResult<NonZeroU16> {
    if (1..u16::MAX as u32).contains(&dimension) {
        Ok(NonZeroU16::new(dimension as u16).unwrap())
    } else {
        Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )))
    }
}

#[inline]
pub fn encode(image: &RgbaImage) -> ImageResult<Vec<u8>> {
    let mut vec = Vec::new();

    encode_to(image, &mut vec)?;

    Ok(vec)
}

#[inline]
pub fn encode_string(image: &RgbaImage) -> ImageResult<String> {
    let mut string = String::new();

    unsafe {
        encode_to_string(image, &mut string)?;
    }

    Ok(string)
}

#[inline]
pub fn encode_to(image: &RgbaImage, vec: &mut Vec<u8>) -> ImageResult<()> {
    image.write_with_encoder(SixelEncoder::new(io::Cursor::new(vec)))
}

#[inline]
pub unsafe fn encode_to_string(image: &RgbaImage, string: &mut String) -> ImageResult<()> {
    encode_to(image, string.as_mut_vec())
}
