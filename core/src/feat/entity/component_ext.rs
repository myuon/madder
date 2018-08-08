extern crate serde_json;
extern crate glib;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
use gstv::prelude::*;
use spec::{Component, HaveComponent};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "component_type")]
pub enum ComponentExt {
    Video(VideoComponent),
}

impl ComponentExt {
    fn from_json(json: serde_json::Value) -> Option<ComponentExt> {
        use ComponentExt::*;

        let t = json.as_object()?.get("component_type")?.clone();
        match t.as_str()? {
            "Video" => Some(Video(VideoComponent::new(json))),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoComponent {
    component: Component,
    video_url: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    video: Option<gst::Element>,
}

impl HaveComponent for ComponentExt {
    fn component(&self) -> &Component {
        use ComponentExt::*;

        match self {
            Video(c) => &c.component,
        }
    }

    fn component_mut(&mut self) -> &mut Component {
        use ComponentExt::*;

        match self {
            Video(c) => &mut c.component,
        }
    }

    fn get_pixbuf(&self, _: gst::ClockTime) -> gdk_pixbuf::Pixbuf {
        unimplemented!()
    }
}

impl From<serde_json::Value> for ComponentExt {
    fn from(json: serde_json::Value) -> ComponentExt {
        ComponentExt::from_json(json).unwrap()
    }
}

impl VideoComponent {
    pub fn new(json: serde_json::Value) -> VideoComponent {
        let mut comp: VideoComponent = serde_json::from_value(json).unwrap();
        comp.video = Some(VideoComponent::create_data(&comp.video_url));
        comp
    }

    pub fn load(&mut self) {
        self.video = Some(VideoComponent::create_data(&self.video_url));
    }

    fn create_data(uri: &str) -> gst::Element {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let queue = gst::ElementFactory::make("queue", None).unwrap();
        let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        src.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &queue, &convert, &pixbufsink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&queue, &convert, &pixbufsink]).unwrap();

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = queue.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        pixbufsink
    }
}

