use std::cmp;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate gtk;
use gtk::prelude::*;

extern crate glib;

extern crate gdk;
use gdk::prelude::*;

extern crate gdk_pixbuf;

extern crate cairo;

mod avi_renderer;
use avi_renderer::AviRenderer;

pub mod serializer;

trait Peekable {
    fn get_duration(&self) -> gst::ClockTime;
    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>;
}

impl Peekable for gst::Element {
    fn get_duration(&self) -> gst::ClockTime {
        100 * 1000 * gst::MSECOND
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.seek_simple(gst::SeekFlags::FLUSH, time).ok().and_then(|_| {
            self.get_property("last-pixbuf").ok().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
        })
    }
}

pub struct Component {
    pub name: String,
    pub start_time: gst::ClockTime,
    pub end_time: gst::ClockTime,
    component: Box<Peekable>,
    coordinate: (i32,i32),
}

impl Component {
    pub fn new_from_structure(structure: &serializer::ComponentStructure) -> Component {
        match structure.component_type {
            serializer::ComponentType::Video => {
                let mut c = VideoFileComponent::new(structure.entity.as_str(), structure.start_time, structure.coordinate).get_component();
                c.end_time = structure.end_time;
                c
            },
            serializer::ComponentType::Image => {
                let mut c = ImageComponent::new(structure.entity.as_str(), structure.start_time, structure.coordinate).get_component();
                c.end_time = structure.end_time;
                c
            },
            _ => unimplemented!(),
        }
    }
}

pub struct Timeline {
    pub elements: Vec<Box<Component>>,
    pub position: gst::ClockTime,
    pub width: i32,
    pub height: i32,
}

impl Timeline {
    pub fn new(width: i32, height: i32) -> Timeline {
        Timeline {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
        }
    }

    pub fn new_from_structure(structure: &serializer::TimelineStructure) -> Timeline {
        let mut timeline = Timeline::new(structure.size.0, structure.size.1);

        for component in structure.components.iter() {
            timeline.register(Box::new(Component::new_from_structure(component)));
        }

        timeline
    }

    pub fn register(&mut self, elem: Box<Component>) {
        self.elements.push(elem);
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(0, false, 8, self.width, self.height).unwrap() };
        {
            let pixbuf = &mut pixbuf.clone();
            let pixels = unsafe { pixbuf.get_pixels() };

            for p in pixels.chunks_mut(3) {
                p[0] = 0;
                p[1] = 0;
                p[2] = 0;
            }
        }

        for elem in &self.elements {
            if let Some(dest) = elem.component.peek(self.position) {
                &dest.composite(
                    &pixbuf, elem.coordinate.0, elem.coordinate.1,
                    cmp::min(dest.get_width(), self.width - elem.coordinate.0),
                    cmp::min(dest.get_height(), self.height - elem.coordinate.1),
                    elem.coordinate.0.into(), elem.coordinate.1.into(), 1f64, 1f64, 0, 255);
            }
        }

        pixbuf
    }

    pub fn renderer(&self, cr: &cairo::Context) -> gtk::Inhibit {
        cr.set_source_pixbuf(&self.get_current_pixbuf(), 0f64, 0f64);
        cr.paint();
        Inhibit(false)
    }

    pub fn write(&mut self, uri: &str, frames: u64, delta: u64) {
        let avi_renderer = AviRenderer::new(uri, self.width as usize, self.height as usize);

        for i in 0..frames {
            if i % 10 == 0 {
                println!("{} / {}", i, frames);
            }
            &avi_renderer.render_step(&self.get_current_pixbuf(), i*delta*gst::MSECOND);
            self.seek_to(i*delta*gst::MSECOND);
        }

        avi_renderer.render_finish();
    }
}

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

impl Peekable for gtk::Image {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.get_pixbuf()
    }
}

pub struct ImageComponent(pub Component);

impl ImageComponent {
    pub fn new(uri: &str, start_time: gst::ClockTime, coordinate: (i32,i32)) -> ImageComponent {
        let image = gtk::Image::new_from_file(uri);

        ImageComponent(Component {
            name: uri.to_string(),
            start_time: start_time,
            end_time: start_time + 100 * gst::MSECOND,
            coordinate: coordinate,
            component: Box::new(image),
        })
    }

    pub fn get_component(self) -> Component {
        self.0
    }
}

