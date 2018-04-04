use std::rc::Rc;

extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::AsWidget;

pub trait PageI {
    type PageElement;

    fn get_elements(&self) -> Vec<Self::PageElement>;
    fn make_widget(&self, Self::PageElement, usize) -> gtk::Widget;
}

pub struct GridPage<M: PageI> {
    grid: gtk::Grid,
    widget: gtk::ScrolledWindow,
    model: Option<M>,
}

impl<E, M: PageI<PageElement = (String, E)>> GridPage<M> {
    pub fn new<T: Clone>(width: i32) -> GridPage<M> {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, width as f64, 1.0, 1.0, width as f64), None);
        let grid = gtk::Grid::new();
        grid.set_column_spacing(10);
        grid.set_row_spacing(5);
        scroll.add(&grid);

        GridPage {
            widget: scroll,
            grid: grid,
            model: None,
        }
    }

    pub fn set_model(&mut self, model: M) {
        self.model = Some(model);
    }

    pub fn setup(&self) {
        let model = self.model.unwrap();

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        for (i, &(ref k, ref v)) in model.get_elements().iter().enumerate() {
            self.grid.attach(&new_label(k, gtk::Align::End), 0, i as i32, 1, 1);
            self.grid.attach(&model.make_widget((*k,*v), i), 1, i as i32, 1, 1);
        }
    }
}

impl<M: PageI> AsWidget for GridPage<M> {
    type T = gtk::ScrolledWindow;

    fn as_widget(&self) -> &Self::T {
        &self.widget
    }
}

pub trait BoxPageI : PageI {
    fn add_button_cont(&self);
    fn remove_button_cont(&self, usize);
}

pub struct BoxPage<M: BoxPageI> {
    vbox: gtk::Box,
    widget: gtk::ScrolledWindow,
    add_button: gtk::Button,
    model: Option<M>
}

impl<E, M: 'static + BoxPageI<PageElement = E>> BoxPage<M> {
    pub fn new<T: Clone>(width: i32) -> BoxPage<M> {
        let scroll = gtk::ScrolledWindow::new(&gtk::Adjustment::new(0.0, 0.0, width as f64, 1.0, 1.0, width as f64), None);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        scroll.add(&vbox);

        let add_button = gtk::Button::new();
        add_button.set_label("Add");
        vbox.pack_start(&add_button, false, false, 0);

        BoxPage {
            vbox: vbox,
            widget: scroll,
            add_button: add_button,
            model: None,
        }
    }

    pub fn model(&mut self, model: M) {
        self.model = Some(model);
    }

    pub fn setup(&self) {
        let model = self.model.unwrap();

        self.add_button.connect_clicked(move |_| {
            model.add_button_cont();
        });

        for (i,v) in model.get_elements().iter().enumerate() {
            let widget = model.make_widget(*v.clone(), i);
            let remove_button_cont = Rc::new(|x| model.remove_button_cont(x));

            widget.connect_button_press_event(move |_, event| {
                let remove_button_cont = remove_button_cont.clone();

                if event.get_button() == 3 {
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

            self.vbox.pack_start(&widget, false, false, 0);
        }
    }
}

impl<M: BoxPageI> AsWidget for BoxPage<M> {
    type T = gtk::ScrolledWindow;

    fn as_widget(&self) -> &Self::T {
        &self.widget
    }
}

pub struct PropertyViewerWidget {
    view: gtk::Box,
    notebook: gtk::Notebook,
    remove_button: gtk::Button,
    pub width: i32,
}

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
