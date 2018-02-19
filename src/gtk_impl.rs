use std::rc::Rc;

extern crate gtk;
use gtk::prelude::*;

use madder_core::*;

pub fn edit_type_to_widget(self_: &EditType, tracker: Vec<i32>, cont: Rc<Fn(String, &[i32]) + 'static>) -> gtk::Widget {
    use EditType::*;

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
        &U64(ref i) => {
            let entry = gtk::Entry::new();
            entry.set_text(&i.to_string());
            entry.connect_changed(move |entry| cont(entry.get_text().unwrap(), &tracker.clone()));
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
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();
                cont(dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(), &tracker.clone());
                dialog.destroy();
            });
            btn.dynamic_cast().unwrap()
        },
    }
}

pub fn read_as_edit_type(dynamic_type: EditType, tracker: &[i32], new_text: String) -> EditType {
    use EditType::*;

    match tracker {
        &[] => {
            match dynamic_type {
                ReadOnly(s) => ReadOnly(s),
                I32(_) => I32(new_text.parse::<i32>().unwrap()),
                U64(_) => U64(new_text.parse::<u64>().unwrap()),
                FilePath(_) => FilePath(new_text),
                _ => unimplemented!(),
            }
        },
        &[0,ref tracker..] => {
            match dynamic_type {
                Pair(box x,y) => Pair(Box::new(read_as_edit_type(x, &tracker, new_text)), y),
                _ => unimplemented!(),
            }
        },
        &[1,ref tracker..] => {
            match dynamic_type {
                Pair(x,box y) => Pair(x, Box::new(read_as_edit_type(y, &tracker, new_text))),
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}


