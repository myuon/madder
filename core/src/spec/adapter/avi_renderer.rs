extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;
extern crate gdk_pixbuf;
extern crate cairo;

use std::rc::Rc;
use spec::*;

#[derive(Clone)]
pub struct AviRenderer {
    appsrc: gsta::AppSrc,
    size: (i32, i32),
    pub current: i32,
    pub frames: i32,
    pub delta: u64,
}

impl AviRenderer {
    pub fn new(uri: &str, audio_pipelines: Vec<Rc<gst::Pipeline>>, width: i32, height: i32, frames: i32, delta: u64) -> AviRenderer {
        let pipeline = gst::Pipeline::new(None);
        let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
        let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();

        let audiotestsrc = gst::ElementFactory::make("audiotestsrc", None).unwrap();
        let audiomix = gst::ElementFactory::make("audiomixer", None).unwrap();
        let audioconvert = gst::ElementFactory::make("audioconvert", None).unwrap();
        let audiorate = gst::ElementFactory::make("audiorate", None).unwrap();

        let avimux = gst::ElementFactory::make("avimux", None).unwrap();
        let sink = gst::ElementFactory::make("filesink", None).unwrap();
        sink.set_property("location", &uri).unwrap();

        pipeline.add_many(&[&appsrc, &videoconvert, &audiotestsrc, &audiomix, &avimux, &audioconvert, &audiorate, &sink]).unwrap();
        gst::Element::link_many(&[&audiotestsrc, &audiomix, &audioconvert, &audiorate, &avimux]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &avimux, &sink]).unwrap();

        let appsrc = appsrc.dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(gstv::VideoFormat::Rgb, width as u32, height as u32).fps(gst::Fraction::new(20,1)).build().unwrap();
        appsrc.set_caps(&info.to_caps().unwrap());
        appsrc.set_property_format(gst::Format::Time);

        let bus = pipeline.get_bus().unwrap();

        {
            let pipeline = pipeline.clone();
            bus.add_watch(move |_,msg| {
                use gst::MessageView;
                println!("{:?}", msg);

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
        let mut buffer = gst::Buffer::with_size((self.size.0*self.size.1*3) as usize).unwrap();
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

pub trait HaveAviRenderer : HavePresenter {
    fn renderer(&self) -> &AviRenderer;
    fn renderer_mut(&mut self) -> &mut AviRenderer;

    fn render_new(&mut self, uri: &str, frames: i32, delta: u64) -> AviRenderer {
        let size = self.project().size;
        let audio_pipelines = self.get_audio_pipelines();
        AviRenderer::new(uri, audio_pipelines, size.0, size.1, frames, delta)
    }

    fn render_init(&mut self, uri: &str, frames: i32, delta: u64);

    fn render_next(&mut self) -> (bool, f64) {
        let pixbuf = self.get_pixbuf(self.renderer().current as u64 * self.renderer().delta * gst::MSECOND);
        if self.renderer_mut().render_step(&pixbuf) {
            (true, self.renderer().current as f64 / self.renderer().frames as f64)
        } else {
            self.renderer().render_finish();
            (false, 1.0)
        }
    }
}

