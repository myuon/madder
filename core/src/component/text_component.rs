extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;

use component::component::*;

pub struct TextComponent {
    component: Component,
    text_color: gdk::RGBA,
    text_font: String,
    data: gdk_pixbuf::Pixbuf,
}

impl TextComponent {
    pub fn new_from_structure(component: &Component) -> TextComponent {
        let text_color = gdk::RGBA::white();
        let text_font = "Serif 24".to_string();

        TextComponent {
            component: component.clone(),
            text_color: text_color,
            text_font: text_font.clone(),
            data: TextComponent::create_data(&component.entity, &text_font, text_color),
        }
    }

    fn create_data(text: &str, font: &str, color: gdk::RGBA) -> gdk_pixbuf::Pixbuf {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string(font));
        let markup = format!("<span foreground=\"#{:02X}{:02X}{:02X}{:02X}\">{}</span>", (color.red * 255.0) as i32, (color.green * 255.0) as i32, (color.blue * 255.0) as i32, (color.alpha * 255.0) as i32, text);
        layout.set_markup(markup.as_str());
        pangocairo::functions::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()
    }

    pub fn reload(&mut self) {
        self.data = TextComponent::create_data(&self.component.entity, &self.text_font, self.text_color);
    }
}

impl Peekable for TextComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

impl ComponentWrapper for TextComponent {
    fn get_component(&self) -> Component {
        self.component.get_component()
    }

    fn get_properties(&self) -> Properties {
        use Property::*;

        let mut props = self.component.get_properties();
        props.insert("entity".to_string(), Document(self.component.entity.clone()));
        props.insert("text_font".to_string(), Font(self.text_font.clone()));
        props.insert("text_color".to_string(), Color(self.text_color.clone()));
        props
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("entity", Document(doc)) => {
                self.component.entity = doc;
                self.reload()
            },
            ("text_font", Font(font)) => {
                self.text_font = font;
                self.reload();
            },
            ("text_color", Color(rgba)) => {
                self.text_color = rgba;
                self.reload();
            },
            (x,y) => self.component.set_property(x.to_string(), y),
        }
    }

    fn get_effect_properties(&self) -> Vec<Property> {
        self.component.get_effect_properties()
    }

    fn set_effect_property(&mut self, i: usize, value: Property) {
        self.component.set_effect_property(i, value);
    }
}

