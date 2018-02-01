use std::env;
use std::os::raw::c_void;

extern crate gtk;
use gtk::prelude::*;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
use gstv::prelude::*;

extern crate glib;
use glib::translate::ToGlibPtr;

extern crate gdk;
use gdk::prelude::*;

fn create_ui(playbin: &gst::Element) {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
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

    window.add(&video_window);
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
