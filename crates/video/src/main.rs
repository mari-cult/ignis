#![allow(unstable_name_collisions)]

use {
    crate::{error::IntoIoError, util::polyfill::SlicePolyfill},
    ffmpeg::{
        codec, format, frame, media::Type as StreamKind, sys, util::format::Pixel as PixelFormat,
    },
    std::{ffi, io, iter, iter::FusedIterator, ptr},
};

mod error;

pub mod util;

pub struct Codecs {
    opaque: *mut ffi::c_void,
}

impl Iterator for Codecs {
    type Item = codec::codec::Codec;

    fn next(&mut self) -> Option<Self::Item> {
        let codec = unsafe { sys::av_codec_iterate(&mut self.opaque) };

        if codec.is_null() {
            None
        } else {
            Some(unsafe { codec::codec::Codec::wrap(codec.cast_mut()) })
        }
    }
}

impl FusedIterator for Codecs {}

pub fn codecs() -> Codecs {
    Codecs {
        opaque: ptr::null_mut(),
    }
}

fn main() -> io::Result<()> {
    ffmpeg::init().map_err(IntoIoError::into_io_error)?;

    let mut input = format::input(&"video.webm").map_err(IntoIoError::into_io_error)?;
    let stream = input
        .streams()
        .best(StreamKind::Video)
        .ok_or(ffmpeg::Error::StreamNotFound)
        .map_err(IntoIoError::into_io_error)?;

    let _index = stream.index();
    let decoder = codec::Context::from_parameters(stream.parameters())
        .map_err(IntoIoError::into_io_error)?
        .decoder()
        .video()
        .map_err(IntoIoError::into_io_error)?;

    let codec_id = decoder.id();

    println!("Required codec: {codec_id:?}");

    let cuvid_codec = codecs()
        .find(|codec| codec.id() == codec_id && codec.name().ends_with("_cuvid"))
        .ok_or(ffmpeg::Error::StreamNotFound)
        .map_err(IntoIoError::into_io_error)?;

    println!("Using codec: {:?}", cuvid_codec.name());

    let mut decoder = codec::Context::from_parameters(stream.parameters())
        .map_err(IntoIoError::into_io_error)?
        .decoder()
        .open_as(cuvid_codec)
        .map_err(IntoIoError::into_io_error)?
        .video()
        .map_err(IntoIoError::into_io_error)?;

    let mut frame = frame::Video::empty();

    for (_stream, packet) in input.packets() {
        decoder
            .send_packet(&packet)
            .map_err(IntoIoError::into_io_error)?;

        match decoder.receive_frame(&mut frame) {
            Ok(()) => {}
            Err(ffmpeg::Error::Other {
                errno: ffmpeg::error::EAGAIN,
            }) => continue,
            Err(error) => return Err(error.into_io_error()),
        }

        println!("Got frame.");
        println!("Format: {:?}", frame.format());

        let mut image = image::RgbImage::new(frame.width(), frame.height());

        match frame.format() {
            PixelFormat::P010LE => {
                println!("P010LE -> RGB");

                p010le_to_rgb_image(frame.data(0), &mut image);
            }
            PixelFormat::NV12 => {
                println!("NV12 -> RGB");

                nv12_to_rgb_image(
                    frame.data(0),
                    (frame.width() * frame.height()) as usize,
                    &mut image,
                );
            }
            PixelFormat::YUV420P => {
                println!("YUV420P -> RGB");

                yuv420p_to_rgb_image(
                    frame.data(0),
                    (frame.width() * frame.height()) as usize,
                    &mut image,
                );
            }
            format => unimplemented!("{format:?}"),
        }

        println!("Resize frame.");

        let image = image::DynamicImage::ImageRgb8(image).resize_exact(
            400,
            200,
            image::imageops::FilterType::Triangle,
        );
        let image = match image {
            image::DynamicImage::ImageRgb8(image) => image,
            _ => unreachable!(),
        };

        let bytes = image.as_raw();

        println!("RGB -> Sixel.");

        let sixel = icy_sixel::sixel_string(
            bytes,
            image.width() as i32,
            image.height() as i32,
            icy_sixel::PixelFormat::RGB888,
            icy_sixel::DiffusionMethod::Auto,
            icy_sixel::MethodForLargest::Auto,
            icy_sixel::MethodForRep::Auto,
            icy_sixel::Quality::LOW,
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        println!("{sixel}\n\n");
    }

    decoder.send_eof().map_err(IntoIoError::into_io_error)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Yuv {
    y: u16,
    u: u8,
    v: u8,
}

impl Yuv {
    #[inline(always)]
    pub const fn new(y: u16, u: u8, v: u8) -> Self {
        Self { y, u, v }
    }

    #[inline(always)]
    pub fn to_f32(self) -> [f32; 3] {
        let Self { y, u, v } = self;

        [y as f32, u as f32, v as f32]
    }

    #[inline(always)]
    pub fn nv12_to_rgb(self) -> [u8; 3] {
        let [y, u, v] = self.to_f32();

        let r = (y + 1.402 * (v - 128.0)).round();
        let g = (y - 0.344136 * (v - 128.0) - 0.714136 * (u - 128.0)).round();
        let b = (y + 1.772 * (u - 128.0)).round();

        [r as u8, g as u8, b as u8]
    }

    #[inline(always)]
    pub fn p010_to_rgb(self) -> [u8; 3] {
        let [y, u, v] = self.to_f32();

        let r = (1.164 * (y - 16.0) + 1.596 * (v - 128.0)).round();
        let g = (1.164 * (y - 16.0) - 0.813 * (v - 128.0) - 0.391 * (u - 128.0)).round();
        let b = (1.164 * (y - 16.0) + 2.018 * (u - 128.0)).round();

        [r as u8, g as u8, b as u8]
    }

    #[inline(always)]
    pub fn yuv_to_rgb(self) -> [u8; 3] {
        let [y, u, v] = self.to_f32();

        let r = 1.164 * (y - 16.0) + 1.596 * (v - 128.0);
        let g = 1.164 * (y - 16.0) - 0.813 * (v - 128.0) - 0.391 * (u - 128.0);
        let b = 1.164 * (y - 16.0) + 2.018 * (u - 128.0);

        [r as u8, g as u8, b as u8]
    }
}

fn p010le_to_rgb_image(source: &[u8], destination: &mut image::RgbImage) {
    #[inline]
    fn map_p010le(bytes: [u8; 6]) -> (u16, u16, u8, u8) {
        let [y0a, y0b, y1a, y1b, u, v] = bytes;

        let y0 = u16::from_le_bytes([y0a, y0b]);
        let y1 = u16::from_le_bytes([y1a, y1b]);

        (y0, y1, u, v)
    }

    #[inline]
    fn map_rgb2(bytes: &mut [u8; 6]) -> (&mut [u8; 3], &mut [u8; 3]) {
        let (rgb0, rgb1) = bytes.split_at_mut(3);

        let rgb0 = <&mut [u8; 3]>::try_from(rgb0).unwrap();
        let rgb1 = <&mut [u8; 3]>::try_from(rgb1).unwrap();

        (rgb0, rgb1)
    }

    let source_iter = source.array_chunks::<6>().copied().map(map_p010le);
    let destination_iter = destination.array_chunks_mut::<6>().map(map_rgb2);

    for ((y0, y1, u, v), (rgb0, rgb1)) in iter::zip(source_iter, destination_iter) {
        *rgb0 = Yuv::new(y0, u, v).p010_to_rgb();
        *rgb1 = Yuv::new(y1, u, v).p010_to_rgb();
    }
}

fn nv12_to_rgb_image(source: &[u8], y_len: usize, destination: &mut image::RgbImage) {
    let (y, uv) = source.split_at(y_len);
    let y_iter = y.array_chunks::<2>().copied().map(u16::from_ne_bytes);
    let uv_iter = uv.array_chunks::<2>().copied();

    let source_iter = iter::zip(y_iter, uv_iter).map(|(y, [u, v])| (y, u, v));
    let destination_iter = destination.array_chunks_mut::<3>();

    for ((y, u, v), rgb) in iter::zip(source_iter, destination_iter) {
        *rgb = Yuv::new(y, u, v).nv12_to_rgb();
    }
}

fn yuv420p_to_rgb_image(source: &[u8], channel_len: usize, destination: &mut image::RgbImage) {
    let (y, source) = source.split_at(channel_len);
    let (u, v) = source.split_at(channel_len);

    let y = y.iter().copied();
    let u = u.iter().copied();
    let v = v.iter().copied();

    let source_iter = iter::zip(iter::zip(y, u), v).map(|((y, u), v)| (y, u, v));
    let destination_iter = destination.array_chunks_mut::<3>();

    for ((y, u, v), rgb) in iter::zip(source_iter, destination_iter) {
        *rgb = Yuv::new(y as u16, u, v).yuv_to_rgb();
    }
}

fn yuv420p10le_to_rgb_image(
    yuv_data: &[u8],
    width: u32,
    height: u32,
    destination: &mut image::RgbImage,
) {
    for y in 0..height {
        for x in 0..width {
            let y_index = (y * width + x) as usize;
            let u_index = ((width * height) + (y / 2) * (width / 2) + (x / 2)) as usize;
            let v_index =
                ((width * height) + (width / 2) * (height / 2) + (y / 2) * (width / 2) + (x / 2))
                    as usize;

            let y_value = yuv_data[y_index] as f32 / 1023.0;
            let u_value = yuv_data[u_index] as f32 / 1023.0 - 0.5;
            let v_value = yuv_data[v_index] as f32 / 1023.0 - 0.5;

            let r = (y_value + 1.402 * v_value).round() as u8;
            let g = (y_value - 0.344136 * u_value - 0.714136 * v_value).round() as u8;
            let b = (y_value + 1.772 * u_value).round() as u8;

            destination.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
}
