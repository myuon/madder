#![feature(box_patterns)]
#![feature(slice_patterns)]
extern crate gstreamer as gst;
extern crate gtk;
extern crate glib;
extern crate gdk;

#[macro_use] extern crate serde_json;

extern crate madder_core;
use madder_core::*;

pub mod widget;
pub mod gtk_impl;
pub mod app;
use app::App;

fn main() {
    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    use std::env;
    let args = env::args().collect::<Vec<String>>();

    let editor =
        if args.len() >= 2 {
            EditorStructure::new_from_file(&args[1])
        } else {
            EditorStructure {
                width: 640,
                height: 480,
                length: 90000,
                components: Box::new([
                    serde_json::from_value(json!({
                        "component_type": "Text",
                        "start_time": 0,
                        "length": 100,
                        "entity": "[ここにテキストを挿入]",
                        "layer_index": 0,
                        "coordinate": [50,50],
                    })).unwrap()
                ]),
            }
        };

    let app = App::new_from_json(&editor);
    App::create_ui(app);

    gtk::main();
}
