extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;
extern crate gdk_pixbuf;
extern crate cairo;

use spec::*;

#[derive(Clone)]
pub struct AviRenderer {
    pipeline: gst::Pipeline,
    appsrc: gsta::AppSrc,
    size: (i32, i32),
    pub current: i32,
    pub frames: i32,
    pub delta: u64,
}

impl AviRenderer {
    pub fn new(self_: impl HaveAviRenderer, uri: &str, audio_streams: Vec<(gst::ClockTime, Vec<gst::Element>)>, width: i32, height: i32, frames: i32, fps: i32) {
        let pipeline = gst::Pipeline::new(None);
        let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
        let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let queue = gst::ElementFactory::make("queue", None).unwrap();

        let avimux = gst::ElementFactory::make("avimux", None).unwrap();
        let sink = gst::ElementFactory::make("filesink", None).unwrap();
        sink.set_property("location", &uri).unwrap();

        pipeline.add_many(&[&appsrc, &videoconvert, &queue, &avimux, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &queue, &avimux, &sink]).unwrap();

        for (_, elems) in audio_streams {
            pipeline.add_many(elems.iter().collect::<Vec<_>>().as_slice()).unwrap();

            let mut vec: Vec<&gst::Element> = elems.iter().collect();
            vec.push(&avimux);
            gst::Element::link_many(vec.as_slice()).unwrap();
        }

        let appsrc = appsrc.dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(gstv::VideoFormat::Rgb, width as u32, height as u32).fps(gst::Fraction::new(fps,1)).build().unwrap();
        appsrc.set_caps(&info.to_caps().unwrap());
        appsrc.set_property_format(gst::Format::Time);

        let mut current = 0;
        let delta = (1000 / fps) as u64;
        appsrc.set_callbacks(
            gsta::AppSrcCallbacks::new()
                .need_data(move |appsrc,_| {
                    if current > frames {
                        let _ = appsrc.end_of_stream();
                        return;
                    }

                    let pixbuf = self_.get_pixbuf(current as u64 * delta * gst::MSECOND);
                    let mut buffer = gst::Buffer::with_size((width*height*3) as usize).unwrap();
                    {
                        let buffer = buffer.get_mut().unwrap();
                        buffer.set_pts(current as u64 * delta * gst::MSECOND);

                        let mut data = buffer.map_writable().unwrap();
                        let mut data = data.as_mut_slice();
                        let pixels = unsafe { pixbuf.get_pixels() };

                        use std::io::Write;
                        data.write_all(pixels).unwrap();
                    }
                    appsrc.push_buffer(buffer).into_result().unwrap();
                    current += 1;
                })
                .build(),
        );

        pipeline.set_state(gst::State::Playing).into_result().unwrap();

        let bus = pipeline.get_bus().unwrap();
        while let Some(msg) = bus.timed_pop(gst::CLOCK_TIME_NONE) {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {:?}",
                        err.get_error(),
                        err.get_debug(),
                    );
                    break;
                }
                _ => (),
            }
        }

        pipeline.set_state(gst::State::Null).into_result().unwrap();
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

pub trait HaveAviRenderer : HavePresenter + Clone + Send + 'static {
    fn renderer(&self) -> &AviRenderer;
    fn renderer_mut(&mut self) -> &mut AviRenderer;

    fn start_render(&mut self, uri: &str, frames: i32, fps: i32) {
        let size = self.project().size;
        AviRenderer::new(self.clone(), uri, self.get_audio_streams(), size.0, size.1, frames, fps)
    }
}

