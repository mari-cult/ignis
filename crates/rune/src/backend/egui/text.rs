use {
    super::{conversion::ToEguiColor, EguiInterface},
    crate::text::{self, Text},
    eframe::egui::widget_text::RichText,
};

/// An [`egui`](eframe::egui) text renderer.
#[derive(Debug)]
pub struct EguiText<'text, 'interface> {
    pub(crate) interface: &'text mut EguiInterface<'interface>,
}

impl<'text, 'interface> EguiText<'text, 'interface> {
    /// Create a new [`egui`](eframe::egui) text renderer.
    #[inline]
    pub fn new(interface: &'text mut EguiInterface<'interface>) -> Self {
        Self { interface }
    }
}

impl<'text, 'interface> text::Backend for EguiText<'text, 'interface> {
    #[inline]
    fn render(&mut self, text: &dyn text::Text) {
        let ui = &mut self.interface.ui;

        if let Some(text) = text.downcast::<&[&dyn text::Text]>() {
            ui.horizontal(|ui| {
                for text in text.iter() {
                    if let Some(hyperlink) = text.downcast::<text::Hyperlink>() {
                        let label = to_rich_text(hyperlink.label);
                        let url = hyperlink.url.to_str();

                        ui.hyperlink_to(label, url);
                    } else {
                        let text = to_rich_text(*text);

                        ui.label(text);
                    }
                }
            });
        } else {
            let text = to_rich_text(text);

            ui.label(text);
        }
    }
}

fn to_rich_text(text: &dyn Text) -> RichText {
    let mut rich = RichText::new(text.to_str());

    if let Some(color) = text.color_override() {
        rich = rich.color(color.to_egui_color());
    }

    rich
}
