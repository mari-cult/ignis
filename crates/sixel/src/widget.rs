use {
    super::SixelEncoder,
    image::{imageops, DynamicImage, RgbaImage},
    ratatui::{
        backend::Backend,
        buffer::Buffer,
        layout::{Rect, Size},
        style::Style,
        widgets::{Block, StatefulWidget, Widget},
    },
    std::{
        io::{self},
        mem,
    },
};

pub struct Image<'a> {
    block: Option<Block<'a>>,
    style: Style,
}

pub struct ImageState<'a> {
    cell_size: Size,
    encoded: String,
    has_drawn: bool,
    image: &'a RgbaImage,
    last_size: Option<Size>,
}

impl<'a> Default for Image<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Image<'a> {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Style::default(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> ImageState<'a> {
    pub fn new(image: &'a RgbaImage, cell_size: Size) -> Self {
        Self {
            cell_size,
            encoded: String::new(),
            has_drawn: false,
            image,
            last_size: None,
        }
    }

    fn update(&mut self, area: Rect) {
        let size = Size {
            width: area.width * self.cell_size.width,
            height: area.height * self.cell_size.height,
        };

        if self.last_size.is_some_and(|last_size| last_size != size) {
            return;
        }

        self.last_size = Some(size);
        self.encoded.clear();

        let encoder = SixelEncoder::new(io::Cursor::new(unsafe { self.encoded.as_mut_vec() }))
            .palette_size(16);

        let _result = DynamicImage::from(self.image.clone())
            .resize(
                u32::from(size.width),
                u32::from(size.height),
                imageops::Triangle,
            )
            .write_with_encoder(encoder);
    }
}

impl<'a> StatefulWidget for Image<'a> {
    type State = ImageState<'a>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self
            .block
            .take()
            .map(|block| {
                let inner_area = block.inner(area);

                block.render(area, buf);

                inner_area
            })
            .unwrap_or(area);

        state.update(area);

        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).set_skip(true);
            }
        }

        if !mem::replace(&mut state.has_drawn, true) {
            buf.get_mut(area.left(), area.top())
                .set_skip(false)
                .set_symbol(&state.encoded);
        }
    }
}
