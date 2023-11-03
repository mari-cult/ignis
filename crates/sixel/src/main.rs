use image::{DynamicImage, RgbaImage};

use {
    self::internal::InternalEncoder,
    bitvec::vec::BitVec,
    image::{buffer::Pixels, imageops, Rgba},
    itertools::Itertools,
    ratatui::{
        backend::{Backend, CrosstermBackend},
        layout::{Constraint, Direction, Layout, Rect},
    },
    std::{
        array,
        io::{self, BufWriter, Write},
        mem,
        ops::Range,
        path::Path,
    },
};

mod internal;

type RgbaImageSlice<'a> = image::ImageBuffer<Rgba<u8>, &'a [u8]>;

#[derive(Clone, Debug)]
struct TransposeRgba<'a> {
    pixels: Pixels<'a, Rgba<u8>>,
    columns: Range<usize>,
    rows: Range<usize>,
}

impl<'a> TransposeRgba<'a> {
    pub fn new(pixels: Pixels<'a, Rgba<u8>>, columns: usize, rows: usize) -> Self {
        Self {
            pixels,
            columns: 0..columns,
            rows: 0..rows,
        }
    }
}

impl<'a> Iterator for TransposeRgba<'a> {
    type Item = [u8; 4];

    fn next(&mut self) -> Option<Self::Item> {
        let Some(column) = self.columns.next() else {
            self.rows.next()?;
            self.columns.start = 0;

            return self.next();
        };

        let index = self.rows.start + column * self.columns.end;
        let pixel = self.pixels.clone().nth(index)?.0;

        Some(pixel)
    }
}

pub struct SixelEncoder<W: Write> {
    encoder: InternalEncoder<W>,
}

impl<W: Write> SixelEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            encoder: InternalEncoder::new(writer),
        }
    }

    pub fn into_inner(self) -> W {
        self.encoder.into_inner()
    }
}

impl<W: Write> image::ImageEncoder for SixelEncoder<W> {
    fn write_image(
        mut self,
        bytes: &[u8],
        width: u32,
        height: u32,
        color_type: image::ColorType,
    ) -> image::ImageResult<()> {
        if color_type != image::ColorType::Rgba8 {
            todo!();
        }

        let width: u16 = width.try_into().map_err(|_error| todo!()).unwrap();
        let height: u16 = height.try_into().map_err(|_error| todo!()).unwrap();

        let image = RgbaImageSlice::from_raw(width as u32, height as u32, bytes).unwrap();
        let color_map = color_quant::NeuQuant::new(10, 256, image.as_raw());

        self.encoder.write_enter_sixel_mode(width, height)?;

        for (index, [red, green, blue, alpha]) in color_map
            .color_map_rgba()
            .chunks_exact(4)
            .map(|rgba| <[u8; 4]>::try_from(rgba).unwrap())
            .enumerate()
        {
            if alpha < 127 {
                continue;
            }

            fn scale(channel: u8) -> u8 {
                ((channel as u16 * 255) / 100) as u8
            }

            let rgb = [scale(red), scale(green), scale(blue)];

            self.encoder.write_set_color_register(index as u16, rgb)?;
        }

        let width = width as usize;
        let width_by_six = width * 6;
        let width_by_six_in_bytes = width_by_six * mem::size_of::<Rgba<u8>>();

        let mut render_bits = BitVec::with_capacity(width_by_six);
        let mut six_rows_iter = image
            .as_raw()
            .chunks_exact(width_by_six_in_bytes)
            .map(|six_rows| RgbaImageSlice::from_raw(width as u32, 6, six_rows).unwrap())
            .peekable();

        while let Some(six_rows) = six_rows_iter.next() {
            let transposed = TransposeRgba::new(six_rows.pixels(), 6, width);
            let mut unique_rgba_iter = transposed.clone().unique().peekable();

            while let Some(unique_rgba) = unique_rgba_iter.next() {
                let index = color_map.index_of(&unique_rgba) as u16;
                let bits = transposed
                    .clone()
                    .map(|rgba| rgba[3] >= 127 && rgba == unique_rgba);

                render_bits.clear();
                render_bits.extend(bits);
                self.encoder.write_render_pixels(index, &render_bits)?;

                if unique_rgba_iter.peek().is_some() {
                    self.encoder.write_move_to_start_of_line()?;
                }
            }

            if six_rows_iter.peek().is_some() {
                self.encoder.write_move_to_next_line()?;
            }
        }

        self.encoder.write_exit_sixel_mode()?;

        Ok(())
    }
}

fn read_image<P: AsRef<Path>>(path: P) -> image::ImageResult<image::DynamicImage> {
    image::io::Reader::open(path)?
        .with_guessed_format()?
        .decode()
}

fn split_horizontal<const N: usize>(area: Rect, constraints: [Constraint; N]) -> [Rect; N] {
    let layout = Layout::new()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);

    array::from_fn(|index| layout[index])
}

fn main() -> image::ImageResult<()> {
    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(BufWriter::new(io::stdout())))?;
    let size = terminal.backend_mut().window_size()?;
    let cell_width = size.pixels.width / size.columns_rows.width;
    let cell_height = size.pixels.height / size.columns_rows.height;
    let mut layout = split_horizontal(
        Rect::new(0, 0, size.columns_rows.width, size.columns_rows.height),
        [Constraint::Length(16), Constraint::Min(1)],
    );
    let mut sixel_string = String::new();
    let sixel_encoder = SixelEncoder::new(io::Cursor::new(unsafe { sixel_string.as_mut_vec() }));

    layout[0].height = layout[0].width;

    let _image = //read_image("sample.png")?
    DynamicImage::from(RgbaImage::from_pixel(64, 64, Rgba([0, 255, 0, 255])))
        .resize(
            (layout[0].width * cell_width) as u32,
            (layout[0].height * cell_height) as u32,
            imageops::Triangle,
        )
        .into_rgba8()
        .write_with_encoder(sixel_encoder)?;

    println!("{sixel_string}");

    Ok(())
}
