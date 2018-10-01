extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate glib;
extern crate serde_json;
use std::rc::Rc;
use gst::prelude::*;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoComponent {
    #[serde(flatten)]
    component: Component,

    data_path: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    data: Option<gst::Element>,
}

impl VideoComponent {
    pub fn new(json: serde_json::Value) -> VideoComponent {
        let mut comp: VideoComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn load(&mut self) {
        self.data = Some(VideoComponent::create_data(&self.data_path));
    }

    fn create_data(uri: &str) -> gst::Element {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();
        src.set_property("location", &glib::Value::from(uri)).unwrap();
        pixbufsink.set_property("post-messages", &glib::Value::from(&false)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &convert, &pixbufsink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&convert, &pixbufsink]).unwrap();

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = convert.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        pixbufsink
    }

    fn peek_pixbuf(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        let _ = self.data.as_ref().unwrap().seek_simple(gst::SeekFlags::FLUSH, time).ok()?;
        let p = self.data.as_ref().unwrap().get_property("last-pixbuf").ok()?;
        p.get::<gdk_pixbuf::Pixbuf>()
    }
}

impl HaveComponent for VideoComponent {
    fn component(&self) -> &Component {
        &self.component
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn get_pixbuf(&self, time: gst::ClockTime) -> Option<Rc<gdk_pixbuf::Pixbuf>> {
        Some(Rc::new(self.peek_pixbuf(time).unwrap()))
    }
}

