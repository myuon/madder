extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;

use component::component::*;

pub struct TextComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
}

impl TextComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> TextComponent {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"blue\">{}</span>", structure.entity).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        TextComponent {
            component: Component {
                structure: structure.clone(),
                name: "text".to_string(),
            },
            data: gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap(),
        }
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

    fn get_properties(&self) -> Vec<Property> {
        self.component.get_properties()
    }

    fn set_property(&mut self, prop: Property) {
        self.component.set_property(prop);
    }
}

