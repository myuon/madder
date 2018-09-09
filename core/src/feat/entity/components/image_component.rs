extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate serde_json;

use spec::*;
use std::rc::Rc;

#[derive(Clone, Serialize, Deserialize)]
pub struct ImageComponent {
    #[serde(flatten)]
    component: Component,

    data_path: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    data: Option<Rc<gdk_pixbuf::Pixbuf>>,
}

impl ImageComponent {
    pub fn new(json: serde_json::Value) -> ImageComponent {
        let mut comp: ImageComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn create_data(uri: &str) -> gdk_pixbuf::Pixbuf {
        gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap()
    }

    fn load(&mut self) {
        self.data = Some(Rc::new(ImageComponent::create_data(&self.data_path)));
    }
}

impl HaveComponent for ImageComponent {
    fn component(&self) -> &Component {
        &self.component
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn get_pixbuf(&self, _: gst::ClockTime) -> Option<Rc<gdk_pixbuf::Pixbuf>> {
        Some(self.data.as_ref().unwrap().clone())
    }
}


