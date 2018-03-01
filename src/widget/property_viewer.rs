extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    view: gtk::Notebook,
    remove_button: gtk::Button,
    width: i32,
}

impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        PropertyViewerWidget {
            view: gtk::Notebook::new(),
            remove_button: gtk::Button::new(),
            width: width,
        }
    }

    pub fn create_ui(&self) {
        self.view.set_size_request(self.width, 100);
    }

    pub fn connect_remove(&self, cont: Box<Fn()>) {
        self.remove_button.connect_clicked(move |_| cont());
    }

    pub fn clear(&self) {
        for widget in self.view.get_children() {
            self.view.remove(&widget);
        }
    }

    pub fn add_grid_properties<T: Clone>(&self, tab_name: String, props: Vec<(String,T)>, renderer: Box<Fn(String,T) -> gtk::Widget>) {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, self.width as f64, 1.0, 1.0, self.width as f64), None);
        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(5);
        scroll.add(&grid);

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        for (i, &(ref k, ref v)) in props.iter().enumerate() {
            grid.attach(&new_label(k, gtk::Align::End), 0, i as i32, 1, 1);
            grid.attach(&renderer(k.to_string(), v.clone().clone()), 1, i as i32, 1, 1);
        }

        self.view.append_page(&scroll, Some(&gtk::Label::new(tab_name.as_str())));
        self.view.show_all();
    }

    pub fn add_box_properties<T: Clone>(&self, tab_name: String, props: Vec<T>, renderer: Box<Fn(usize, T) -> gtk::Widget>) {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, self.width as f64, 1.0, 1.0, self.width as f64), None);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        scroll.add(&vbox);

        for (i,v) in props.iter().enumerate() {
            vbox.pack_start(&renderer(i, v.clone()), false, false, 5);
        }

        self.view.append_page(&scroll, Some(&gtk::Label::new(tab_name.as_str())));
        self.view.show_all();
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::Notebook;

    fn to_widget(&self) -> &Self::T {
        &self.view
    }
}
