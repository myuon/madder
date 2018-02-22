use std::rc::Rc;

extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gdk::prelude::*;

use gtk::prelude::*;
use madder_core::*;

pub fn edit_type_to_widget(self_: &Property, tracker: Vec<i32>, cont: Rc<Fn(String, &[i32]) + 'static>) -> gtk::Widget {
    use Property::*;

    match self_ {
        &ReadOnly(ref s) => {
            let label = gtk::Label::new(s.as_str());
            label.set_halign(gtk::Align::Start);
            label.set_margin_top(5);
            label.set_margin_bottom(5);
            label.dynamic_cast().unwrap()
        },
        &I32(ref i) => {
            let entry = gtk::Entry::new();
            entry.set_text(&i.to_string());
            entry.connect_changed(move |entry| cont(entry.get_text().unwrap(), &tracker.clone()));
            entry.dynamic_cast().unwrap()
        },
        &Time(ref time) => {
            let entry = gtk::Entry::new();
            entry.set_text(&time.mseconds().unwrap().to_string());
//            let window = gtk::Window::new(gtk::WindowType::Popup);
//            window.add(&gtk::Label::new("piyo"));
            entry.connect_changed(move |entry| {
//                window.show_all();
                cont(entry.get_text().unwrap(), &tracker.clone());
            });
            entry.dynamic_cast().unwrap()
        },
        &Pair(box ref wx, box ref wy) => {
            let expander = gtk::Expander::new(format!("[{:?},{:?}]", wx, wy).as_str());
            expander.set_margin_top(5);
            expander.set_margin_bottom(5);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

            {
                let mut tracker = tracker.clone();
                tracker.push(0);
                vbox.pack_start(&edit_type_to_widget(&wx, tracker, cont.clone()), true, true, 5);
            }
            {
                let mut tracker = tracker.clone();
                tracker.push(1);
                vbox.pack_start(&edit_type_to_widget(&wy, tracker, cont.clone()), true, true, 5);
            }

            vbox.set_margin_left(20);
            expander.add(&vbox);
            expander.dynamic_cast().unwrap()
        },
        &FilePath(ref path) => {
            let btn = gtk::Button::new();
            btn.set_label(path);
            btn.connect_clicked(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("Entity"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.*");
                    dialog.add_filter(&filter);
                }
                dialog.run();
                cont(dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(), &tracker.clone());
                dialog.destroy();
            });
            btn.dynamic_cast().unwrap()
        },
        &Document(ref doc) => {
            let textarea = gtk::TextView::new();
            let buffer = textarea.get_buffer().unwrap();
            buffer.set_text(doc);
            buffer.connect_changed(move |buffer| {
                cont(buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), true).unwrap(), &tracker.clone())
            });
            textarea.dynamic_cast().unwrap()
        },
        &Color(ref rgba) => {
            let colorbtn = gtk::ColorButton::new_with_rgba(rgba);
            colorbtn.connect_color_set(move |colorbtn| {
                cont(colorbtn.get_rgba().to_string(), &tracker.clone())
            });
            colorbtn.dynamic_cast().unwrap()
        }
    }
}

pub fn read_as_edit_type(dynamic_type: Property, tracker: &[i32], new_text: String) -> Option<Property> {
    use Property::*;

    match tracker {
        &[] => {
            match dynamic_type {
                ReadOnly(s) => Some(ReadOnly(s)),
                I32(_) => new_text.parse::<i32>().ok().map(|x| I32(x)),
                Time(_) => new_text.parse::<u64>().ok().map(|x| Time(gst::ClockTime::from_mseconds(x))),
                FilePath(_) => Some(FilePath(new_text)),
                Document(_) => Some(Document(new_text)),
                Color(_) => new_text.parse().ok().map(|x| Color(x)),
                _ => unimplemented!(),
            }
        },
        &[0,ref tracker..] => {
            match dynamic_type {
                Pair(box x,y) => read_as_edit_type(x, &tracker, new_text).map(|x| Pair(Box::new(x),y)),
                _ => unimplemented!(),
            }
        },
        &[1,ref tracker..] => {
            match dynamic_type {
                Pair(x,box y) => read_as_edit_type(y, &tracker, new_text).map(|y| Pair(x, Box::new(y))),
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}


