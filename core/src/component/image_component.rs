extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use component::component::*;

pub struct ImageComponent(pub Component);

impl ImageComponent {
    pub fn new(uri: &str, start_time: gst::ClockTime, coordinate: (i32,i32)) -> ImageComponent {
        let image = gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap();

        ImageComponent(Component {
            name: uri.to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(image),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

impl Peekable for gdk_pixbuf::Pixbuf {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        Some(self.clone())
    }
}

