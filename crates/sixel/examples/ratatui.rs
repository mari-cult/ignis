use {
    crossterm::terminal,
    image::{imageops, io::Reader as ImageReader, DynamicImage, ImageResult, RgbaImage},
    ratatui::{
        backend::{Backend, CrosstermBackend},
        buffer::Buffer,
        layout::{Constraint, Direction, Layout, Rect, Size},
        style::Style,
        widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
    },
    sixel::SixelEncoder,
    sixel::{Image, ImageState},
    std::{
        io::{self, BufWriter},
        mem,
    },
};

fn main() -> ImageResult<()> {
    let image = ImageReader::open("sample.png")?
        .with_guessed_format()?
        .decode()?
        .into_rgba8();

    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(BufWriter::new(io::stdout())))?;
    let window_size = terminal.backend_mut().window_size()?;
    let cell_size = Size {
        width: window_size.pixels.width / window_size.columns_rows.width,
        height: window_size.pixels.height / window_size.columns_rows.height,
    };

    let mut image_state = ImageState::new(&image, cell_size);

    terminal::enable_raw_mode()?;

    crossterm::execute!(
        terminal.backend_mut(),
        terminal::Clear(terminal::ClearType::All),
    )?;

    terminal.draw(|frame| {
        let area = Rect::new(
            0,
            0,
            window_size.columns_rows.width,
            window_size.columns_rows.height,
        );

        let layout = Layout::new()
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .direction(Direction::Vertical)
            .split(area);

        let layout = Layout::new()
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .direction(Direction::Horizontal)
            .split(layout[0]);

        frame.render_widget(Paragraph::new("hi").block(block("hi block")), layout[0]);
        frame.render_stateful_widget(
            Image::new().block(block("image block")),
            layout[1],
            &mut image_state,
        );

        frame.render_widget(
            Paragraph::new("crazy!").block(block("crazy block")),
            layout[2],
        );
    })?;

    terminal::disable_raw_mode()?;

    Ok(())
}

fn block(title: &'static str) -> Block<'static> {
    Block::new().borders(Borders::ALL).title(title)
}
