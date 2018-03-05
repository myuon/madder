extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use component::component::*;

pub struct ImageComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
}

impl ImageComponent {
    pub fn new_from_structure(component: &Component) -> ImageComponent {
        ImageComponent {
            component: component.clone(),
            data: ImageComponent::create_data(&component.entity),
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

impl AsRef<Component> for ImageComponent {
    fn as_ref(&self) -> &Component {
        &self.component
    }
}

impl AsMut<Component> for ImageComponent {
    fn as_mut(&mut self) -> &mut Component {
        &mut self.component
    }
}

impl ComponentWrapper for ImageComponent {
    fn get_properties(&self) -> Properties {
        use Property::*;

        let mut props = self.component.get_properties();
        props.push(("entity".to_string(), FilePath(self.component.entity.clone())));
        props
    }

    fn set_property(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("entity", FilePath(uri)) => self.reload(uri.as_str()),
            (x,y) => self.component.set_property(x, y),
        }
    }

    fn get_info(&self) -> String {
        let mut w = 0;
        let mut h = 0;
        format!("image {:?}", gdk_pixbuf::Pixbuf::get_file_info(&self.component.entity, &mut w, &mut h).unwrap().get_description())
    }
}

