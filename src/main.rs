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
use madder_core::{Editor, serializer, Component};

pub mod widget;
use widget::*;

struct App {
    editor: Editor,
    timeline: Rc<RefCell<TimelineWidget>>,
    canvas: gtk::DrawingArea,
    property: gtk::TreeView,
    window: gtk::Window,
}

impl App {
    pub fn new(width: i32, height: i32) -> App {
        App {
            editor: Editor::new(width, height),
            timeline: TimelineWidget::new(width),
            canvas: gtk::DrawingArea::new(),
            property: gtk::TreeView::new(),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
        }
    }

    pub fn new_from_json(json: &serializer::EditorStructure) -> App {
        let mut app = App::new(json.size.0, json.size.1);
        json.components.iter().for_each(|item| app.register(Component::new_from_structure(item)));
        app
    }

    fn queue_draw(&self) {
        self.canvas.queue_draw();

        let timeline: &TimelineWidget = &self.timeline.as_ref().borrow();
        timeline.queue_draw();
    }

    fn register(&mut self, component: Component) {
        let time_to_length = |p: gst::ClockTime| p.mseconds().unwrap() as i32;
        TimelineWidget::add_component_widget(self.timeline.clone(), &component.name, time_to_length(component.start_time), time_to_length(component.end_time - component.start_time));

        self.editor.register(component);
    }

    fn register_from_json(&mut self, json: &serializer::ComponentStructure) {
        self.register(Component::new_from_structure(json))
    }

    pub fn create_ui(self_: Rc<RefCell<App>>) {
        {
            let timeline = &self_.as_ref().borrow().timeline;

            {
                let self_ = self_.clone();
                timeline.as_ref().borrow().connect_button_press_event(move |event| {
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

        let menubar = gtk::MenuBar::new();
        vbox.pack_start(&menubar, true, true, 0);

        {
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

                    self_.borrow_mut().register_from_json(&serializer::ComponentStructure {
                        component_type: serializer::ComponentType::Video,
                        start_time: 0,
                        end_time: 100,
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

                    self_.borrow_mut().register_from_json(&serializer::ComponentStructure {
                        component_type: serializer::ComponentType::Image,
                        start_time: 0,
                        end_time: 100,
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
                    self_.borrow_mut().register_from_json(&serializer::ComponentStructure {
                        component_type: serializer::ComponentType::Text,
                        start_time: 0,
                        end_time: 100,
                        entity: "dummy entity".to_string(),
                        coordinate: (50,50),
                    });
                });
            }
        }

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&self_.as_ref().borrow().canvas, true, true, 0);
        hbox.pack_start(&self_.as_ref().borrow().property, true, true, 0);

        {
            let property = &self_.as_ref().borrow().property;
            let editor = &self_.as_ref().borrow().editor;
            let self_ = self_.clone();
            property.connect_draw(move |_,_| {
                let w = &self_.as_ref().borrow().window.get_focus();
                println!("{:?}", w);
                Inhibit(false)
            });

            let liststore = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::String]);
            liststore.insert_with_values(None, &[0,1], &[&"piyo", &"hoge"]);

            property.set_size_request(250, editor.height);
            property.set_model(&liststore);

            {
                let column = gtk::TreeViewColumn::new();
                column.set_title("Key");

                let cell = gtk::CellRendererText::new();
                column.pack_start(&cell, true);
                column.add_attribute(&cell, "text", 0);

                property.append_column(&column);
            }

            {
                let column = gtk::TreeViewColumn::new();
                column.set_title("Value");

                let cell = gtk::CellRendererText::new();
                column.pack_start(&cell, true);
                column.add_attribute(&cell, "text", 0);

                property.append_column(&column);
            }
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
            serializer::EditorStructure::new_from_file(&args[1])
        } else {
            serializer::EditorStructure {
                size: (640,480),
                components: Box::new([
                    serializer::ComponentStructure {
                        component_type: serializer::ComponentType::Text,
                        start_time: 0,
                        end_time: 100,
                        entity: "[ここにテキストを挿入]".to_string(),
                        coordinate: (50,50),
                    }
                ]),
            }
        };

    let app = App::new_from_json(&editor);
    App::create_ui(Rc::new(RefCell::new(app)));

    gtk::main();
}
