extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate serde_json;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TextComponent {
    #[serde(flatten)]
    component: Component,
    text_font: String,
    text_color: RGBA,
    text: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    data: Option<gdk_pixbuf::Pixbuf>,
}

impl TextComponent {
    pub fn new(json: serde_json::Value) -> TextComponent {
        let mut comp: TextComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn create_data(text: &str, font: &str, color: &RGBA) -> gdk_pixbuf::Pixbuf {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string(font));
        let markup = format!("<span foreground=\"#{:02X}{:02X}{:02X}{:02X}\">{}</span>", color.red as i32, color.green as i32, color.blue as i32, color.alpha as i32, text);
        layout.set_markup(markup.as_str());
        pangocairo::functions::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()
    }

    fn load(&mut self) {
        self.data = Some(Self::create_data(&self.text, &self.text_font, &self.text_color));
    }
}

impl HaveComponent for TextComponent {
    fn component(&self) -> &Component {
        &self.component
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn get_pixbuf(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.clone()
    }
}




