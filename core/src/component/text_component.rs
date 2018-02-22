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
    data: gdk_pixbuf::Pixbuf,
}

impl TextComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> TextComponent {
        TextComponent {
            component: Component {
                structure: structure.clone(),
                name: "text".to_string(),
            },
            text_color: gdk::RGBA::white(),
            data: TextComponent::create_data(&structure.entity, gdk::RGBA::white()),
        }
    }

    fn create_data(text: &str, color: gdk::RGBA) -> gdk_pixbuf::Pixbuf {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"#{:X}{:X}{:X}{:X}\">{}</span>", (color.red * 255.0) as i32, (color.green * 255.0) as i32, (color.blue * 255.0) as i32, (color.alpha * 255.0) as i32, text).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()
    }

    pub fn reload(&mut self, new_text: String) {
        self.data = TextComponent::create_data(new_text.as_str(), self.text_color);
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
        props.insert("entity".to_string(), Document(self.component.structure.entity.clone()));
        props.insert("text_color".to_string(), Color(self.text_color.clone()));
        props
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("entity", Document(doc)) => self.reload(doc),
            (x,y) => self.component.set_property(x.to_string(), y),
        }
    }
}

