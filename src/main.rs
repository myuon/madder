use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;

extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate pango;

use gtk::prelude::*;
use gdk::prelude::*;

extern crate madder_core;
use madder_core::*;

pub mod widget;
use widget::*;

struct App {
    editor: Editor,
    timeline: Rc<RefCell<TimelineWidget>>,
    canvas: gtk::DrawingArea,
    property: PropertyViewerWidget,
    selected_component_index: Option<usize>,
    window: gtk::Window,
}

impl App {
    pub fn new(width: i32, height: i32) -> App {
        let prop_width = 250;

        App {
            editor: Editor::new(width, height),
            timeline: TimelineWidget::new(width + prop_width),
            canvas: gtk::DrawingArea::new(),
            property: PropertyViewerWidget::new(prop_width),
            selected_component_index: None,
            window: gtk::Window::new(gtk::WindowType::Toplevel),
        }
    }

    pub fn new_from_json(json: &EditorStructure) -> Rc<RefCell<App>> {
        let app = Rc::new(RefCell::new(App::new(json.width, json.height)));

        {
            let app = app.clone();
            json.components.iter().for_each(move |item| App::register(app.clone(), Component::new_from_structure(item)));
        }
        app
    }

    fn queue_draw(&self) {
        self.canvas.queue_draw();

        let timeline: &TimelineWidget = &self.timeline.as_ref().borrow();
        timeline.queue_draw();
    }

    fn queue_change_component_property(&self, index: usize) {
        self.property.set_properties(self.editor.request_component_info(index));
    }

    fn register(self_: Rc<RefCell<App>>, component: Component) {
        let name = &component.name.clone();
        let start_time = component.structure.start_time;
        let length = component.structure.length;
        let index = self_.as_ref().borrow_mut().editor.register(component);

        {
            let self_ = self_.clone();
            let self__ = self_.clone();
            let time_to_length = |p: gst::ClockTime| p.mseconds().unwrap() as i32;
            TimelineWidget::add_component_widget(
                self_.as_ref().borrow().timeline.clone(),
                &index.to_string(),
                &name,
                time_to_length(start_time), time_to_length(length),
                Box::new(move |evbox| {
                    self__.as_ref().borrow_mut().select_component(evbox.clone());
                    gtk::Inhibit(false)
                })
            );
        }
    }

    fn register_from_json(self_: Rc<RefCell<App>>, json: &ComponentStructure) {
        App::register(self_, Component::new_from_structure(json))
    }

    fn select_component(&mut self, selected_box: gtk::EventBox) {
        let name = gtk::WidgetExt::get_name(&selected_box).unwrap();
        let index = name.parse::<usize>().unwrap();
        self.queue_change_component_property(index);
        self.selected_component_index = Some(index);
    }

    fn create_menu(self_: Rc<RefCell<App>>) -> gtk::MenuBar {
        let menubar = gtk::MenuBar::new();

        let editor_item = gtk::MenuItem::new_with_label("タイムライン");
        menubar.append(&editor_item);

        let editor_menu = gtk::Menu::new();
        editor_item.set_submenu(&editor_menu);

        {
            let video_item = gtk::MenuItem::new_with_label("動画");
            editor_menu.append(&video_item);

            let self_ = self_.clone();
            video_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self_.as_ref().borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                App::register_from_json(self_.clone(), &ComponentStructure {
                    component_type: ComponentType::Video,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    coordinate: (0,0),
                });

                dialog.destroy();
            });
        }

        {
            let image_item = gtk::MenuItem::new_with_label("画像");
            editor_menu.append(&image_item);

            let self_ = self_.clone();
            image_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), Some(&self_.as_ref().borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                App::register_from_json(self_.clone(), &ComponentStructure {
                    component_type: ComponentType::Image,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    coordinate: (0,0),
                });

                dialog.destroy();
            });
        }

        {
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            editor_menu.append(&text_item);

            let self_ = self_.clone();
            text_item.connect_activate(move |_| {
                App::register_from_json(self_.clone(), &ComponentStructure {
                    component_type: ComponentType::Text,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: "dummy entity".to_string(),
                    coordinate: (50,50),
                });
            });
        }

        menubar
    }

    pub fn create_ui(self_: Rc<RefCell<App>>) {
        {
            let timeline = &self_.as_ref().borrow().timeline;

            {
                let self_ = self_.clone();
                timeline.as_ref().borrow().ruler_connect_button_press_event(move |event| {
                    let (x,_) = event.get_position();
                    self_.as_ref().borrow_mut().editor.seek_to(x as u64 * gst::MSECOND);
                    self_.as_ref().borrow().queue_draw();

                    Inhibit(false)
                });
            }

            {
                let self_ = self_.clone();
                timeline.as_ref().borrow().tracker_connect_draw(move |cr| {
                    cr.set_source_rgb(200f64, 0f64, 0f64);

                    cr.move_to(self_.as_ref().borrow().editor.position.mseconds().unwrap() as f64, 0f64);
                    cr.rel_line_to(0f64, 100f64);
                    cr.stroke();

                    Inhibit(false)
                });
            }
        }

        {
            let canvas = &self_.as_ref().borrow().canvas;
            let self_ = self_.clone();
            canvas.connect_draw(move |_,cr| {
                cr.set_source_pixbuf(&self_.as_ref().borrow().editor.get_current_pixbuf(), 0f64, 0f64);
                cr.paint();
                Inhibit(false)
            });
        }

        {
            let self_: &App = &(*self_.as_ref()).borrow();
            self_.canvas.set_size_request(self_.editor.width, self_.editor.height);
            self_.window.set_default_size(self_.editor.width, self_.editor.height + 200);
            self_.window.set_title("madder");
        }

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&App::create_menu(self_.clone()), true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&self_.as_ref().borrow().canvas, true, true, 0);
        hbox.pack_start(&self_.as_ref().borrow().property.view, true, true, 0);

        {
            let property = &self_.as_ref().borrow().property;
            property.create_ui();

            let self_ = self_.clone();
            property.connect_cell_edited(Box::new(move |_, tree_path, new_text: &str| {
                let store = self_.as_ref().borrow().property.store.clone();
                let iter = store.get_iter(&tree_path).unwrap();
                let index = self_.as_ref().borrow().selected_component_index.unwrap();

                self_.as_ref().borrow_mut().editor.request_set_component_property(index, store.get_value(&iter, 0).get::<String>().unwrap(), new_text.to_string());
                self_.as_ref().borrow_mut().property.store.set_value(&iter, 1, &glib::Value::from(new_text));
                self_.as_ref().borrow().queue_change_component_property(index);
            }));
        }

        vbox.pack_start(&hbox, true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let entry = gtk::Entry::new();
        let go_btn = gtk::Button::new();

        hbox.pack_start(&entry, true, true, 0);
        hbox.pack_start(&go_btn, true, true, 5);
        vbox.pack_start(&hbox, true, true, 5);

        let btn = gtk::Button::new();
        btn.set_label("render");

        {
            let self_ = self_.clone();
            btn.connect_clicked(move |_| {
                self_.borrow_mut().editor.write("output/output.avi", 100, 5);
            });
        }

        {
            let entry = entry.clone();
            let entry = Rc::new(entry);

            go_btn.set_label("Go");

            let self_ = self_.clone();
            go_btn.connect_clicked(move |_| {
                if let Ok(time) = entry.get_text().unwrap().parse::<u64>() {
                    self_.borrow_mut().editor.seek_to(time * gst::MSECOND);
                }
            });
        }

        vbox.pack_start(&btn, true, true, 5);

        {
            let self_: &App = &self_.as_ref().borrow();
            vbox.pack_start(self_.timeline.as_ref().borrow().to_widget(), true, true, 5);
        }

        {
            let self_: &App = &self_.as_ref().borrow();
            self_.window.add(&vbox);
            self_.window.show_all();
            self_.window.connect_delete_event(move |_,_| {
                gtk::main_quit();
                Inhibit(false)
            });
        }
    }
}

fn main() {
    gtk::init().expect("Gtk initialization error");
    gst::init().expect("Gstreamer initialization error");

    use std::env;
    let args = env::args().collect::<Vec<String>>();

    let editor =
        if args.len() >= 2 {
            EditorStructure::new_from_file(&args[1])
        } else {
            EditorStructure {
                width: 640,
                height: 480,
                components: Box::new([
                    ComponentStructure {
                        component_type: ComponentType::Text,
                        start_time: 0 * gst::MSECOND,
                        length: 100 * gst::MSECOND,
                        entity: "[ここにテキストを挿入]".to_string(),
                        coordinate: (50,50),
                    }
                ]),
            }
        };

    let app = App::new_from_json(&editor);
    App::create_ui(app);

    gtk::main();
}
