extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use component::component::*;

pub struct ImageComponent(pub Component);

impl ImageComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> ImageComponent {
        let image = gdk_pixbuf::Pixbuf::new_from_file(&structure.entity).unwrap();

        ImageComponent(Component {
            structure: structure.clone(),
            name: "image".to_string(),
            data: Box::new(image),
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

