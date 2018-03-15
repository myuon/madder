extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;
extern crate gdk_pixbuf;
extern crate cairo;

#[derive(Clone)]
pub struct AviRenderer {
    appsrc: gsta::AppSrc,
    size: (usize, usize),
    pub current: i32,
    pub frames: i32,
    pub delta: u64,
}

impl AviRenderer {
    pub fn new(uri: &str, width: usize, height: usize, frames: i32, delta: u64) -> AviRenderer {
        let pipeline = gst::Pipeline::new(None);
        let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let avimux = gst::ElementFactory::make("avimux", None).unwrap();
        let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
        appsrc.set_property("emit-signals", &glib::Value::from(&true)).unwrap();

        let sink = gst::ElementFactory::make("filesink", None).unwrap();
        sink.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&appsrc, &videoconvert, &avimux, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &avimux, &sink]).unwrap();

        let appsrc = appsrc.clone().dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(gstv::VideoFormat::Rgb, width as u32, height as u32).fps(gst::Fraction::new(20,1)).build().unwrap();
        appsrc.set_caps(&info.to_caps().unwrap());
        appsrc.set_property_format(gst::Format::Time);
        appsrc.set_max_bytes(1);
        appsrc.set_property_block(true);

        let bus = pipeline.get_bus().unwrap();

        {
            let pipeline = pipeline.clone();
            bus.add_watch(move |_,msg| {
                use gst::MessageView;

                match msg.view() {
                    MessageView::Eos(..) => {
                        pipeline.set_state(gst::State::Null).into_result().unwrap();
                        glib::Continue(false)
                    },
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {:?}",
                            err.get_error(),
                            err.get_debug(),
                        );
                        pipeline.set_state(gst::State::Null).into_result().unwrap();
                        glib::Continue(false)
                    }
                    _ => glib::Continue(true),
                }
            });
        }

        pipeline.set_state(gst::State::Playing).into_result().unwrap();

        AviRenderer {
            appsrc: appsrc,
            size: (width,height),
            current: 0,
            frames: frames,
            delta: delta,
        }
    }

    pub fn render_step(&mut self, pixbuf: &gdk_pixbuf::Pixbuf) -> bool {
        let mut buffer = gst::Buffer::with_size(self.size.0*self.size.1*3).unwrap();
        {
            let buffer = buffer.get_mut().unwrap();
            buffer.set_pts(self.current as u64 * self.delta * gst::MSECOND);

            let mut data = buffer.map_writable().unwrap();
            let mut data = data.as_mut_slice();
            let pixels = unsafe { pixbuf.get_pixels() };

            use std::io::Write;
            data.write_all(pixels).unwrap();
        }
        self.appsrc.push_buffer(buffer).into_result().unwrap();
        self.current += 1;

        self.current < self.frames
    }

    pub fn render_finish(&self) {
        self.appsrc.end_of_stream().into_result().unwrap();
    }
}
