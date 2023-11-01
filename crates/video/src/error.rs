use std::io;

pub trait IntoIoError {
    fn into_io_error(self) -> io::Error;
}

macro_rules! convert {
    ($self:expr; $($variant:ident => $name:literal,)*) => {
        match $self {
            $(Self::$variant => io_not_found(concat!($name, " not found")),)*
            Self::InvalidData => io_invalid_data("invalid data encountered"),
            Self::Other { errno } => io::Error::from_raw_os_error(errno),
            error => io::Error::new(io::ErrorKind::Other, error),
        }
    };
}

impl IntoIoError for ffmpeg::Error {
    fn into_io_error(self) -> io::Error {
        convert! {
            self;
            BsfNotFound => "bitstream filter",
            DecoderNotFound => "decoder",
            DemuxerNotFound => "dmuxer",
            EncoderNotFound => "encoder",
            OptionNotFound => "option",
            MuxerNotFound => "muxer",
            FilterNotFound => "filter",
            ProtocolNotFound => "protocol",
            StreamNotFound => "stream",
            HttpNotFound => "HTTP",
        }
    }
}

fn io_not_found(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, message)
}

fn io_invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
