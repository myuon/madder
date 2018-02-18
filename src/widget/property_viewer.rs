extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    pub view: gtk::Grid,
}

impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        let view = gtk::Grid::new();
        view.set_size_request(width, 100);

        PropertyViewerWidget {
            view: view,
        }
    }

    /*
    fn add_new_column(&self, column_title: &str, column_attribute: &str, column_index: i32) {
        let column = gtk::TreeViewColumn::new();
        column.set_title(column_title);

        let cell = gtk::CellRendererText::new();
        column.pack_start(&cell, true);
        column.add_attribute(&cell, column_attribute, column_index);

        self.view.append_column(&column);
    }*/

    pub fn create_ui(&self) {
        self.view.set_column_spacing(10);
        self.view.set_row_spacing(10);
        self.view.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::white());
//        self.add_new_column("Key", "text", 0);
//        self.add_new_column("Value", "text", 1);
//        self.view.get_column(1).unwrap().get_cells()[0].set_property("editable", &true).unwrap();
    }

    pub fn connect_cell_edited(&self, cont: Box<Fn(&gtk::CellRendererText, gtk::TreePath, &str) + 'static>) {
//        let cell = self.view.get_column(1).unwrap().get_cells()[0].clone().dynamic_cast::<gtk::CellRendererText>().unwrap();
//        cell.connect_edited(move |cell, path, new_text| {
//            cont(cell, path, new_text)
//        });
    }

    pub fn set_properties<T: Clone>(&self, props: Vec<(String, T)>, renderer: Box<Fn(T) -> gtk::Widget>) {
        for widget in self.view.get_children() {
            self.view.remove(&widget);
        }

        let new_label = |label: &str, align: gtk::Align| {
            let w = gtk::Label::new(label);
            w.set_halign(align);
            w
        };

        for (i, &(ref k,ref v)) in props.iter().enumerate() {
            self.view.attach(&new_label(k.as_str(), gtk::Align::End), 0, i as i32 + 1, 1, 1);
            self.view.attach(&renderer(v.clone()), 1, i as i32 + 1, 1, 1);
        }

        self.view.show_all();
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::Grid;

    fn to_widget(&self) -> &Self::T {
        &self.view
    }
}
