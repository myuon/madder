use std::rc::Rc;

extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    view: gtk::Box,
    notebook: gtk::Notebook,
    remove_button: gtk::Button,
    width: i32,
}

impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        PropertyViewerWidget {
            view: gtk::Box::new(gtk::Orientation::Vertical, 0),
            notebook: gtk::Notebook::new(),
            remove_button: gtk::Button::new(),
            width: width,
        }
    }

    pub fn create_ui(&self) {
        self.notebook.set_size_request(self.width, 100);
        self.remove_button.set_label("Remove");

        self.view.pack_start(&self.notebook, true, true, 0);
        self.view.pack_start(&self.remove_button, false, false, 5);
    }

    pub fn connect_remove(&self, cont: Box<Fn()>) {
        self.remove_button.connect_clicked(move |_| cont());
    }

    pub fn clear(&self) {
        for widget in self.notebook.get_children() {
            self.notebook.remove(&widget);
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

        self.notebook.append_page(&scroll, Some(&gtk::Label::new(tab_name.as_str())));
        self.notebook.show_all();
    }

    pub fn add_box_properties<T: Clone>(&self, tab_name: String, props: Vec<T>, renderer: Box<Fn(usize, T) -> gtk::Widget>, add_button_cont: Box<Fn()>, remove_button_cont: Box<Fn(usize)>) {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, self.width as f64, 1.0, 1.0, self.width as f64), None);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        scroll.add(&vbox);

        let add_button = gtk::Button::new();
        add_button.set_label("Add");
        add_button.connect_clicked(move |_| {
            add_button_cont();
        });
        vbox.pack_start(&add_button, false, false, 0);

        let remove_button_cont = Rc::new(remove_button_cont);
        for (i,v) in props.iter().enumerate() {
            let widget = renderer(i, v.clone());
            let remove_button_cont = remove_button_cont.clone();

            widget.connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    let remove_button_cont = remove_button_cont.clone();

                    let menu = gtk::Menu::new();
                    let remove_item = {
                        let remove_item = gtk::MenuItem::new_with_label("remove");
                        remove_item.connect_activate(move |_| {
                            remove_button_cont(i);
                        });

                        remove_item
                    };
                    let move_up_item = {
                        let item = gtk::MenuItem::new_with_label("move up");
                        item
                    };
                    let move_down_item = {
                        let item = gtk::MenuItem::new_with_label("move down");
                        item
                    };

                    menu.append(&remove_item);
                    menu.append(&move_up_item);
                    menu.append(&move_down_item);

                    menu.popup_easy(0, gtk::get_current_event_time());
                    menu.show_all();
                }

                Inhibit(false)
            });

            vbox.pack_start(&widget, false, false, 0);
        }

        self.notebook.append_page(&scroll, Some(&gtk::Label::new(tab_name.as_str())));
        self.notebook.show_all();
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::Box;

    fn to_widget(&self) -> &Self::T {
        &self.view
    }
}
