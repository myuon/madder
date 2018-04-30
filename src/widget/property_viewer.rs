extern crate gtk;
extern crate gdk;
use gtk::prelude::*;

extern crate relm;
use relm::*;

extern crate serde_json;

use madder_core::*;

#[derive(Debug, Clone)]
pub enum WidgetType {
    NumberEntry(serde_json::Number),
    TextEntry(String),
    Choose(Vec<String>, Option<usize>),
    Label(String),
    VBox(Vec<WidgetType>),
    Grid(Vec<(String, WidgetType)>),
    Expander(String, Box<WidgetType>),
    FileChooser(String),
    TextArea(String),
    Font(String),
    Color(gdk::RGBA),
}

impl WidgetType {
    fn to_widget(&self, stream: EventStream<PropertyMsg>, path: String) -> gtk::Widget {
        use self::WidgetType::*;

        match self {
            Label(label) => {
                let label = gtk::Label::new(label.as_str());
                label.dynamic_cast().unwrap()
            },
            NumberEntry(label) => {
                let entry = gtk::Entry::new();
                entry.set_text(&label.to_string());
                entry.connect_changed(move |entry| {
                    if let Some(num) = entry.get_text().and_then(|t| t.parse::<serde_json::Number>().ok()) {
                        stream.emit(PropertyMsg::OnChangeAttr(
                            NumberEntry(num),
                            Pointer::from_str(&path),
                        ));
                    }
                });
                entry.dynamic_cast().unwrap()
            },
            TextEntry(label) => {
                let entry = gtk::Entry::new();
                entry.set_text(label);
                entry.connect_changed(move |entry| {
                    stream.emit(PropertyMsg::OnChangeAttr(
                        TextEntry(entry.get_text().unwrap()),
                        Pointer::from_str(&path),
                    ));
                });
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

                combo.connect_changed(move |combo| {
                    stream.emit(PropertyMsg::OnChangeAttr(
                        Choose(vec![], Some(combo.get_active() as usize)),
                        Pointer::from_str(&path),
                    ));
                });

                combo.dynamic_cast().unwrap()
            },
            Grid(vec) => {
                let grid = gtk::Grid::new();

                for (i, (key, widget_type)) in vec.iter().enumerate() {
                    grid.attach(&gtk::Label::new(key.as_str()), 0, i as i32, 1, 1);
                    grid.attach(&widget_type.to_widget(stream.clone(), format!("{}/{}", path, key)), 1, i as i32, 1, 1);
                }

                grid.dynamic_cast().unwrap()
            },
            VBox(vec) => {
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

                for (i, widget_type) in vec.iter().enumerate() {
                    vbox.pack_start(&widget_type.to_widget(stream.clone(), format!("{}/{}", path, i)), false, false, 0);
                }

                vbox.dynamic_cast().unwrap()
            },
            Expander(label, box widget_type) => {
                let expander = gtk::Expander::new(label.as_str());
                expander.set_margin_top(5);
                expander.set_margin_bottom(5);

                expander.add(&widget_type.to_widget(stream, path));
                expander.dynamic_cast().unwrap()
            },
            FileChooser(filename) => {
                let btn = gtk::Button::new();
                btn.set_label(filename);
                btn.connect_clicked(move |_| {
                    let dialog = gtk::FileChooserDialog::new(Some("Entity"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
                    dialog.add_button("追加", 0);

                    {
                        let filter = gtk::FileFilter::new();
                        filter.add_pattern("*.*");
                        dialog.add_filter(&filter);
                    }
                    dialog.run();
                    stream.emit(PropertyMsg::OnChangeAttr(
                        FileChooser(dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string()),
                        Pointer::from_str(&path),
                    ));
                    dialog.destroy();
                });
                btn.dynamic_cast().unwrap()
            },
            TextArea(doc) => {
                let textarea = gtk::TextView::new();

                let buffer = textarea.get_buffer().unwrap();
                buffer.set_text(doc);
                buffer.connect_changed(move |buffer| {
                    stream.emit(PropertyMsg::OnChangeAttr(
                        buffer.get_text(&buffer.get_start_iter(), &buffer.get_end_iter(), true).map(TextArea).unwrap(),
                        Pointer::from_str(&path),
                    ));
                });
                textarea.dynamic_cast().unwrap()
            },
            Font(font) => {
                let fontbtn = gtk::FontButton::new_with_font(font);
                fontbtn.connect_font_set(move |fontbtn| {
                    stream.emit(PropertyMsg::OnChangeAttr(
                        fontbtn.get_font().map(Font).unwrap(),
                        Pointer::from_str(&path),
                    ));
                });
                fontbtn.dynamic_cast().unwrap()
            },
            Color(color) => {
                let colorbtn = gtk::ColorButton::new_with_rgba(color);
                colorbtn.connect_color_set(move |colorbtn| {
                    stream.emit(PropertyMsg::OnChangeAttr(
                        Color(colorbtn.get_rgba()),
                        Pointer::from_str(&path),
                    ));
                });
                colorbtn.dynamic_cast().unwrap()
            },
        }
    }
}

pub struct Model {
    width: i32,
    relm: Relm<PropertyViewerWidget>,
}

#[derive(Msg)]
pub enum PropertyMsg {
    OnRemove,
    OnChangeAttr(WidgetType, Pointer),
    ClearPage(usize),
    AppendPage(&'static str),
    AppendVBoxWidget(usize, Vec<(WidgetType, String)>),
    AppendGridWidget(usize, Vec<(String, WidgetType, String)>),
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

    fn model(relm: &Relm<Self>, width: i32) -> Model {
        Model {
            width: width,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: PropertyMsg) {
        use self::PropertyMsg::*;

        match event {
            ClearPage(index) => {
                let tab_widget = self.notebook.get_children()[index].clone().dynamic_cast::<gtk::Grid>().unwrap();
                for child in tab_widget.get_children() {
                    tab_widget.remove(&child);
                }
            },
            AppendPage(name) => {
                let grid = gtk::Grid::new();
                grid.set_column_spacing(10);
                grid.set_row_spacing(5);

                self.notebook.append_page(&grid, Some(&gtk::Label::new(name)));
                self.notebook.show_all();
            },
            AppendVBoxWidget(index, widgets) => {
                let tab_widget = self.notebook.get_children()[index].clone().dynamic_cast::<gtk::Grid>().unwrap();
                for child in tab_widget.get_children() {
                    tab_widget.remove(&child);
                }

                for (i, (widget_type, path)) in widgets.into_iter().enumerate() {
                    tab_widget.attach(&widget_type.to_widget(self.model.relm.stream().clone(), path), 0, i as i32, 1, 1);
                }

                tab_widget.show_all();
            },
            AppendGridWidget(index, widgets) => {
                let tab_widget = self.notebook.get_children()[index].clone().dynamic_cast::<gtk::Grid>().unwrap();
                let widget_num = tab_widget.get_children().len() as i32;

                for (i, (key, widget_type, path)) in widgets.into_iter().enumerate() {
                    let label = gtk::Label::new(key.as_str());
                    label.set_halign(gtk::Align::End);

                    tab_widget.attach(&label, 0, widget_num + i as i32, 1, 1);
                    tab_widget.attach(&widget_type.to_widget(self.model.relm.stream().clone(), path), 1, widget_num + i as i32, 1, 1);
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

