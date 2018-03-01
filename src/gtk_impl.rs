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
    EffectType,
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
        &Choose(ref name, ref i) => {
            let make_widget = |items: Vec<String>| {
                let combo = gtk::ComboBoxText::new();
                for item in items {
                    combo.append_text(item.as_str());
                }
                combo.set_active(i.clone() as i32);

                let name = name.to_string();
                combo.connect_changed(move |combo| {
                    cont(Some(Choose(name.clone(), combo.get_active())), &tracker.clone());
                });

                combo.dynamic_cast().unwrap()
            };

            match name {
                _ if name == &format!("{:?}", Tracker::EffectType) => make_widget(EffectType::types().iter().map(|x| format!("{:?}", x)).collect()),
                _ if name == &format!("{:?}", Tracker::Transition) => make_widget(Transition::transitions().iter().map(|x| format!("{:?}", x)).collect()),
                _ => unimplemented!(),
            }
        },
        &EffectInfo(ref eff_type, ref transition, ref start_value, ref end_value) => {
            let expander = gtk::Expander::new(format!("{:?}", eff_type).as_str());
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            expander.add(&vbox);
            vbox.set_margin_left(10);

            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::EffectType);
                vbox.pack_start(
                    &edit_type_to_widget(
                        &Choose(
                            format!("{:?}", Tracker::EffectType),
                            EffectType::types().iter().position(|i| i == eff_type).unwrap() as i32,
                        ),
                        tracker, cont.clone()),
                    true, true, 0);
            }

            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::Transition);
                vbox.pack_start(
                    &edit_type_to_widget(
                        &Choose(
                            format!("{:?}", Tracker::Transition),
                            Transition::transitions().iter().position(|i| i == transition).unwrap() as i32,
                        ),
                        tracker, cont.clone()),
                    true, true, 0);
            }

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

            expander.dynamic_cast().unwrap()
        },
    }
}

pub fn recover_property(dynamic_type: Property, tracker: &[Tracker], value: Property) -> Property {
    use Property::*;

    match tracker {
        &[] => value,
        &[Tracker::X, ref tracker..] => {
            match dynamic_type {
                Pair(box x,y) => Pair(Box::new(recover_property(x, &tracker, value)),y),
                _ => unimplemented!(),
            }
        },
        &[Tracker::Y, ref tracker..] => {
            match dynamic_type {
                Pair(x,box y) => Pair(x, Box::new(recover_property(y, &tracker, value))),
                _ => unimplemented!(),
            }
        },
        &[Tracker::EffectType, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) =>
                    EffectInfo(
                        EffectType::types()[recover_property(
                            Choose(
                                format!("{:?}", Tracker::EffectType),
                                EffectType::types().iter().position(|i| i == &effect_type).unwrap() as i32,
                            ),
                            &tracker, value
                        ).as_choose().unwrap() as usize].clone(),
                        transition, start_value, end_value),
                _ => unimplemented!(),
            }
        },
        &[Tracker::Transition, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) =>
                    EffectInfo(
                        effect_type,
                        Transition::transitions()[recover_property(
                            Choose(
                                format!("{:?}", Tracker::Transition),
                                Transition::transitions().iter().position(|i| i == &transition).unwrap() as i32,
                            ),
                            &tracker, value
                        ).as_choose().unwrap() as usize].clone(),
                        start_value, end_value),
                _ => unimplemented!(),
            }
        },
        &[Tracker::StartValue, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) => EffectInfo(effect_type, transition, recover_property(F64(start_value), &tracker, value).as_f64().unwrap(), end_value),
                _ => unimplemented!(),
            }
        },
        &[Tracker::EndValue, ref tracker..] => {
            match dynamic_type {
                EffectInfo(effect_type, transition, start_value, end_value) => EffectInfo(effect_type, transition, start_value, recover_property(F64(end_value), &tracker, value).as_f64().unwrap()),
                _ => unimplemented!(),
            }
        },
    }
}


