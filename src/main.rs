use std::env;
use std::rc::Rc;
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
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

struct AviRenderer {
    appsrc: gsta::AppSrc,
    size: (usize, usize),
}

impl AviRenderer {
    fn new(uri: &str, width: usize, height: usize) -> AviRenderer {
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
                        gtk::main_quit();
                    },
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {:?}",
                            err.get_error(),
                            err.get_debug(),
                        );
                        pipeline.set_state(gst::State::Null).into_result().unwrap();
                        gtk::main_quit();
                    }
                    _ => (),
                };

                glib::Continue(true)
            });
        }

        pipeline.set_state(gst::State::Playing).into_result().unwrap();

        AviRenderer {
            appsrc: appsrc,
            size: (width,height),
        }
    }

    fn render_step(&self, pixbuf: &gdk_pixbuf::Pixbuf, time: gst::ClockTime) {
        let mut buffer = gst::Buffer::with_size(self.size.0*self.size.1*3).unwrap();
        {
            let buffer = buffer.get_mut().unwrap();
            buffer.set_pts(time);

            let mut data = buffer.map_writable().unwrap();
            let mut data = data.as_mut_slice();
            let pixels = unsafe { pixbuf.get_pixels() };

            use std::io::Write;
            data.write_all(pixels).unwrap();
        }
        self.appsrc.push_buffer(buffer).into_result().unwrap();
    }

    fn render_finish(&self) {
        self.appsrc.end_of_stream().into_result().unwrap();

        println!("Rendering finished!");
    }
}

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

struct Component {
    start_time: gst::ClockTime,
    end_time: gst::ClockTime,
    component: Box<Peekable>,
}

struct Timeline {
    elements: Vec<Box<Component>>,
    position: gst::ClockTime,
    width: i32,
    height: i32,
}

impl Timeline {
    fn new(width: i32, height: i32) -> Timeline {
        Timeline {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
        }
    }

    fn register(&mut self, elem: Box<Component>) {
        self.elements.push(elem);
    }

    fn seek_to(&mut self, time: gst::ClockTime) {
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
                    &pixbuf, 0, 0,
                    cmp::min(dest.get_width(), self.width), cmp::min(dest.get_height(), self.height),
                    0f64, 0f64, 1f64, 1f64, 0, 255);
            }
        }

        pixbuf
    }

    fn renderer(&self, cr: &cairo::Context) -> gtk::Inhibit {
        cr.set_source_pixbuf(&self.get_current_pixbuf(), 0f64, 0f64);
        cr.paint();
        Inhibit(false)
    }

    fn write(&mut self, uri: &str, frames: u64, delta: u64) {
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

struct VideoTestComponent(Component);

impl VideoTestComponent {
    fn new() -> VideoTestComponent {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        pipeline.add_many(&[&src, &pixbufsink]).unwrap();
        src.link(&pixbufsink).unwrap();

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        VideoTestComponent(Component {
            start_time: 0 * gst::MSECOND,
            end_time: 100 * gst::MSECOND,
            component: Box::new(pixbufsink),
        })
    }
}

struct VideoFileComponent(Component);

impl VideoFileComponent {
    fn new(uri: &str) -> VideoFileComponent {
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
            start_time: 0 * gst::MSECOND,
            end_time: 100 * gst::MSECOND,
            component: Box::new(pixbufsink),
        })
    }
}

fn create_ui(uri: &String) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(640,600);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let canvas = gtk::DrawingArea::new();
    canvas.set_size_request(640, 480);

    vbox.pack_start(&canvas, true, true, 0);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let entry = gtk::Entry::new();
    let go_btn = gtk::Button::new();

    hbox.pack_start(&entry, true, true, 0);
    hbox.pack_start(&go_btn, true, true, 5);
    vbox.pack_start(&hbox, true, true, 5);

    let btn = gtk::Button::new();
    btn.set_label("render");

    let timeline: Rc<RefCell<Timeline>> = Rc::new(RefCell::new(Timeline::new(640,480)));

    {
        let timeline: &RefCell<Timeline> = timeline.borrow();
//        timeline.borrow_mut().register(Box::new(VideoTestComponent::new().0));
        timeline.borrow_mut().register(Box::new(VideoFileComponent::new(uri).0));
    }

    {
        let timeline = timeline.clone();
        btn.connect_clicked(move |_| {
            let timeline: &RefCell<Timeline> = &timeline.borrow();
            timeline.borrow_mut().write("output/output.avi", 100, 5);
        });
    }

    {
        let timeline: Rc<RefCell<Timeline>> = timeline.clone();
        canvas.connect_draw(move |_,cr| {
            let timeline: &RefCell<Timeline> = timeline.borrow();
            timeline.borrow_mut().renderer(cr)
        });
    }

    {
        let entry = entry.clone();
        let entry = Rc::new(entry);

        let timeline: Rc<RefCell<Timeline>> = timeline.clone();

        go_btn.set_label("Go");
        go_btn.connect_clicked(move |_| {
            if let Ok(time) = entry.get_text().unwrap().parse::<u64>() {
                let timeline: &RefCell<Timeline> = timeline.borrow();
                timeline.borrow_mut().seek_to(time * gst::MSECOND);
            }
        });
    }

    vbox.pack_start(&btn, true, true, 5);

    let fixed = gtk::Fixed::new();
    {
        fixed.set_size_request(640,100);

        let evbox = gtk::EventBox::new();
        evbox.set_size_request(100,30);

        {
            let label = gtk::Label::new(format!("{}", uri).as_str());
            label.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::blue());

            evbox.add(&label);

            let evbox = evbox.clone();
            let fixed = fixed.clone();
            let offset: Rc<Cell<i32>> = Rc::new(Cell::new(0));

            {
                let offset = offset.clone();
                evbox.connect_button_press_event(move |evbox,button| {
                    let (rx,_) = evbox.get_parent().unwrap().get_window().unwrap().get_position();
                    let (x,_) = button.get_position();
                    let offset: &Cell<i32> = offset.borrow();
                    offset.set(rx + x as i32);
                    Inhibit(false)
                });
            }
            evbox.connect_motion_notify_event(move |evbox,motion| {
                let (rx,_) = motion.get_window().unwrap().get_position();
                let (x,_) = motion.get_position();
                let offset: &Cell<i32> = offset.borrow();
                let x_max = evbox.get_parent().unwrap().get_allocation().width - evbox.get_allocation().width;

                fixed.move_(evbox, cmp::max(cmp::min(rx + x as i32 - offset.get(), x_max), 0), 50);
                Inhibit(false)
            });
        }

        fixed.put(&evbox, 0, 50);
    }

    vbox.pack_start(&fixed, true, true, 5);

    window.add(&vbox);
    window.show_all();

    window.connect_delete_event(move |_,_| {
        gtk::main_quit();
        Inhibit(false)
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Give the video filepath as a first argument");
    }

    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    create_ui(&args[1]);
    gtk::main();
}
