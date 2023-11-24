use {
    image::{ImageBuffer, Pixel},
    std::slice::ChunksExact,
};

pub struct RowsN<'a, P: Pixel + 'a>
where
    <P as Pixel>::Subpixel: 'a,
{
    pixels: ChunksExact<'a, P::Subpixel>,
    width: u32,
}

impl<'a, P: Pixel + 'a> RowsN<'a, P> {
    pub fn with_image(pixels: &'a [P::Subpixel], width: u32, height: u32, rows: u32) -> Self {
        let rows_n_len = width as usize * usize::from(<P as Pixel>::CHANNEL_COUNT) * rows as usize;

        if rows_n_len == 0 {
            Self {
                pixels: [].chunks_exact(1),
                width,
            }
        } else {
            let pixels = &pixels[..(rows_n_len * height as usize).min(pixels.len())];

            Self {
                pixels: pixels.chunks_exact(rows_n_len),
                width,
            }
        }
    }
}

impl<'a, P: Pixel + 'a> Iterator for RowsN<'a, P>
where
    <P as Pixel>::Subpixel: 'a,
{
    type Item = ImageBuffer<P, &'a [P::Subpixel]>;

    fn next(&mut self) -> Option<Self::Item> {
        self.pixels.next().map(|buf| unsafe {
            let height =
                buf.len() / (self.width as usize * usize::from(<P as Pixel>::CHANNEL_COUNT));

            ImageBuffer::from_raw(self.width, height as u32, buf).unwrap_unchecked()
        })
    }
}
