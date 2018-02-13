extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;
extern crate pangocairo;
use gtk::ImageExt;

use component::component::*;

pub struct TextComponent(pub Component);

impl TextComponent {
    pub fn new(label: &str, size: (i32,i32), start_time: gst::ClockTime, coordinate: (i32,i32)) -> TextComponent {
        use pango::prelude::*;

        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, size.0, size.1).unwrap();
        let context = cairo::Context::new(&surface);
        let layout = pangocairo::functions::create_layout(&context).unwrap();
        layout.set_font_description(&pango::FontDescription::from_string("Serif 24"));
        layout.set_markup(format!("<span foreground=\"blue\">{}</span>", label).as_str());
        pangocairo::functions::show_layout(&context, &layout);

        TextComponent(Component {
            name: "text".to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(gdk::pixbuf_get_from_surface(&surface, 0, 0, surface.get_width(), surface.get_height()).unwrap()),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

impl Peekable for gtk::Image {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.get_pixbuf()
    }
}

