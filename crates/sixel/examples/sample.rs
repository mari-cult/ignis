use {
    image::{imageops, io::Reader as ImageReader, ImageResult},
    std::io,
};

fn main() -> ImageResult<()> {
    let image = ImageReader::open("sample.png")?
        .with_guessed_format()?
        .decode()?
        .resize(400, 400, imageops::Triangle)
        .into_rgba8();

    let mut string = String::new();
    let encoder = sixel::SixelEncoder::new(io::Cursor::new(unsafe { string.as_mut_vec() }));

    image.write_with_encoder(encoder)?;

    println!("{string}");

    Ok(())
}
