extern crate gstreamer as gst;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;

use component::component::*;

pub struct TextComponent(pub Component);

impl TextComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> TextComponent {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 640, 480).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"blue\">{}</span>", structure.entity).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        TextComponent(Component {
            structure: structure.clone(),
            name: "text".to_string(),
            data: Box::new(gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

