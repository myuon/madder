#![feature(box_patterns)]
#![feature(slice_patterns)]
extern crate gstreamer as gst;
extern crate gtk;
extern crate glib;
extern crate gdk;

#[macro_use] extern crate serde_json;

extern crate madder_core;
extern crate madder_util as util;
use madder_core::*;

pub mod widget;
pub mod gtk_impl;
pub mod ui_impl;
pub mod app;
use app::App;

static mut WINDOW_NUMBER: i32 = 0;

fn main() {
    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    use std::env;
    let args = env::args().collect::<Vec<String>>();

    let app =
        if args.len() >= 2 {
            App::new_from_file(&args[1])
        } else {
            App::new_from_json(serde_json::from_value(json!({
                "width": 640,
                "height": 480,
                "length": 90000,
                "components": [
                    {
                        "component_type": "Text",
                        "start_time": 0,
                        "length": 100,
                        "layer_index": 0,
                        "prop": {
                            "coordinate": [50,50],
                            "entity": "[ここにテキストを挿入]",
                        },
                    }
                ],
            })).unwrap())
        };

    App::create_ui(app);

    gtk::main();
}
