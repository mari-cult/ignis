use {
    crossterm::style::Stylize,
    std::io::{self, Write},
};

/// A single bar.
struct Bar<'a> {
    label: &'a str,
    completed: usize,
    total: usize,
}

/// A terminal widget to display one or more progress bars.
#[must_use]
pub struct ProgressBars<'a> {
    use_ascii: bool,
    terminal_width: usize,
    bars: Vec<Bar<'a>>,
}

impl<'a> ProgressBars<'a> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            use_ascii: false,
            terminal_width: 80,
            bars: Vec::new(),
        }
    }

    #[inline]
    pub const fn ascii(mut self, ascii: bool) -> Self {
        self.use_ascii = ascii;
        self
    }

    #[inline]
    pub fn add(mut self, label: &'a str, completed: usize, total: usize) -> Self {
        assert!(completed <= total);

        self.bars.push(Bar {
            label,
            completed,
            total,
        });

        self
    }

    #[inline]
    pub const fn terminal_width(mut self, width: usize) -> Self {
        self.terminal_width = width;
        self
    }

    #[inline]
    pub fn auto_terminal_width(self) -> Self {
        let width = terminal_size::terminal_size()
            .map(|(width, _height)| width.0 as usize)
            .unwrap_or(80);

        self.terminal_width(width)
    }

    pub fn render<W: Write>(self, writer: &mut W) -> io::Result<()> {
        let Self {
            use_ascii,
            terminal_width,
            bars,
        } = self;

        // "\u{2501}" is a box drawing character.
        let bar_character = if use_ascii { "-" } else { "\u{2501}" };

        // Determine the longest of each to insert padding for.
        let label_width = bars
            .iter()
            .map(|bar| bar.label.len())
            .max()
            .unwrap_or_default();

        let completed_width = bars
            .iter()
            .map(|bar| bar.completed.checked_ilog10().unwrap_or_default())
            .max()
            .unwrap_or_default() as usize;

        let remaining_width = terminal_width
            .saturating_sub(label_width)
            .saturating_sub(completed_width * 2)
            .saturating_sub(9);

        let bar_count = bars.len();

        // Reserve lines for the progress bar(s), then move back up.
        write!(
            writer,
            "{}\r\x1b[{bar_count}A",
            "\r\n\x1b[K".repeat(bar_count)
        )?;

        for Bar {
            label,
            completed,
            total,
        } in bars
        {
            const EMPTY: &str = "";

            let label_padding = label_width - label.len();
            let label = label.blue();
            let message = format!("{completed:completed_width$} / {total}");
            let repeat = ((completed as f32) / (total as f32) * (remaining_width as f32)) as usize;
            let bar = bar_character.repeat(repeat);

            write!(
                writer,
                "\r\x1b[K{label}{EMPTY:label_padding$} {bar:remaining_width$} {message}\x1b[1B"
            )?;
        }

        // Move back up.
        write!(writer, "\r\x1b[{bar_count}A")?;

        writer.flush()?;

        Ok(())
    }
}
