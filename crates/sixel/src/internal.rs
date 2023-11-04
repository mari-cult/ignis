use {
    bitvec::{field::BitField, slice::BitSlice},
    itertools::Itertools,
    std::{
        io::{self, Write},
        slice,
    },
};

pub struct InternalEncoder<W: Write> {
    writer: W,
}

impl<W: Write> InternalEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.write_bytes(slice::from_ref(&byte))
    }

    pub fn write_enter_sixel_mode(&mut self, width: u16, height: u16) -> io::Result<()> {
        write!(self.writer, "\x1bPq\"1;1;{width};{height}")
    }

    pub fn write_exit_sixel_mode(&mut self) -> io::Result<()> {
        self.write_bytes(b"\x1b\\")
    }

    pub fn write_set_color_register(&mut self, index: u16, rgb: [u8; 3]) -> io::Result<()> {
        let [red, green, blue] = rgb;

        write!(self.writer, "#{index};2;{red};{green};{blue}\n")
    }

    pub fn write_render_pixels(&mut self, index: u16, pixel: u8, repeat: usize) -> io::Result<()> {
        write!(self.writer, "#{index}")?;

        if repeat > 1 {
            write!(self.writer, "!{repeat}")?;
        }

        self.write_byte(pixel + 63)
    }

    pub fn write_render_line(&mut self, index: u16, render_bits: &BitSlice) -> io::Result<()> {
        let ones = render_bits.count_ones();

        if ones == 0 {
            Ok(())
        } else if ones == render_bits.len() {
            self.write_render_pixels(index, 0b111111, render_bits.len())
        } else {
            let iter = render_bits
                .chunks(6)
                .map(|pixel| pixel.load::<u8>())
                .dedup_with_count();

            for (repeat, pixel) in iter {
                self.write_render_pixels(index, pixel, repeat)?;
            }

            Ok(())
        }
    }

    pub fn write_move_to_start_of_line(&mut self) -> io::Result<()> {
        self.write_bytes(b"$\n")
    }

    pub fn write_move_to_next_line(&mut self) -> io::Result<()> {
        self.write_bytes(b"$-\n")
    }
}
