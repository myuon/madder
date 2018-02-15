use component::component::*;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;

pub struct VideoTestComponent(Component);

impl VideoTestComponent {
    pub fn new(start_time: gst::ClockTime, coordinate: (i32,i32)) -> VideoTestComponent {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        pipeline.add_many(&[&src, &pixbufsink]).unwrap();
        src.link(&pixbufsink).unwrap();

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        VideoTestComponent(Component {
            name: "videotest".to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(pixbufsink),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

pub struct VideoFileComponent(Component);

impl VideoFileComponent {
    pub fn new(uri: &str, start_time: gst::ClockTime, coordinate: (i32,i32)) -> VideoFileComponent {
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

        VideoFileComponent(Component {
            name: uri.to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(pixbufsink),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}
