use std::rc::Rc;

extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

extern crate serde_json;

use madder_core::*;
use widget::*;

pub fn attribute_to_widget_type(attr: Attribute) -> WidgetType {
    use Attribute::*;

    match attr {
        ReadOnly(s) => WidgetType::Label(s),
        I32(n) => WidgetType::NumberEntry(From::from(n)),
        F64(n) => WidgetType::NumberEntry(serde_json::Number::from_f64(n).unwrap()),
        Usize(n) => WidgetType::NumberEntry(From::from(n)),
        Time(n) => WidgetType::NumberEntry(serde_json::Number::from_f64(n.mseconds().unwrap() as f64).unwrap()),
        Pair(box x, box y) => {
            let widget_x = attribute_to_widget_type(x);
            let widget_y = attribute_to_widget_type(y);

            WidgetType::Expander(
                format!(
                    "({},{})",
                    if let WidgetType::NumberEntry(x) = &widget_x { x.to_string() } else { "-".to_string() },
                    if let WidgetType::NumberEntry(y) = &widget_y { y.to_string() } else { "-".to_string() },
                ),
                Box::new(WidgetType::VBox(vec![
                    widget_x,
                    widget_y,
                ]))
            )
        },
        FilePath(path) => WidgetType::FileChooser(path),
        Document(doc) => WidgetType::TextArea(doc),
        Font(font) => WidgetType::Font(font),
        Color(color) => WidgetType::Color(color),
        Choose(options, index) => WidgetType::Choose(options, index),
        Sequence(seq) => WidgetType::VBox(seq.into_iter().map(attribute_to_widget_type).collect()),
    }
}

pub fn widget_type_to_value(widget_type: WidgetType) -> serde_json::Value {
    match widget_type {
        WidgetType::NumberEntry(label) => json!(label),
        WidgetType::TextEntry(label) => json!(label),
        WidgetType::Choose(_, index) => json!(index),
        WidgetType::Label(_) => unreachable!(),
        WidgetType::Grid(vec) => {
            json!(vec.into_iter().map(|(key,widget_type)| {
                (key, widget_type_to_value(widget_type))
            }).collect::<Vec<_>>())
        },
        WidgetType::VBox(_) => unimplemented!(),
        WidgetType::Expander(_,_) => unimplemented!(),
        WidgetType::FileChooser(_) => unimplemented!(),
        WidgetType::TextArea(_) => unimplemented!(),
        WidgetType::Font(_) => unimplemented!(),
        WidgetType::Color(_) => unimplemented!(),
    }
}

#[derive(Clone, Debug)]
pub enum Tracker {
    X,
    Y,
    EffectType,
    Transition,
}

pub fn edit_type_as_widget(self_: &Attribute, tracker: Vec<Tracker>, cont: Rc<Fn(Option<Attribute>, &Vec<Tracker>)>) -> gtk::Widget {
    use Attribute::*;

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
                vbox.pack_start(&edit_type_as_widget(&wx, tracker, cont.clone()), true, true, 5);
            }
            {
                let mut tracker = tracker.clone();
                tracker.push(Tracker::Y);
                vbox.pack_start(&edit_type_as_widget(&wy, tracker, cont), true, true, 5);
            }

            vbox.set_margin_start(20);
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
        &Choose(ref labels, ref i) => {
            let combo = gtk::ComboBoxText::new();
            for item in labels {
                combo.append_text(item.as_str());
            }
            combo.set_active(i.unwrap() as i32);

            let labels = Rc::new(labels.clone());
            let labels = labels.clone();
            combo.connect_changed(move |combo| {
                cont(Some(Choose((*labels).clone(), Some(combo.get_active() as usize))), &tracker.clone());
            });

            combo.dynamic_cast().unwrap()
        },
        &Sequence(_) => {
            gtk::Label::new("sequence here").dynamic_cast().unwrap()
        }
    }
}

pub fn recover_property(dynamic_type: Attribute, tracker: &[Tracker], value: Attribute) -> Attribute {
    use Attribute::*;

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
        &[Tracker::EffectType] => value,
        &[Tracker::Transition] => value,
        _ => unimplemented!(),
    }
}

