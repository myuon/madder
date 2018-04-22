use std::rc::Rc;

extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

extern crate relm;
use relm::*;

use widget::AsWidget;

pub trait PageI {
    type PageElement;

    fn get_elements(&self) -> Vec<Self::PageElement>;
    fn make_widget(&self, Self::PageElement, usize) -> gtk::Widget;
}

pub struct GridPage {
    widget: gtk::ScrolledWindow,
}

impl GridPage {
    pub fn new<T: Clone>(width: i32, elements: Vec<(String, T)>, make_widget: &Fn(&str, T, usize) -> gtk::Widget) -> GridPage {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, width as f64, 1.0, 1.0, width as f64), None);
        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(5);
        scroll.add(&grid);

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        for (i, &(ref k, ref v)) in elements.iter().enumerate() {
            grid.attach(&new_label(k, gtk::Align::End), 0, i as i32, 1, 1);
            grid.attach(&make_widget(&k,v.clone(),i), 1, i as i32, 1, 1);
        }

        GridPage {
            widget: scroll,
        }
    }
}

impl AsWidget for GridPage {
    type T = gtk::ScrolledWindow;

    fn as_widget(&self) -> &Self::T {
        &self.widget
    }
}

pub trait BoxPageI : PageI {
    fn add_button_cont(&self);
    fn remove_button_cont(&self, usize);
}

pub struct BoxPage {
    widget: gtk::ScrolledWindow,
}

impl BoxPage {
    pub fn new<T: Clone>(width: i32, elements: Vec<T>, make_widget: &Fn(T, usize) -> gtk::Widget, add_button_cont: Box<Fn()>, remove_button_cont: Box<Fn(usize)>) -> BoxPage {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, width as f64, 1.0, 1.0, width as f64), None);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        scroll.add(&vbox);

        let add_button = gtk::Button::new();
        add_button.set_label("Add");
        vbox.pack_start(&add_button, false, false, 0);

        add_button.connect_clicked(move |_| {
            add_button_cont();
        });

        let remove_button_cont = Rc::new(remove_button_cont);
        for (i,v) in elements.iter().enumerate() {
            let widget = make_widget(v.clone(), i);

            let remove_button_cont = remove_button_cont.clone();
            widget.connect_button_press_event(move |_, event| {
                if event.get_button() == 3 {
                    let menu = gtk::Menu::new();
                    let remove_item = {
                        let remove_item = gtk::MenuItem::new_with_label("remove");
                        let remove_button_cont = remove_button_cont.clone();
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

        BoxPage {
            widget: scroll,
        }
    }
}

impl AsWidget for BoxPage {
    type T = gtk::ScrolledWindow;

    fn as_widget(&self) -> &Self::T {
        &self.widget
    }
}

pub struct Model {
    remove_button: gtk::Button,
    width: i32,
}

#[derive(Msg)]
pub enum PropertyMsg {
}

pub struct PropertyViewerWidget {
    model: Model,
    vbox: gtk::Box,
}

impl Update for PropertyViewerWidget {
    type Model = Model;
    type ModelParam = i32;
    type Msg = PropertyMsg;

    fn model(_: &Relm<Self>, width: i32) -> Model {
        Model {
            remove_button: gtk::Button::new(),
            width: width,
        }
    }

    fn update(&mut self, _event: PropertyMsg) {
    }
}

impl Widget for PropertyViewerWidget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(_relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&gtk::Label::new("prop viewer"), false, false, 0);

        PropertyViewerWidget {
            model: model,
            vbox: vbox,
        }
    }
}

impl UpdateNew for PropertyViewerWidget {
    fn new(relm: &Relm<Self>, model: Self::Model) -> Self {
        Widget::view(relm, model)
    }
}

/*
impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        let self_ = PropertyViewerWidget {
            view: gtk::Box::new(gtk::Orientation::Vertical, 0),
            notebook: gtk::Notebook::new(),
            remove_button: gtk::Button::new(),
            width: width,
        };
        self_.create_ui();
        self_
    }

    fn create_ui(&self) {
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

    pub fn append_page<T: AsWidget>(&self, tab_name: &str, page: T) {
        self.notebook.append_page(page.as_widget(), Some(&gtk::Label::new(tab_name)));
        self.notebook.show_all();
    }
}

impl AsWidget for PropertyViewerWidget {
    type T = gtk::Box;

    fn as_widget(&self) -> &Self::T {
        &self.view
    }
}
 */
