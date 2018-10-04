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
        let sink = gst::ElementFactory::make("appsink", Some("appsink")).unwrap();
        src.set_property("location", &glib::Value::from(uri)).unwrap();
        sink.set_property("emit-signals", &glib::Value::from(&true)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &convert, &sink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&convert, &sink]).unwrap();

        let convert_ = convert.clone();
        decodebin.connect_pad_added(move |_,src_pad| {
            let sink_pad = convert_.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
            println!("added!");
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        // Wait until pipeline is ready
        // TODO: fix the dirty hack
        thread::sleep(time::Duration::from_secs(1));

        let appsink = sink.dynamic_cast::<gsta::AppSink>().unwrap();
        let pad = convert.get_static_pad("sink").unwrap();
        appsink.set_caps(&pad.get_current_caps().unwrap());

        pipeline
    }

    fn peek_pixbuf(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.pipeline.as_ref()?.seek_simple(gst::SeekFlags::FLUSH, time).unwrap();

        let convert = self.pipeline.as_ref()?.get_by_name("appsink")?;
        let pad = convert.get_static_pad("sink")?;
        let video_info: gstv::VideoInfo = gstv::VideoInfo::from_caps(pad.get_current_caps()?.as_ref())?;

        let sample = self.pipeline.as_ref()?.get_by_name("appsink")?.dynamic_cast::<gsta::AppSink>().ok()?.pull_preroll()?;
        let buffer = sample.get_buffer()?;
        let map = buffer.map_readable()?;

        println!("{:?} {:?}", video_info.width(), video_info.height());
        let pixbuf = gdk_pixbuf::Pixbuf::new_from_vec(
            map.to_vec(),
            gdk_pixbuf::Colorspace::Rgb,
            video_info.has_alpha(),
            8,
            video_info.width() as i32,
            video_info.height() as i32 / 2,
            video_info.width() as i32 * 3,
        );

        Some(pixbuf)
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

