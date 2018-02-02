use std::{env, thread};
use std::os::raw::c_void;

extern crate gtk;
use gtk::prelude::*;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;
use glib::translate::ToGlibPtr;

extern crate gdk;
extern crate gdk_pixbuf;
use gdk::prelude::*;

fn create_ui(playbin: &gst::Element) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(500,400);
    window.connect_delete_event(|_,_| {
        gtk::main_quit();
        gtk::Inhibit(false)
    });

    let video_window = gtk::DrawingArea::new();
    video_window.set_double_buffered(false);

    let video_overlay = playbin
        .clone()
        .dynamic_cast::<gstv::VideoOverlay>()
        .expect("Failed to cast to gstv::VideoOverlay");

    video_window.connect_realize(move |video_window| {
        let video_overlay = &video_overlay;
        let gdk_window = video_window.get_window().unwrap();

        if !gdk_window.ensure_native() {
            panic!("Cannot create a native window");
        }

        let display_type_name = gdk_window.get_display().get_type().name();
        if display_type_name == "GdkX11Display" {
            extern "C" {
                pub fn gdk_x11_window_get_xid(window: *mut glib::object::GObject) -> *mut c_void;
            }

            unsafe {
                let xid = gdk_x11_window_get_xid(gdk_window.to_glib_none().0);
                video_overlay.set_window_handle(xid as usize);

            }
        }
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&video_window, true, true, 0);

    let render_btn = gtk::Button::new();
    render_btn.set_label("render");

    {
        let playbin = playbin.clone();
        render_btn.connect_clicked(move |_| {
            playbin.set_state(gst::State::Paused).into_result().unwrap();

            let pipeline = gst::Pipeline::new(None);
            let src = gst::ElementFactory::make("appsrc", None).unwrap();
            let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
            let avimux = gst::ElementFactory::make("avimux", None).unwrap();
            let sink = gst::ElementFactory::make("filesink", None).unwrap();
            sink.set_property("location", &glib::Value::from("output.avi")).unwrap();

            pipeline.add_many(&[&src, &videoconvert, &avimux, &sink]).expect("pipeline.add_many failed");
            gst::Element::link_many(&[&src, &videoconvert, &avimux, &sink]).expect("link_many failed");

            let appsrc = src.clone().dynamic_cast::<gsta::AppSrc>().expect("dynamic_cast::<gsta::AppSrc> failed");
            let info = gstv::VideoInfo::new(gstv::VideoFormat::Rgba, 640, 480)
                .fps(gst::Fraction::new(2,1))
                .build()
                .expect("Failed to create video info");

            appsrc.set_caps(&info.to_caps().unwrap());
            appsrc.set_property_format(gst::Format::Time);
            appsrc.set_max_bytes(1);
            appsrc.set_property_block(true);

            thread::spawn(move || {
                for i in 0..100 {
                    let mut buffer = gst::Buffer::with_size(640*480*4).unwrap();

                    {
                        let buffer = buffer.get_mut().unwrap();
                        buffer.set_pts(i * 500 * gst::MSECOND);

                        let mut data = buffer.map_writable().unwrap();

                        let r = if i % 2 == 0 { 0 } else { 255 };
                        for p in data.as_mut_slice().chunks_mut(4) {
                            p[0] = r;
                            p[1] = 128;
                            p[2] = 255;
                            p[3] = 255;
                        }
                    }

                    let r = appsrc.push_buffer(buffer);
                    if r != gst::FlowReturn::Ok {
                        println!("{:?}", r);
                        break;
                    }

                }

                appsrc.end_of_stream().into_result().unwrap();
            });

            pipeline.set_state(gst::State::Playing).into_result().expect("set_state(gst::State::Playing) failed");

            let bus = pipeline.get_bus().expect("pipeline.get_bus failed");
            while let Some(msg) = bus.timed_pop(gst::CLOCK_TIME_NONE) {
                use gst::MessageView;

                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(_) => panic!("MessageView error"),
                    _ => (),
                }
            }

            pipeline.set_state(gst::State::Null).into_result().unwrap();
        });
    }

    vbox.pack_start(&render_btn, false, false, 0);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    {
        let input = gtk::Entry::new();
        input.set_placeholder_text("frame");

        let go_btn = gtk::Button::new();
        go_btn.set_label("Go!");

        hbox.pack_start(&input, false, false, 0);
        hbox.pack_start(&go_btn, false, false, 0);
    }
    vbox.pack_start(&hbox, false, false, 10);

    window.add(&vbox);
    window.show_all();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Give the video filepath as a first argument");
    }

    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    let playbin = gst::ElementFactory::make("playbin", None).unwrap();
    playbin.set_property("uri", &glib::Value::from(format!("file://{}", args[1]).as_str())).unwrap();

    create_ui(&playbin);

    playbin.set_state(gst::State::Playing).into_result().unwrap();

    gtk::main();
}
