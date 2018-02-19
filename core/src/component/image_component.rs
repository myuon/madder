extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use component::component::*;

pub struct ImageComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
}

impl ImageComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> ImageComponent {
        let image = gdk_pixbuf::Pixbuf::new_from_file(&structure.entity).unwrap();

        ImageComponent {
            component: Component {
                structure: structure.clone(),
                name: "image".to_string(),
            },
            data: image,
        }
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

impl Peekable for ImageComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

impl ComponentWrapper for ImageComponent {
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

