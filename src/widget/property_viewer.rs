use std::collections::HashMap;
extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    view: HashMap<String, gtk::Grid>,
    scrolled: gtk::ScrolledWindow,
    remove_button: gtk::Button,
}

impl PropertyViewerWidget {
    pub fn new(width: i32, labels: &[&str]) -> PropertyViewerWidget {
        let mut view = HashMap::new();
        for label in labels {
            let grid = gtk::Grid::new();
            grid.set_column_spacing(10);
            grid.set_row_spacing(10);
            view.insert(label.to_string(), grid);
        }

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
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        for (label, view) in &self.view {
            let expander = gtk::Expander::new(label.as_str());
            expander.add(view);

            vbox.pack_start(&expander, false, false, 5);
        }

        vbox.pack_start(&self.remove_button, false, false, 5);
        self.scrolled.add(&vbox);
    }

    pub fn connect_remove(&self, cont: Box<Fn()>) {
        self.remove_button.connect_clicked(move |_| cont());
    }

    pub fn clear(&self) {
        for (_, view) in &self.view {
            for widget in view.get_children() {
                view.remove(&widget);
            }
        }
    }

    pub fn set_properties<T: Clone>(&self, tab: &str, props: HashMap<String,T>, renderer: Box<Fn(String,T) -> gtk::Widget>) {
        self.clear();

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        let mut props = props.iter().collect::<Vec<_>>();
        props.sort_by_key(|&(x,_)| x);
        for (i, &(ref k,ref v)) in props.iter().enumerate() {
            self.view[tab].attach(&new_label(k.as_str(), gtk::Align::End), 0, i as i32, 1, 1);
            self.view[tab].attach(&renderer(k.to_string(),v.clone().clone()), 1, i as i32, 1, 1);
        }

        for (_, view) in &self.view {
            view.show_all();
        }
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::ScrolledWindow;

    fn to_widget(&self) -> &Self::T {
        &self.scrolled
    }
}
