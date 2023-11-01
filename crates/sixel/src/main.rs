use {
    bitvec::{field::BitField, slice::BitSlice, vec::BitVec},
    itertools::Itertools,
    std::{
        collections::{hash_map::Entry, HashMap},
        io::{self, BufWriter, Write},
        slice,
    },
};

use image::{imageops, DynamicImage, ImageBuffer, ImageEncoder, RgbaImage};

pub use self::pixel::ScaledRgb;

mod pixel;

pub struct SixelEncoder<W: Write> {
    writer: W,
    color_map: HashMap<ScaledRgb, usize>,
}

impl<W: Write> SixelEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            color_map: HashMap::new(),
        }
    }

    /// Encode the sixel header.
    fn encode_header(&mut self, width: u16, height: u16) -> io::Result<()> {
        write!(self.writer, "\x1bPq\"1;1;{width};{height}")
    }

    /// Encode the sixel footer.
    fn encode_footer(&mut self) -> io::Result<()> {
        self.writer.write_all(b"\x1b\\")
    }

    /// Encode a new line (move to the next row).
    fn encode_newline(&mut self) -> io::Result<()> {
        self.writer.write_all(b"$-")
    }

    /// Encode an overprint.
    fn encode_overprint(&mut self) -> io::Result<()> {
        self.writer.write_all(b"$")
    }

    /// Encode a scaled RGB pixel and register it in the internal color map.
    fn encode_color(&mut self, pixel: ScaledRgb) -> io::Result<usize> {
        let index = self.color_map.len();

        match self.color_map.entry(pixel) {
            Entry::Occupied(occupied) => Ok(*occupied.get()),
            Entry::Vacant(entry) => {
                entry.insert(index);

                let ScaledRgb([r, g, b]) = pixel;

                write!(self.writer, "#{index};2;{r};{g};{b}\n")?;

                Ok(index)
            }
        }
    }

    /// Encode a 6-bit pixel and perhaps a repeat sequence.
    fn encode_pixel(&mut self, pixel: u8, repeat: u16) -> io::Result<()> {
        assert!(pixel < 64, "not a 6-bit pixel");

        if repeat > 1 {
            write!(self.writer, "!{repeat}")?;
        }

        let byte = pixel + 63;

        self.writer.write_all(slice::from_ref(&byte))
    }

    /// Encode a row of pixels specified by `render` using the color from `index`.
    ///
    /// Repeating pixels are automatically folded into a repeat sequence.
    fn encode_row(&mut self, index: usize, render: &BitSlice) -> io::Result<()> {
        write!(self.writer, "#{index}")?;

        for (count, pixel) in render
            .chunks_exact(6)
            .map(|pixel| pixel.load::<u8>())
            .dedup_with_count()
        {
            self.encode_pixel(pixel, count as u16)?;
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut image = image::io::Reader::open("sample.png")
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .resize_exact(128, 64, imageops::Triangle)
        .into_rgba8();

    let color_map = color_quant::NeuQuant::new(10, 256, image.as_raw());
    let mut encoder = SixelEncoder::new(BufWriter::new(io::stdout()));
    let mut color_indices = Vec::new();

    image::imageops::dither(&mut image, &color_map);

    let width = image.width() as u16;
    let height = image.height() as u16;

    encoder.encode_header(width, height)?;

    for pixel in image.pixels() {
        let pixel = ScaledRgb::from_rgba(*pixel);
        let index = encoder.encode_color(pixel)?;

        color_indices.push(index);
    }

    writeln!(encoder.writer)?;

    let width = width as usize;
    let mut bits = BitVec::with_capacity(width);
    let mut rows = color_indices.chunks_exact(width).peekable();

    while let Some(row) = rows.next() {
        let mut iter = row.iter().copied().unique().peekable();

        while let Some(color_index) = iter.next() {
            bits.extend(row.iter().copied().map(|index| index == color_index));
            debug_assert!(bits.len() == width);
            encoder.encode_row(color_index, &bits)?;
            bits.clear();

            if iter.peek().is_some() {
                encoder.encode_overprint()?;
                writeln!(encoder.writer)?;
            }
        }

        if rows.peek().is_some() {
            encoder.encode_newline()?;
            writeln!(encoder.writer)?;
        }
    }

    encoder.encode_footer()?;
    encoder.writer.flush()?;

    Ok(())
}
