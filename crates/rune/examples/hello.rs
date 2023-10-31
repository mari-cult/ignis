use {
    rune::{
        style::Color,
        text::{Hyperlink, Text, WithColor},
        Backend, Interface,
    },
    std::borrow::Cow,
};

#[derive(Debug, Default)]
pub struct App {
    text_area: String,
}

impl rune::App for App {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed("Hello!")
    }

    fn render(&self, interface: &mut Interface) {
        let text: &[&dyn Text] = &[
            &WithColor::new(&"Hello, ", Color::rgb_u8(0xFF, 0x00, 0x00)),
            &Hyperlink::new(&"world", &"https://www.weforum.org/"),
            &"!",
        ];

        interface.render_text(&text);
    }
}

fn main() -> Result<(), ()> {
    let mut app = App::default();

    #[cfg(all(debug_assertions, not(feature = "backend-egui")))]
    let backend = Backend::Mock;

    #[cfg(not(all(debug_assertions, not(feature = "backend-egui"))))]
    let backend = Backend::Egui;

    rune::run(&mut app, backend)
}
