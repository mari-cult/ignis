/// A color.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    /// Create a new `Color` from RGB.
    #[inline]
    pub const fn rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Create a new `Color` from RGBA.
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Create a new `Color` from 8-bit RGB.
    #[inline]
    pub fn rgb_u8(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba_u8(red, green, blue, u8::MAX)
    }

    /// Create a new `Color` from 8-bit RGBA.
    pub fn rgba_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red: red as f32 / u8::MAX as f32,
            green: green as f32 / u8::MAX as f32,
            blue: blue as f32 / u8::MAX as f32,
            alpha: alpha as f32 / u8::MAX as f32,
        }
    }

    /// Convert `Color` to 8-bit RGB.
    #[inline]
    pub fn to_rgb_u8(self) -> [u8; 3] {
        let [red, green, blue, _alpha] = self.to_rgba_u8();

        [red, green, blue]
    }

    /// Convert `Color` to 8-bit RGBA.
    pub fn to_rgba_u8(self) -> [u8; 4] {
        let Self {
            red,
            green,
            blue,
            alpha,
        } = self;

        let red = (red * u8::MAX as f32) as u8;
        let green = (green * u8::MAX as f32) as u8;
        let blue = (blue * u8::MAX as f32) as u8;
        let alpha = (alpha * u8::MAX as f32) as u8;

        [red, green, blue, alpha]
    }
}
