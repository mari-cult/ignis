//! Less than optimal remote `strings` program.

use {
    clap::Parser,
    linux::process::Process,
    std::{
        fs,
        io::{self, Read},
        ops::Range,
    },
};

#[derive(Parser)]
pub struct Args {
    process_id: u32,
}

fn main() -> io::Result<()> {
    let Args { process_id } = Args::parse();

    let maps = fs::read_to_string(format!("/proc/{process_id}/maps"))?;
    let ranges = maps
        .split('\n')
        .flat_map(parse_line)
        .filter(|(_range, readable)| *readable)
        .map(|(range, _readale)| range);

    let mut string = String::new();

    for range in ranges {
        let Range { start, end } = range;

        let mut process = Process::new(process_id, start);
        let mut buffer = vec![0; end - start];

        if process.read_exact(&mut buffer).is_err() {
            continue;
        }

        string.push_str(&String::from_utf8_lossy(&buffer));
    }

    let string = string
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || character.is_ascii_punctuation())
        .collect::<String>();

    println!("{string:?}");

    Ok(())
}

/// Parse `561623aed000-561623b1e000 r--p 00000000 00:0e 44001813                   /usr/bin/zsh`.
fn parse_line(line: &str) -> io::Result<(Range<usize>, bool)> {
    let mut parts = line.split(' ');
    let range = parts
        .next()
        .ok_or_else(|| invalid_data("missing address range"))?;

    let range = parse_addr_range(range)?;
    let permissions = parts
        .next()
        .ok_or_else(|| invalid_data("missing permissions"))?;

    let readable = permissions.contains('r');

    Ok((range, readable))
}

/// Parse `7ffda677f000-7ffda6781000`.
fn parse_addr_range(range: &str) -> io::Result<Range<usize>> {
    let (start, end) = range
        .split_once('-')
        .ok_or_else(|| invalid_data("missing hyphen in range"))?;

    let start = parse_addr(start)?;
    let end = parse_addr(end)?;

    if start > end {
        return Err(invalid_data(
            "start address is greater than the end address",
        ));
    }

    Ok(start..end)
}

/// Parse `7ffda677f000`.
fn parse_addr(addr: &str) -> io::Result<usize> {
    usize::from_str_radix(addr, 16).map_err(invalid_data)
}

fn invalid_data<E>(error: E) -> io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    io::Error::new(io::ErrorKind::InvalidData, error)
}
