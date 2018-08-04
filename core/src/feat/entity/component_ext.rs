extern crate gdk_pixbuf;
extern crate gstreamer as gst;
use spec::{Component, HaveComponent};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "component_type")]
pub enum ComponentExt {
    Video(Component),
}

impl HaveComponent for ComponentExt {
    fn component(&self) -> &Component {
        use ComponentExt::*;

        match self {
            Video(c) => c,
        }
    }

    fn component_mut(&mut self) -> &mut Component {
        use ComponentExt::*;

        match self {
            Video(c) => c,
        }
    }

    fn get_pixbuf(&self, _: gst::ClockTime) -> gdk_pixbuf::Pixbuf {
        unimplemented!()
    }
}

impl ComponentExt {
}

