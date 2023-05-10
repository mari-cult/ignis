use std::{
    io::{self, BufWriter, Write},
    thread,
    time::Duration,
};

fn main() {
    println!();

    let mut stdout = BufWriter::new(io::stdout());

    let mut j = 0;
    let mut z = 0;

    for i in (0..26567_usize).step_by(187) {
        if i % 100 > 50 {
            j += 1;
        }

        if z % 10 < 7 {
            z += 1;
        }

        let mut bars = ProgressBars::new();

        if j < 57 {
            bars = bars.add("libzstd", j, 57);
        }

        if z < 298 {
            bars = bars.add("musl", z, 298);
        }

        bars.add("chromium", i, 26567)
            .terminal_width(80)
            .render(&mut stdout)
            .unwrap();

        thread::sleep(Duration::from_millis(59));
    }

    println!();
}

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

        let bar_count = bars.len();

        // Reserve lines for the progress bar.
        write!(writer, "{}", "\r\n\x1b[K".repeat(bar_count))?;

        // Move back up.
        write!(writer, "\r\x1b[{bar_count}A")?;

        for Bar {
            label,
            completed,
            total,
        } in bars
        {
            let message = format!("{completed:completed_width$} / {total}");
            let remaining_width = terminal_width.saturating_sub(message.len() - 1);
            let repeat = ((completed as f32) / (total as f32) * (remaining_width as f32)) as usize;
            let bar = bar_character.repeat(repeat);

            write!(
                writer,
                "\r{label:label_width$} {bar:remaining_width$} {message}\x1b[1B"
            )?;
        }

        // Move back up.
        write!(writer, "\r\x1b[{bar_count}A")?;

        writer.flush()?;

        Ok(())
    }
}
