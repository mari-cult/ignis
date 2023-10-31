use {
    super::Backend,
    crate::{text, App, Interface},
};

#[derive(Debug)]
pub struct MockInterface;

#[derive(Debug)]
pub struct MockText;

impl Backend for MockInterface {
    fn render_text(&mut self, text: &dyn text::Text) {
        let mut backend = MockText;
        let mut renderer = text::Renderer::new(&mut backend);

        text.render(&mut renderer);
    }
}

impl text::Backend for MockText {
    fn render(&mut self, text: &dyn text::Text) {
        if let Some(text) = text.downcast::<&[&dyn text::Text]>() {
            for text in text.iter() {
                if let Some(hyperlink) = text.downcast::<text::Hyperlink>() {
                    let label = hyperlink.label.to_str();
                    let url = hyperlink.url.to_str();

                    print!("[{label}]({url})");
                } else {
                    let text = text.to_str();

                    print!("{text}");
                }
            }
        } else {
            let text = text.to_str();

            print!("{text}");
        }

        println!();
    }
}

pub fn run<A: App>(app: &mut A) -> Result<(), ()> {
    println!("Run mock: {}", app.title());

    let mut backend = MockInterface;
    let mut interface = Interface::new(&mut backend);

    app.render(&mut interface);

    Ok(())
}
