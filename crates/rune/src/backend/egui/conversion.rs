use {crate::style::Color, eframe::egui::Color32};

/// Provides conversion to [`Color32`].
pub trait ToEguiColor {
    fn to_egui_color(self) -> Color32;
}

impl ToEguiColor for Color {
    #[inline]
    fn to_egui_color(self) -> Color32 {
        let [red, green, blue, alpha] = self.to_rgba_u8();

        Color32::from_rgba_premultiplied(red, green, blue, alpha)
    }
}
