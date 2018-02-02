use std::env;

extern crate gtk;
use gtk::prelude::*;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate glib;

extern crate gdk;
extern crate gdk_pixbuf;

fn create_ui(path: &String) {
    let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
    //    src.set_property("location", &glib::Value::from(format!("file://{}", path).as_str())).unwrap();
    let tee = gst::ElementFactory::make("tee", None).unwrap();

    let pipeline = gst::Pipeline::new(None);
    let (sink,widget) = if let Some(gtkglsink) = gst::ElementFactory::make("gtkglsink", None) {
        let glsinkbin = gst::ElementFactory::make("glsinkbin", None).unwrap();
        glsinkbin.set_property("sink", &gtkglsink.to_value()).unwrap();

        let widget = gtkglsink.get_property("widget").unwrap();
        (glsinkbin, widget.get::<gtk::Widget>().unwrap())
    } else {
        let sink = gst::ElementFactory::make("gtksink", None).unwrap();
        let widget = sink.get_property("widget").unwrap();
        (sink, widget.get::<gtk::Widget>().unwrap())
    };
    sink.set_property("async", &glib::Value::from(&false)).unwrap();

    let appsink = gst::ElementFactory::make("appsink", None).unwrap();
    appsink.set_property("async", &glib::Value::from(&false)).unwrap();

    {
        let appsink = appsink.clone();
        let appsink = appsink.dynamic_cast::<gsta::AppSink>().unwrap();
        appsink.set_caps(&gst::Caps::new_simple(
            "video/x-raw",
            &[
                ("format", &gstv::VideoFormat::Rgb.to_string()),
            ]
        ));
    }

    pipeline.add_many(&[&src, &tee, &appsink, &sink] as &[&gst::Element]).unwrap();
    src.link(&tee).unwrap();
    tee.link_pads("src_1", &sink, "sink").unwrap();
    tee.link_pads("src_2", &appsink, "sink").unwrap();

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(500,400);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&widget, true, true, 0);

    let label = gtk::Label::new("Position: 00:00:00");
    vbox.pack_start(&label, true, true, 5);

    let btn = gtk::Button::new();
    btn.set_label("render");

    {
        let pipeline = pipeline.clone();
        let appsink = appsink.clone();
        btn.connect_clicked(move |_| {
            pipeline.set_state(gst::State::Paused).into_result().unwrap();

            let sample = appsink.emit("pull-preroll", &[]).unwrap().unwrap().get::<gst::Sample>().unwrap();
            let caps = sample.get_caps().unwrap();
            let s = caps.get_structure(0).unwrap();
            let buffer = sample.get_buffer().unwrap();

            let width = s.get_value("width").unwrap().get::<i32>().unwrap();
            let height = s.get_value("height").unwrap().get::<i32>().unwrap();

            let bmap = buffer.map_readable().unwrap();

            fn round_up_4(num: i32) -> i32 {
                (num+3) & !3
            }

            let pixbuf = gdk_pixbuf::Pixbuf::new_from_vec(bmap.as_slice().to_vec(), 0, false, 8, width, height, round_up_4(width*3));

            {
                let pipeline = gst::Pipeline::new(None);
                let videoconvert = gst::ElementFactory::make("videoconvert", None).unwrap();
                let avimux = gst::ElementFactory::make("avimux", None).unwrap();
                let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();
                appsrc.set_property("emit-signals", &glib::Value::from(&true)).unwrap();

                let sink = gst::ElementFactory::make("filesink", None).unwrap();
                sink.set_property("location", &glib::Value::from("output.avi")).unwrap();

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

                for i in 0..20 {
                    println!("{}",i);
                    let mut buffer = gst::Buffer::with_size((width as usize)*(height as usize)*3).unwrap();

                    {
                        let buffer = buffer.get_mut().unwrap();
                        buffer.set_pts(i * 500 * gst::MSECOND);

                        let mut data = buffer.map_writable().unwrap();

                        let r = if i % 2 == 0 { 0 } else { 255 };
                        for p in data.as_mut_slice().chunks_mut(3) {
                            p[0] = r;
                            p[1] = 128;
                            p[2] = 255;
                        }
                    }

                    let r = appsrc.push_buffer(buffer);
                    if r != gst::FlowReturn::Ok {
                        println!("{:?}", r);
                        break;
                    }

                }

                for i in 20..30 {
                    println!("{}",i);
                    let mut buffer = gst::Buffer::with_size((width as usize)*(height as usize)*3).unwrap();
                    {
                        let buffer = buffer.get_mut().unwrap();
                        buffer.set_pts(500 * i * gst::MSECOND);

                        let mut data = buffer.map_writable().unwrap();
                        let mut data = data.as_mut_slice();
                        let pixels = unsafe { pixbuf.get_pixels() };

                        use std::io::Write;
                        data.write_all(pixels).unwrap();
                    }
                    appsrc.push_buffer(buffer).into_result().unwrap();
                }

                appsrc.end_of_stream().into_result().unwrap();
            }
        });
    }
    vbox.pack_start(&btn, true, true, 5);

    window.add(&vbox);
    window.show_all();

    {
        let pipeline = pipeline.clone();
        gtk::timeout_add(500, move || {
            let pipeline = &pipeline.clone();
            let position = pipeline.query_position::<gst::ClockTime>().unwrap_or_else(|| 0.into());
            label.set_text(&format!("Position: {:.0}", position));

            glib::Continue(true)
        });
    }

    window.connect_delete_event(move |_,_| {
        gtk::main_quit();
        Inhibit(false)
    });

    pipeline.set_state(gst::State::Playing).into_result().unwrap();

    let bus = pipeline.get_bus().unwrap();
    bus.add_watch(move |_,msg| {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => gtk::main_quit(),
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {:?}",
                    err.get_error(),
                    err.get_debug(),
                );
            }
            _ => (),
        };

        glib::Continue(true)
    });

    window.connect_unrealize(move |_| {
        pipeline.set_state(gst::State::Null).into_result().unwrap();
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
