use std::collections::HashMap;
extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    view: gtk::Grid,
    scrolled: gtk::ScrolledWindow,
    remove_button: gtk::Button,
}

impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        let view = gtk::Grid::new();

        let scrolled = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, width as f64, 1.0, 1.0, width as f64), None);
        scrolled.set_size_request(width, 100);

        let remove_button = gtk::Button::new();
        remove_button.set_label("Remove");

        PropertyViewerWidget {
            view: view,
            scrolled: scrolled,
            remove_button: remove_button,
        }
    }

    pub fn create_ui(&self) {
        self.view.set_column_spacing(10);
        self.view.set_row_spacing(10);
        self.view.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::white());

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add(&self.view);
        vbox.add(&self.remove_button);
        self.scrolled.add(&vbox);
    }

    pub fn connect_remove(&self, cont: Box<Fn()>) {
        self.remove_button.connect_clicked(move |_| cont());
    }

    pub fn clear(&self) {
        for widget in self.view.get_children() {
            self.view.remove(&widget);
        }
    }

    pub fn set_properties<T: Clone>(&self, props: HashMap<String,T>, renderer: Box<Fn(String,T) -> gtk::Widget>) {
        self.clear();

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        let mut props = props.iter().collect::<Vec<_>>();
        props.sort_by_key(|&(x,_)| x);
        for (i, &(ref k,ref v)) in props.iter().enumerate() {
            self.view.attach(&new_label(k.as_str(), gtk::Align::End), 0, i as i32, 1, 1);
            self.view.attach(&renderer(k.to_string(),v.clone().clone()), 1, i as i32, 1, 1);
        }

        self.view.show_all();
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::ScrolledWindow;

    fn to_widget(&self) -> &Self::T {
        &self.scrolled
    }
}
