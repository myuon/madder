use std::rc::Rc;

extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gdk::prelude::*;

use gtk::prelude::*;
use madder_core::*;

pub fn edit_type_to_widget(self_: &Property, tracker: Vec<i32>, cont: Rc<Fn(Option<Property>, &[i32]) + 'static>) -> gtk::Widget {
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
            entry.connect_changed(move |entry| cont(entry.get_text().and_then(|x| x.parse().ok()).map(I32), &tracker.clone()));
            entry.dynamic_cast().unwrap()
        },
        &Usize(ref i) => {
            let entry = gtk::Entry::new();
            entry.set_text(&i.to_string());
            entry.connect_changed(move |entry| cont(entry.get_text().and_then(|x| x.parse().ok()).map(Usize), &tracker.clone()));
            entry.dynamic_cast().unwrap()
        },
        &Time(ref time) => {
            let entry = gtk::Entry::new();
            entry.set_text(&time.mseconds().unwrap().to_string());
//            let window = gtk::Window::new(gtk::WindowType::Popup);
//            window.add(&gtk::Label::new("piyo"));
            entry.connect_changed(move |entry| {
//                window.show_all();
                cont(entry.get_text().and_then(|x| x.parse::<u64>().ok()).map(gst::ClockTime::from_mseconds).map(Time), &tracker.clone());
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
                cont(dialog.get_filename().unwrap().as_path().to_str().map(|x| FilePath(x.to_string())), &tracker.clone());
                dialog.destroy();
            });
            btn.dynamic_cast().unwrap()
        },
        &Document(ref doc) => {
            let textarea = gtk::TextView::new();
            let buffer = textarea.get_buffer().unwrap();
            buffer.set_text(doc);
            buffer.connect_changed(move |buffer| {
                cont(buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), true).map(Document), &tracker.clone())
            });
            textarea.dynamic_cast().unwrap()
        },
        &Font(ref font) => {
            let fontbtn = gtk::FontButton::new_with_font(font);
            fontbtn.connect_font_set(move |fontbtn| {
                cont(fontbtn.get_font().map(Font), &tracker.clone())
            });
            fontbtn.dynamic_cast().unwrap()
        },
        &Color(ref rgba) => {
            let colorbtn = gtk::ColorButton::new_with_rgba(rgba);
            colorbtn.connect_color_set(move |colorbtn| {
                cont(Some(Color(colorbtn.get_rgba())), &tracker.clone())
            });
            colorbtn.dynamic_cast().unwrap()
        },
        &EffectInfo(ref eff_type, ref transition, ref start_value, ref end_value) => {
            let label = gtk::Label::new(format!("{} -- {:?} --> {}", start_value, eff_type, end_value).as_str());
            label.dynamic_cast().unwrap()
        },
        &Array(ref arr) => {
            let expander = gtk::Expander::new(format!("effects: {}", arr.len()).as_str());
            expander.set_margin_top(5);
            expander.set_margin_bottom(5);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

            for (i,eff) in arr.iter().enumerate() {
                let mut tracker = tracker.clone();
                tracker.push(i as i32);
                vbox.pack_start(&edit_type_to_widget(&eff, tracker, cont.clone()), true, true, 5);
            }

            vbox.set_margin_left(20);
            expander.add(&vbox);
            expander.dynamic_cast().unwrap()
        }
    }
}

pub fn recover_property(dynamic_type: Property, tracker: &[i32], value: Property) -> Property {
    use Property::*;

    match tracker {
        &[] => value,
        &[0,ref tracker..] => {
            match dynamic_type {
                Pair(box x,y) => Pair(Box::new(recover_property(x, &tracker, value)),y),
                _ => unimplemented!(),
            }
        },
        &[1,ref tracker..] => {
            match dynamic_type {
                Pair(x,box y) => Pair(x, Box::new(recover_property(y, &tracker, value))),
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}


