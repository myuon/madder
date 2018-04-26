extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

extern crate relm;
use relm::*;

pub enum WidgetType {
    Entry(String),
    Choose(Vec<String>, Option<usize>),
    Label(String),
    VBox(Vec<WidgetType>),
    Expander(String, Box<WidgetType>),
    FileChooser(String),
    TextArea(String),
    Font(String),
    Color(gdk::RGBA),
}

impl WidgetType {
    fn to_widget(&self) -> gtk::Widget {
        use self::WidgetType::*;

        match self {
            Entry(label) => {
                let entry = gtk::Entry::new();
                entry.set_text(label);
                entry.dynamic_cast().unwrap()
            },
            Choose(options, index) => {
                let combo = gtk::ComboBoxText::new();
                for item in options {
                    combo.append_text(item.as_str());
                }

                if let Some(index) = index {
                    combo.set_active(*index as i32);
                }
                combo.dynamic_cast().unwrap()
            },
            Label(label) => {
                let label = gtk::Label::new(label.as_str());
                label.dynamic_cast().unwrap()
            },
            VBox(vec) => {
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

                for widget_type in vec {
                    vbox.pack_start(&widget_type.to_widget(), false, false, 0);
                }

                vbox.dynamic_cast().unwrap()
            },
            Expander(label, box widget_type) => {
                let expander = gtk::Expander::new(label.as_str());
                expander.set_margin_top(5);
                expander.set_margin_bottom(5);

                expander.add(&widget_type.to_widget());
                expander.dynamic_cast().unwrap()
            },
            FileChooser(path) => {
                let btn = gtk::Button::new();
                btn.set_label(path);
                btn.dynamic_cast().unwrap()
            },
            TextArea(doc) => {
                let textarea = gtk::TextView::new();
                let buffer = textarea.get_buffer().unwrap();
                buffer.set_text(doc);
                textarea.dynamic_cast().unwrap()
            },
            Font(font) => {
                let fontbtn = gtk::FontButton::new_with_font(font);
                fontbtn.dynamic_cast().unwrap()
            },
            Color(color) => {
                let colorbtn = gtk::ColorButton::new_with_rgba(color);
                colorbtn.dynamic_cast().unwrap()
            },
        }
    }
}

pub struct Model {
    width: i32,
}

#[derive(Msg)]
pub enum PropertyMsg {
    OnRemove,
    Clear,
    AppendPage(&'static str),
    SetVBoxWidget(usize, Vec<WidgetType>),
    SetGridWidget(usize, Vec<(String, WidgetType)>),
}

pub struct PropertyViewerWidget {
    model: Model,
    vbox: gtk::Box,
    notebook: gtk::Notebook,
    remove_button: gtk::Button,
}

impl Update for PropertyViewerWidget {
    type Model = Model;
    type ModelParam = i32;
    type Msg = PropertyMsg;

    fn model(_: &Relm<Self>, width: i32) -> Model {
        Model {
            width: width,
        }
    }

    fn update(&mut self, event: PropertyMsg) {
        use self::PropertyMsg::*;

        match event {
            Clear => {
                for widget in self.notebook.get_children() {
                    self.notebook.remove(&widget);
                }
            },
            AppendPage(name) => {
                let grid = gtk::Grid::new();
                grid.set_column_spacing(10);
                grid.set_row_spacing(5);

                self.notebook.append_page(&grid, Some(&gtk::Label::new(name)));
                self.notebook.show_all();
            },
            SetVBoxWidget(index, widgets) => {
                let tab_widget = self.notebook.get_children()[index].clone().dynamic_cast::<gtk::Grid>().unwrap();
                for child in tab_widget.get_children() {
                    tab_widget.remove(&child);
                }

                for (i, widget_type) in widgets.into_iter().enumerate() {
                    tab_widget.attach(&widget_type.to_widget(), 0, i as i32, 1, 1);
                }

                tab_widget.show_all();
            },
            SetGridWidget(index, widgets) => {
                let tab_widget = self.notebook.get_children()[index].clone().dynamic_cast::<gtk::Grid>().unwrap();
                for child in tab_widget.get_children() {
                    tab_widget.remove(&child);
                }

                for (i, (key, widget_type)) in widgets.into_iter().enumerate() {
                    let label = gtk::Label::new(key.as_str());
                    label.set_halign(gtk::Align::End);

                    tab_widget.attach(&label, 0, i as i32, 1, 1);
                    tab_widget.attach(&widget_type.to_widget(), 1, i as i32, 1, 1);
                }

                tab_widget.show_all();
            },
            _ => (),
        }
    }
}

impl Widget for PropertyViewerWidget {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.vbox.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let notebook = gtk::Notebook::new();
        notebook.set_size_request(model.width, 100);

        let remove_button = gtk::Button::new();
        remove_button.set_label("Remove");
        connect!(relm, remove_button, connect_clicked(_), PropertyMsg::OnRemove);

        vbox.pack_start(&notebook, true, true, 0);
        vbox.pack_start(&remove_button, false, false, 5);

        PropertyViewerWidget {
            model: model,
            vbox: vbox,
            notebook: notebook,
            remove_button: remove_button,
        }
    }
}

