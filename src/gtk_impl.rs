use std::rc::Rc;

extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gdk::prelude::*;

use gtk::prelude::*;
use madder_core::*;

#[derive(Clone, Debug)]
pub enum Tracker {
    X,
    Y,
    Transition,
    StartValue,
    EndValue,
}

pub fn edit_type_to_widget(self_: &Property, tracker: Vec<Tracker>, cont: Rc<Fn(Option<Property>, &Vec<Tracker>) + 'static>) -> gtk::Widget {
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
        &F64(ref i) => {
            let entry = gtk::Entry::new();
            entry.set_text(&i.to_string());
            entry.connect_changed(move |entry| cont(entry.get_text().and_then(|x| x.parse().ok()).map(F64), &tracker.clone()));
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
                tracker.push(Tracker::X);
                vbox.pack_start(&edit_type_to_widget(&wx, tracker, cont.clone()), true, true, 5);
            }
            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::Y);
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
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            vbox.pack_start(&gtk::Label::new(format!("{:?}", eff_type).as_str()), true, true, 0);

            let combo = gtk::ComboBoxText::new();
            for trans in Transition::transitions() {
                combo.append_text(&format!("{:?}", trans).as_str());
            }
            combo.set_active(Transition::transitions().iter().position(|t| t == transition).unwrap() as i32);

            vbox.pack_start(&combo, true, true, 0);

            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::StartValue);
                vbox.pack_start(&edit_type_to_widget(&F64(*start_value), tracker, cont.clone()), true, true, 0);
            }

            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::EndValue);
                vbox.pack_start(&edit_type_to_widget(&F64(*end_value), tracker, cont.clone()), true, true, 0);
            }

            vbox.dynamic_cast().unwrap()
        },
    }
}

pub fn recover_property(dynamic_type: Property, tracker: &[Tracker], value: Property) -> Property {
    use Property::*;
    use self::Tracker::*;

    match tracker {
        &[] => value,
        &[X, ref tracker..] => {
            match dynamic_type {
                Pair(box x,y) => Pair(Box::new(recover_property(x, &tracker, value)),y),
                _ => unimplemented!(),
            }
        },
        &[Y, ref tracker..] => {
            match dynamic_type {
                Pair(x,box y) => Pair(x, Box::new(recover_property(y, &tracker, value))),
                _ => unimplemented!(),
            }
        },
        &[StartValue, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) => EffectInfo(effect_type, transition, recover_property(F64(start_value), &tracker, value).as_f64().unwrap(), end_value),
                _ => unimplemented!(),
            }
        },
        &[EndValue, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) => EffectInfo(effect_type, transition, start_value, recover_property(F64(end_value), &tracker, value).as_f64().unwrap()),
                _ => unimplemented!(),
            }
        },
        _ => unimplemented!(),
    }
}


