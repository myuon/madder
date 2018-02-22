extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use component::component::*;

pub struct ImageComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
}

impl ImageComponent {
    pub fn new_from_structure(structure: &ComponentStructure) -> ImageComponent {
        ImageComponent {
            component: Component {
                structure: structure.clone(),
                name: "image".to_string(),
            },
            data: ImageComponent::create_data(&structure.entity),
        }
    }

    fn create_data(uri: &str) -> gdk_pixbuf::Pixbuf {
        gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap()
    }

    pub fn reload(&mut self, uri: &str) {
        self.data = gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap();
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

    fn get_properties(&self) -> Properties {
        use Property::*;

        let mut props = self.component.get_properties();
        props.insert("entity".to_string(), FilePath(self.component.structure.entity.clone()));
        props
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("entity", FilePath(uri)) => self.reload(uri.as_str()),
            (x,y) => self.component.set_property(x.to_string(), y),
        }
    }
}

