extern crate gstreamer as gst;
extern crate gstreamer_app as gsta;
extern crate gstreamer_video as gstv;
extern crate gdk_pixbuf;
extern crate glib;
extern crate serde_json;
use std::rc::Rc;
use std::{thread, time};
use gst::prelude::*;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct VideoComponent {
    #[serde(flatten)]
    component: Component,

    data_path: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    pipeline: Option<gst::Pipeline>,
}

impl VideoComponent {
    pub fn new(json: serde_json::Value) -> VideoComponent {
        let mut comp: VideoComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn load(&mut self) {
        self.pipeline = Some(VideoComponent::create_data(&self.data_path));
    }

    fn create_data(uri: &str) -> gst::Pipeline {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let convert = gst::ElementFactory::make("videoconvert", Some("convert")).unwrap();
        let sink = gst::ElementFactory::make("gdkpixbufsink", Some("appsink")).unwrap();
        src.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &convert, &sink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&convert, &sink]).unwrap();

        let convert_ = convert.clone();
        decodebin.connect_pad_added(move |_,src_pad| {
            let sink_pad = convert_.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        // Wait until pipeline is ready
        // TODO: fix the dirty hack
        thread::sleep(time::Duration::from_secs(1));

        sink.set_property("post-messages", &glib::Value::from(&false)).unwrap();

        pipeline
    }

    fn peek_pixbuf(&self, time: gst::ClockTime) -> Result<gdk_pixbuf::Pixbuf, String> {
        let pipeline = self.pipeline.as_ref().unwrap();
        pipeline.seek_simple(gst::SeekFlags::FLUSH, time).map_err(|t| t.to_string())?;

        let appsink = pipeline.get_by_name("appsink").unwrap();
        let pixbuf = appsink.get_property("last-pixbuf").map_err(|t| t.to_string())?;
        pixbuf.get().ok_or("failed to fetch last-pixbuf".to_string())
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

