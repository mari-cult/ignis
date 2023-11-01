use image::Pixel;

/// Maximum component value for scaled RGB.
pub const MAX: u8 = 100;

/// Percentage-based RGB colors.
///
/// Each channel is `0`-`100`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct ScaledRgb(pub [u8; 3]);

impl ScaledRgb {
    /// Convert RGB to scaled RGB.
    pub fn from_rgb(rgb: image::Rgb<u8>) -> Self {
        Self(rgb.map(downscale_component).0)
    }

    /// Convert RGBA to scaled RGB.
    pub fn from_rgba(rgb: image::Rgba<u8>) -> Self {
        Self(rgb.to_rgb().map(downscale_component).0)
    }
}

impl image::Pixel for ScaledRgb {
    type Subpixel = u8;

    const CHANNEL_COUNT: u8 = 3;
    const COLOR_MODEL: &'static str = "Scaled RGB";

    fn channels(&self) -> &[Self::Subpixel] {
        &self.0
    }

    fn channels_mut(&mut self) -> &mut [Self::Subpixel] {
        &mut self.0
    }

    fn channels4(
        &self,
    ) -> (
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
        Self::Subpixel,
    ) {
        let [r, g, b] = self.0;

        (r, g, b, MAX)
    }

    fn from_channels(
        a: Self::Subpixel,
        b: Self::Subpixel,
        c: Self::Subpixel,
        d: Self::Subpixel,
    ) -> Self {
        let _d = d;

        Self([a, b, c])
    }

    fn from_slice(slice: &[Self::Subpixel]) -> &Self {
        assert_eq!(slice.len(), Self::CHANNEL_COUNT as usize);

        unsafe { &*(slice.as_ptr() as *const Self) }
    }

    fn from_slice_mut(slice: &mut [Self::Subpixel]) -> &mut Self {
        assert_eq!(slice.len(), Self::CHANNEL_COUNT as usize);

        unsafe { &mut *(slice.as_mut_ptr() as *mut Self) }
    }

    fn to_rgb(&self) -> image::Rgb<Self::Subpixel> {
        image::Rgb(self.map(upscale_component).0)
    }

    fn to_rgba(&self) -> image::Rgba<Self::Subpixel> {
        self.to_rgb().to_rgba()
    }

    fn to_luma(&self) -> image::Luma<Self::Subpixel> {
        self.to_rgb().to_luma()
    }

    fn to_luma_alpha(&self) -> image::LumaA<Self::Subpixel> {
        self.to_rgba().to_luma_alpha()
    }

    fn map<F>(&self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = *self;

        this.apply(f);
        this
    }

    fn apply<F>(&mut self, mut f: F)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        for component in &mut self.0 {
            *component = f(*component);
        }
    }

    fn map_with_alpha<F, G>(&self, f: F, g: G) -> Self
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = *self;

        this.apply_with_alpha(f, g);
        this
    }

    fn apply_with_alpha<F, G>(&mut self, f: F, g: G)
    where
        F: FnMut(Self::Subpixel) -> Self::Subpixel,
        G: FnMut(Self::Subpixel) -> Self::Subpixel,
    {
        self.apply(f);

        let _g = g;
    }

    fn map2<F>(&self, other: &Self, f: F) -> Self
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        let mut this = *self;

        this.apply2(other, f);
        this
    }

    fn apply2<F>(&mut self, other: &Self, mut f: F)
    where
        F: FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    {
        for (a, &b) in self.0.iter_mut().zip(other.0.iter()) {
            *a = f(*a, b)
        }
    }

    fn invert(&mut self) {
        let mut rgb = self.to_rgb();

        rgb.invert();

        *self = Self::from_rgb(rgb);
    }

    fn blend(&mut self, other: &Self) {
        let mut rgb = self.to_rgb();

        rgb.blend(&other.to_rgb());

        *self = Self::from_rgb(rgb);
    }
}

/// Convert an 8-bit color component to a scaled color component.
fn downscale_component(component: u8) -> u8 {
    ((component as u16 * MAX as u16) / u8::MAX as u16) as u8
}

/// Convert a scaled color component to an 8-bit color component.
fn upscale_component(component: u8) -> u8 {
    assert!(component <= 100);

    ((component as u16 * u8::MAX as u16) / MAX as u16) as u8
}
