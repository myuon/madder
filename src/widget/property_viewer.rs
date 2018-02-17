extern crate gtk;
use gtk::prelude::*;

use widget::WidgetWrapper;

pub struct PropertyViewerWidget {
    pub view: gtk::TreeView,
    pub store: gtk::ListStore,
}

impl PropertyViewerWidget {
    pub fn new(width: i32) -> PropertyViewerWidget {
        let view = gtk::TreeView::new();
        view.set_size_request(width, 100);

        PropertyViewerWidget {
            view: view,
            store: gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]),
        }
    }

    fn add_new_column(&self, column_title: &str, column_attribute: &str, column_index: i32) {
        let column = gtk::TreeViewColumn::new();
        column.set_title(column_title);

        let cell = gtk::CellRendererText::new();
        column.pack_start(&cell, true);
        column.add_attribute(&cell, column_attribute, column_index);

        self.view.append_column(&column);
    }

    pub fn create_ui(&self) {
        self.view.set_model(&self.store);

        self.add_new_column("Key", "text", 0);
        self.add_new_column("Value", "text", 1);
        self.view.get_column(1).unwrap().get_cells()[0].set_property("editable", &true).unwrap();
    }

    pub fn connect_cell_edited(&self, cont: Box<Fn(&gtk::CellRendererText, gtk::TreePath, &str) + 'static>) {
        let cell = self.view.get_column(1).unwrap().get_cells()[0].clone().dynamic_cast::<gtk::CellRendererText>().unwrap();
        cell.connect_edited(move |cell, path, new_text| {
            cont(cell, path, new_text)
        });
    }

    pub fn set_properties(&self, props: Vec<(String, String)>) {
        self.store.clear();

        for (i,j) in props {
            self.store.insert_with_values(None, &[0,1], &[&i,&j]);
        }
    }
}

impl WidgetWrapper for PropertyViewerWidget {
    type T = gtk::TreeView;

    fn to_widget(&self) -> &Self::T {
        &self.view
    }
}
