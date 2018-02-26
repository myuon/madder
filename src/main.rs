#![feature(box_patterns)]
#![feature(slice_patterns)]
use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;

extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate cairo;
extern crate pango;

use gtk::prelude::*;
use gdk::prelude::*;

extern crate madder_core;
use madder_core::*;

pub mod widget;
use widget::*;

pub mod gtk_impl;

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
            property: PropertyViewerWidget::new(prop_width, &[format!("{:?}", PropertyGroupTab::ComponentProperty), format!("{:?}", PropertyGroupTab::EffectProperty)]),
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

        let timeline: &TimelineWidget = &self.timeline.borrow();
        timeline.queue_draw();
    }

    fn register(self_: Rc<RefCell<App>>, component: Box<ComponentLike>) {
        self_.borrow_mut().editor.register(component);
    }

    fn register_from_json(self_: Rc<RefCell<App>>, json: &Component) {
        App::register(self_, Component::new_from_structure(json))
    }

    fn remove_selected(self_: Rc<RefCell<App>>) {
        let index = self_.borrow().selected_component_index.unwrap();
        self_.borrow_mut().editor.elements.remove(index);
        self_.borrow_mut().selected_component_index = None;
        self_.borrow().property.clear();
        self_.borrow().queue_draw();
    }

    fn select_component(self_: Rc<RefCell<App>>, index: usize) {
        let self__ = self_.clone();

        self_.borrow().property.set_properties(
            self_.borrow().editor.request_component_property(index),
            Box::new(move |prop_name, prop| {
                let prop_name = Rc::new(prop_name);
                let self__ = self__.clone();

                (format!("{:?}", prop.get_group_tab()),
                 gtk_impl::edit_type_to_widget(&prop, vec![], Rc::new(move |new_prop, tracker| {
                    // request the property again, since in this callback the value of property might have been changed
                    let prop = self__.borrow().editor.request_component_property(index)[prop_name.as_str()].clone();
                    if let Some(new_prop) = new_prop {
                        self__.borrow_mut().editor.set_component_property(index, prop_name.as_ref().clone(), gtk_impl::recover_property(prop, tracker, new_prop));
                    }

                    self__.borrow().queue_draw();
                })))
            }),
        );

        self_.borrow_mut().selected_component_index = Some(index);
        self_.borrow().queue_draw();
    }

    fn create_menu(self_: Rc<RefCell<App>>) -> gtk::MenuBar {
        let menubar = gtk::MenuBar::new();
        let file_item = {
            let file_item = gtk::MenuItem::new_with_label("ファイル");
            let file_menu = gtk::Menu::new();
            file_item.set_submenu(&file_menu);

            let output = gtk::MenuItem::new_with_label("動画の書き出し");
            file_menu.append(&output);

            let self__ = self_.clone();
            output.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Save);
                dialog.add_button("出力", 0);
                dialog.run();
                let path = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                dialog.destroy();

                let self__ = self__.clone();
                let window = gtk::Window::new(gtk::WindowType::Popup);
                let progress_bar = gtk::ProgressBar::new();
                let label = gtk::Label::new("進捗…");
                let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
                window.add(&vbox);
                window.set_transient_for(&self__.borrow().window);
                window.set_position(gtk::WindowPosition::CenterOnParent);

                vbox.pack_start(&label, true, true, 0);
                vbox.pack_start(&progress_bar, true, true, 0);
                progress_bar.set_pulse_step(0.0);

                window.show_all();

                let progress_bar = progress_bar.clone();
                self__.borrow_mut().editor.write_init(&path, 100, 5);

                idle_add(move || {
                    let (cont, frac) = self__.borrow_mut().editor.write_next();
                    progress_bar.set_fraction(frac);

                    if cont == false {
                        window.destroy();
                    }
                    Continue(cont)
                });
            });

            file_item
        };
        let editor_item = {
            let editor_item = gtk::MenuItem::new_with_label("タイムライン");
            let editor_menu = gtk::Menu::new();
            editor_item.set_submenu(&editor_menu);

            let video_item = gtk::MenuItem::new_with_label("動画");
            let image_item = gtk::MenuItem::new_with_label("画像");
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            editor_menu.append(&video_item);
            editor_menu.append(&image_item);
            editor_menu.append(&text_item);

            let self__ = self_.clone();
            video_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                App::register_from_json(self__.clone(), &Component {
                    component_type: ComponentType::Video,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    layer_index: 0,
                    coordinate: (0,0),
                    effect: vec![],
                });

                self__.borrow().queue_draw();
                dialog.destroy();
            });

            let self__ = self_.clone();
            image_item.connect_activate(move |_| {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), Some(&self__.borrow().window), gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                App::register_from_json(self__.clone(), &Component {
                    component_type: ComponentType::Image,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                    layer_index: 0,
                    coordinate: (0,0),
                    effect: vec![],
                });

                self__.borrow().queue_draw();
                dialog.destroy();
            });

            let self__ = self_.clone();
            text_item.connect_activate(move |_| {
                App::register_from_json(self__.clone(), &Component {
                    component_type: ComponentType::Text,
                    start_time: 0 * gst::MSECOND,
                    length: 100 * gst::MSECOND,
                    entity: "dummy entity".to_string(),
                    layer_index: 0,
                    coordinate: (50,50),
                    effect: vec![],
                });
                self__.borrow().queue_draw();
            });

            editor_item
        };

        menubar.append(&file_item);
        menubar.append(&editor_item);

        menubar
    }

    pub fn create_ui(self_: Rc<RefCell<App>>) {
        let app = self_.borrow();

        app.timeline.borrow_mut().create_ui();

        let self__ = self_.clone();
        app.timeline.borrow().connect_select_component(Box::new(move |index| {
            App::select_component(self__.clone(), index);
        }));

        let self__ = self_.clone();
        let self___ = self_.clone();
        app.timeline.borrow().connect_drag_component(
            Box::new(move |index,distance,layer_index| {
                let props = self__.borrow().editor.request_component_property(index);
                let add_time = |a: gst::ClockTime, b: f64| {
                    if b < 0.0 {
                        if a < b.abs() as u64 * gst::MSECOND {
                            0 * gst::MSECOND
                        } else {
                            a - b.abs() as u64 * gst::MSECOND
                        }
                    } else {
                        a + b as u64 * gst::MSECOND
                    }
                };

                self__.borrow_mut().editor.set_component_property(
                    index,
                    "start_time".to_string(),
                    Property::Time(add_time(props["start_time"].as_time().unwrap(), distance as f64)),
                );
                self__.borrow_mut().editor.set_component_property(
                    index,
                    "layer_index".to_string(),
                    Property::Usize(std::cmp::max(layer_index, 0)),
                );
                self__.borrow().queue_draw();
            }),
            Box::new(move |index,distance| {
                let props = self___.borrow().editor.request_component_property(index);
                let add_time = |a: gst::ClockTime, b: f64| {
                    if b < 0.0 {
                        if a < b.abs() as u64 * gst::MSECOND {
                            5 * gst::MSECOND
                        } else {
                            std::cmp::max(a - b.abs() as u64 * gst::MSECOND, 5.0 as u64 * gst::MSECOND)
                        }
                    } else {
                        a + b as u64 * gst::MSECOND
                    }
                };

                self___.borrow_mut().editor.set_component_property(
                    index,
                    "length".to_string(),
                    Property::Time(add_time(props["length"].as_time().unwrap(), distance as f64)),
                );
                self___.borrow().queue_draw();
            }),
        );

        let self__ = self_.clone();
        app.timeline.borrow().connect_request_objects(Box::new(move || {
            self__.borrow().editor.elements.iter().enumerate().map(|(i,component)| {
                BoxObject::new(
                    component.get_component().start_time.mseconds().unwrap() as i32,
                    component.get_component().length.mseconds().unwrap() as i32,
                    i
                ).label(component.get_component().entity)
                    .selected(Some(i) == self__.borrow().selected_component_index)
                    .layer_index(component.get_component().layer_index)
            }).collect()
        }));

        let self__ = self_.clone();
        app.timeline.borrow().ruler_connect_button_press_event(move |event| {
            let (x,_) = event.get_position();
            self__.borrow_mut().editor.seek_to(x as u64 * gst::MSECOND);
            self__.borrow().queue_draw();

            Inhibit(false)
        });

        let self__ = self_.clone();
        app.timeline.borrow().tracker_connect_draw(move |cr| {
            cr.set_source_rgb(200f64, 0f64, 0f64);

            cr.move_to(self__.borrow().editor.position.mseconds().unwrap() as f64, 0f64);
            cr.rel_line_to(0f64, 100f64);
            cr.stroke();

            Inhibit(false)
        });

        let self__ = self_.clone();
        app.canvas.connect_draw(move |_,cr| {
            cr.set_source_pixbuf(&self__.borrow().editor.get_current_pixbuf(), 0f64, 0f64);
            cr.paint();
            Inhibit(false)
        });

        app.canvas.set_size_request(app.editor.width, app.editor.height);
        app.window.set_default_size(app.editor.width, app.editor.height + 200);
        app.window.set_title("madder");

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&App::create_menu(self_.clone()), true, true, 0);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&app.canvas, true, true, 0);
        hbox.pack_start(app.property.to_widget(), true, true, 0);
        app.property.create_ui();

        let self__ = self_.clone();
        app.property.connect_remove(Box::new(move || {
            App::remove_selected(self__.clone());
        }));

        vbox.pack_start(&hbox, true, true, 0);
        vbox.pack_start(app.timeline.borrow().to_widget(), true, true, 5);

        app.window.add(&vbox);
        app.window.show_all();
        app.window.connect_delete_event(move |_,_| {
            gtk::main_quit();
            Inhibit(false)
        });
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
                    Component {
                        component_type: ComponentType::Text,
                        start_time: 0 * gst::MSECOND,
                        length: 100 * gst::MSECOND,
                        entity: "[ここにテキストを挿入]".to_string(),
                        layer_index: 0,
                        coordinate: (50,50),
                        effect: vec![],
                    }
                ]),
            }
        };

    let app = App::new_from_json(&editor);
    App::create_ui(app);

    gtk::main();
}
