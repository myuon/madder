extern crate serde_json;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
use std::rc::Rc;
use spec::{Component, HaveComponent};
use feat::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "component_type")]
pub enum ComponentExt {
    Video(VideoComponent),
    Image(ImageComponent),
}

impl ComponentExt {
    fn from_json(json: serde_json::Value) -> Option<ComponentExt> {
        use ComponentExt::*;

        let t = json.as_object()?.get("component_type")?.clone();
        match t.as_str()? {
            "Video" => Some(Video(VideoComponent::new(json))),
            "Image" => Some(Image(ImageComponent::new(json))),
            _ => unreachable!(),
        }
    }
}

impl HaveComponent for ComponentExt {
    fn component(&self) -> &Component {
        use ComponentExt::*;

        match self {
            Video(c) => c.component(),
            Image(c) => c.component(),
        }
    }

    fn component_mut(&mut self) -> &mut Component {
        use ComponentExt::*;

        match self {
            Video(c) => c.component_mut(),
            Image(c) => c.component_mut(),
        }
    }

    fn get_pixbuf(&self, time: gst::ClockTime) -> Rc<gdk_pixbuf::Pixbuf> {
        use ComponentExt::*;

        match self {
            Video(c) => c.get_pixbuf(time),
            Image(c) => c.get_pixbuf(time),
        }
    }
}

impl From<serde_json::Value> for ComponentExt {
    fn from(json: serde_json::Value) -> ComponentExt {
        ComponentExt::from_json(json).unwrap()
    }
}

