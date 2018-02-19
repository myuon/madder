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

    fn get_properties(&self) -> Vec<Property> {
        use EditType::*;

        let mut vec = self.component.get_properties();
        vec.push(
            Property { name: "entity".to_string(), edit_type: FilePath(self.component.structure.entity.clone()) }
        );
        vec
    }

    fn set_property(&mut self, prop: Property) {
        use EditType::*;

        match (prop.name.as_str(), prop.edit_type) {
            ("entity", FilePath(uri)) => self.reload(uri.as_str()),
            (x,y) => self.component.set_property(Property { name: x.to_string(), edit_type: y }),
        }
    }
}

