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

    fn encode_palette(&mut self, image: &RgbaImage) -> io::Result<Vec<usize>> {
        let area = (image.width() * image.height()) as usize;
        let mut indices = Vec::with_capacity(area);

        for pixel in image.pixels().copied() {
            indices.push(self.encode_color(ScaledRgb::from_rgba(pixel))?);
        }

        assert_eq!(indices.len(), area, "pixel count mismatch");

        Ok(indices)
    }

    fn encode_indices(&mut self, indices: Vec<usize>, width: usize) -> io::Result<()> {
        let mut bits = BitVec::with_capacity(width);
        let mut six_row_iter = indices.chunks_exact_mut(width * 6).peekable();

        while let Some(six_rows) = six_row_iter.next() {
            transpose(&mut six_rows, width, 6);

            while let Some(index) = index_iter.next() {
                bits.extend(row.iter().copied().map(|i| i == index));
                assert!(bits.len() == width);
                self.encode_row(index, &bits)?;
                bits.clear();

                if index_iter.peek().is_some() {
                    self.writer.write_all(b"$")?;
                }
            }

            if row_iter.peek().is_some() {
                self.writer.write_all(b"-")?;
            }
        }

        Ok(())
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

pub fn calculate_render_mask(colors: &[usize], target: usize, bits: &mut BitVec) {
    // Ideally this would explicitly use SIMD as follows:
    //
    // ```
    // vector.simd_eq(Simd::splat(target)).to_bitmask()
    // ```
    //
    // Until portable_simd is stablized, we shall use this:
    bits.extend(colors.iter().copied().map(|color| color == target));
}

fn transpose(matrix: &mut [usize], rows: usize, cols: usize) {
    for i in 0..rows {
        for j in (i + 1)..cols {
            matrix.swap(i * cols + j, j * rows + i);
        }
    }
}

fn main() -> io::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _resuilt = crossterm::terminal::disable_raw_mode();

        eprintln!("{info}");
    }));

    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(
        BufWriter::new(io::stdout()),
    ))?;

    crossterm::terminal::enable_raw_mode()?;

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;

    let size = terminal.backend_mut().window_size()?;

    let cell_width = size.pixels.width / size.columns_rows.width;
    let cell_height = size.pixels.height / size.columns_rows.height;

    let layout = Layout::new()
        .constraints([Constraint::Length(29), Constraint::Min(1)])
        .direction(ratatui::prelude::Direction::Horizontal)
        .split(Rect::new(
            0,
            0,
            size.columns_rows.width,
            size.columns_rows.height,
        ));

    //println!("{size:#?}");

    let mut image = image::io::Reader::open("sample.png")
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    /* println!("original: {:#?}", image.dimensions());

    let mut image = DynamicImage::from(image)
        .resize(512, 512, imageops::Triangle)
        .into_rgba8();

    println!("scaled: {:#?}", image.dimensions());*/

    let mut image = DynamicImage::from(image)
        .resize_exact(512 * 6, 384 / 6, imageops::Triangle)
        .into_rgba8();

    //println!("unfucked: {:#?}", image.dimensions());

    let color_map = color_quant::NeuQuant::new(10, 256, image.as_raw());
    let mut encoder = SixelEncoder::new(io::Cursor::new(Vec::new()));

    image::imageops::dither(&mut image, &color_map);

    //println!("dithered: {:#?}", image.dimensions());

    let width = image.width() as u16;
    let height = image.height() as u16;

    encoder.encode_header(width, height)?;

    let color_indices = encoder.encode_palette(&image)?;

    encoder.encode_indices(color_indices, width as usize)?;
    encoder.encode_footer()?;

    let string = unsafe { String::from_utf8_unchecked(encoder.into_inner().into_inner()) };

    let mut image_state = ImageState { encoded: string };

    terminal.draw(|frame| {
        let image = Image {};
        let text = Paragraph::new(vec![
            Line::from("Such epic! Wow!"),
            Line::from(format!("cell_width: {cell_width}")),
            Line::from(format!("cell_height: {cell_height}")),
            Line::from(format!("image_width: {width}")),
            Line::from(format!("image_height: {height}")),
        ]);

        frame.render_stateful_widget(image, layout[0], &mut image_state);
        frame.render_widget(text, layout[1]);
    })?;

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

use ratatui::{
    buffer::{Buffer, Cell},
    layout::{Constraint, Layout, Rect},
    prelude::Backend,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget},
};

pub struct Image {}

pub struct ImageState {
    encoded: String,
}

impl StatefulWidget for Image {
    type State = ImageState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buffer.get_mut(x, y).set_skip(true);
            }
        }

        buffer
            .get_mut(area.left(), area.top())
            .set_skip(false)
            .set_symbol(&state.encoded);
    }
}
