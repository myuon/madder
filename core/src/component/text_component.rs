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
        TextComponent {
            component: Component {
                structure: structure.clone(),
                name: "text".to_string(),
            },
            data: TextComponent::create_data(&structure.entity),
        }
    }

    fn create_data(text: &str) -> gdk_pixbuf::Pixbuf {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"white\">{}</span>", text).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()
    }

    pub fn reload(&mut self, new_text: String) {
        self.data = TextComponent::create_data(new_text.as_str());
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
        use EditType::*;

        let mut vec = self.component.get_properties();
        vec.pop();
        vec.push(
            Property { name: "entity".to_string(), edit_type: Document(self.component.structure.entity.clone()) }
        );
        vec
    }

    fn set_property(&mut self, prop: Property) {
        use EditType::*;

        match (prop.name.as_str(), prop.edit_type) {
            ("entity", Document(doc)) => self.reload(doc),
            (x,y) => self.component.set_property(Property { name: x.to_string(), edit_type: y }),
        }
    }
}

