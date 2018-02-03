use std::env;
use std::rc::Rc;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;

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
    fn init(width: usize, height: usize) -> AviRenderer {
        let pipeline = gst::Pipeline::new(None);
        let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let avimux = gst::ElementFactory::make("avimux", None).unwrap();
        let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
        appsrc.set_property("emit-signals", &glib::Value::from(&true)).unwrap();

        let sink = gst::ElementFactory::make("filesink", None).unwrap();
        sink.set_property("location", &glib::Value::from("output/output.avi")).unwrap();

        pipeline.add_many(&[&appsrc, &videoconvert, &avimux, &sink]).unwrap();
        gst::Element::link_many(&[&appsrc, &videoconvert, &avimux, &sink]).unwrap();

        let appsrc = appsrc.clone().dynamic_cast::<gsta::AppSrc>().unwrap();
        let info = gstv::VideoInfo::new(gstv::VideoFormat::Rgb, width as u32, height as u32).fps(gst::Fraction::new(2,1)).build().unwrap();
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

    fn render(&self, pixbuf: &gdk_pixbuf::Pixbuf, frames: u64) {
        for i in 0..frames {
            let mut buffer = gst::Buffer::with_size(self.size.0*self.size.1*3).unwrap();
            {
                let buffer = buffer.get_mut().unwrap();
                buffer.set_pts(500 * i * gst::MSECOND);

                let mut data = buffer.map_writable().unwrap();
                let mut data = data.as_mut_slice();
                let pixels = unsafe { pixbuf.get_pixels() };

                use std::io::Write;
                data.write_all(pixels).unwrap();
            }
            self.appsrc.push_buffer(buffer).into_result().unwrap();
        }

        self.appsrc.end_of_stream().into_result().unwrap();

        println!("Rendering finished!");
    }
}

#[derive(Debug, Clone)]
enum Component {
    SinkContainer(gst::Element),
}

impl Component {
    fn new_from_gdkpixbufsink(sink: gst::Element) -> Component {
        Component::SinkContainer(sink)
    }

    fn request(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        use Component::*;

        match self {
            &SinkContainer(ref appsink) => {
                appsink.seek_simple(gst::SeekFlags::FLUSH, time).ok();
                self.peek()
            },
        }
    }

    fn peek(&self) -> Option<gdk_pixbuf::Pixbuf> {
        use Component::*;

        match self {
            &SinkContainer(ref sink) => {
//                sink.emit("preroll-pixbuf", &[]).unwrap().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
                sink.get_property("last-pixbuf").ok().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
                /*
                appsink.emit("pull-preroll", &[]).ok().and_then(|emitted| {
                    let sample = emitted.unwrap().get::<gst::Sample>().unwrap();
                    let caps = sample.get_caps().unwrap();
                    let s = caps.get_structure(0).unwrap();
                    let buffer = sample.get_buffer().unwrap();

                    let width = s.get_value("width").unwrap().get::<i32>().unwrap();
                    let height = s.get_value("height").unwrap().get::<i32>().unwrap();

                    let bmap = buffer.map_readable().unwrap();

                    fn round_up_4(num: i32) -> i32 {
                        (num+3) & !3
                    }

                    Some(gdk_pixbuf::Pixbuf::new_from_vec(bmap.as_slice().to_vec(), 0, false, 8, width, height, round_up_4(width*3)))
                })
                 */
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Timeline {
    elements: Vec<Component>,
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

    fn register(&mut self, elem: Component) {
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
            if let Some(dest) = elem.peek() {
                &dest.composite(&pixbuf, 0, 0, dest.get_width(), dest.get_height(), 0f64, 0f64, 1f64, 1f64, 0, 255);
            }
        }

        pixbuf
    }

    fn renderer(&self, cr: &cairo::Context) -> gtk::Inhibit {
        println!("{}", self.position);
        cr.set_source_pixbuf(&self.get_current_pixbuf(), 0 as f64, 0 as f64);
        cr.paint();
        Inhibit(false)
    }
}

fn create_ui(path: &String) {
    let pipeline = gst::Pipeline::new(None);

    let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
//    src.set_property("location", &glib::Value::from(path)).unwrap();

    let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();
    pixbufsink.set_property("emit_signals", &glib::Value::from(&true)).unwrap();

    pipeline.add_many(&[&src, &pixbufsink]).unwrap();
    src.link(&pixbufsink).unwrap();

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
        timeline.borrow_mut().register(Component::new_from_gdkpixbufsink(pixbufsink));
    }

    {
        let timeline = timeline.clone();
        btn.connect_clicked(move |_| {
            let timeline: &RefCell<Timeline> = &timeline.borrow();
            let pixbuf = timeline.borrow().elements[0].peek().unwrap();
            let renderer = AviRenderer::init(pixbuf.get_width() as usize, pixbuf.get_height() as usize);
            renderer.render(&pixbuf, 10);
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

    window.add(&vbox);
    window.show_all();

    window.connect_delete_event(move |_,_| {
        gtk::main_quit();
        Inhibit(false)
    });

    pipeline.set_state(gst::State::Paused).into_result().unwrap();
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
