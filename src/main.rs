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
                components: Box::new([
                    ComponentBuilder::default()
                        .component_type(ComponentType::Text)
                        .start_time(0 * gst::MSECOND)
                        .length(100 * gst::MSECOND)
                        .entity("[ここにテキストを挿入]".to_string())
                        .layer_index(0)
                        .coordinate((50,50))
                        .build().unwrap()
                ]),
            }
        };

    let app = App::new_from_json(&editor);
    App::create_ui(app);

    gtk::main();
}
