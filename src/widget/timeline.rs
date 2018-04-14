use std::rc::Rc;
use std::cmp;
extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate cairo;
extern crate pango;
extern crate serde_json;

extern crate relm;
extern crate relm_attributes;
extern crate relm_derive;
use relm_attributes::widget;
use relm::*;

extern crate madder_core;
use madder_core::*;
use widget::*;

pub struct Model {
    tracking_position: i32,
    width: i32,
    height: i32,
    length: i32,
    pub connect_get_component: Box<Fn(usize) -> component::Component>,
    pub connect_select_component: Box<Fn(usize)>,
    pub connect_select_component_menu: Box<Fn(usize, gst::ClockTime) -> gtk::Menu>,
    pub connect_set_component_attr: Box<Fn(usize, &str, Attribute)>,
    pub connect_new_component: Box<Fn(serde_json::Value)>,
    pub menu: gtk::Menu,
}

#[derive(Msg)]
pub enum TimelineMsg {
    RulerSeekTime(f64),
    DrawObjects(cairo::Context),
}

use self::BoxViewerMsg::*;

#[widget]
impl<Renderer: 'static> Widget for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> {
    fn model(_: &Relm<Self>, parameter: (i32, i32, i32)) -> Model {
        Model {
            tracking_position: 0,
            width: parameter.0,
            height: parameter.1,
            length: parameter.2,
            connect_get_component: Box::new(|_| unreachable!()),
            connect_select_component: Box::new(|_| unreachable!()),
            connect_select_component_menu: Box::new(|_,_| unreachable!()),
            connect_set_component_attr: Box::new(|_,_,_| unreachable!()),
            connect_new_component: Box::new(|_| unreachable!()),
            menu: gtk::Menu::new(),
        }
    }

    fn update(&mut self, event: TimelineMsg) {
        use self::TimelineMsg::*;

        match event {
            RulerSeekTime(time) => {
                self.model.tracking_position = time as i32;
            },
            DrawObjects(_) => {
            },
        }
    }

    fn init_view(&mut self) {
        let tracker = gtk::DrawingArea::new();
        tracker.set_size_request(self.model.length, -1);

        self.overlay.add_overlay(&tracker);
        self.overlay.set_overlay_pass_through(&tracker, true);
    }

    view! {
        #[name="grid"]
        gtk::Grid {
            column_spacing: 4,

            #[name="scaler"]
            gtk::Scale {
                cell: {
                    top_attach: 0,
                    left_attach: 0,
                },
            },
            gtk::Label {
                label: "Layers here",
                cell: {
                    top_attach: 1,
                    left_attach: 0,
                },
            },
            gtk::ScrolledWindow {
                hexpand: true,
                vexpand: true,
                cell: {
                    top_attach: 0,
                    left_attach: 1,
                    height: 2,
                },

                #[name="overlay"]
                gtk::Overlay {
                    gtk::Box {
                        orientation: gtk::Orientation::Vertical,

                        #[name="ruler_box"]
                        gtk::EventBox {
                            RulerWidget(self.model.length, 20) {
                            },

                            button_press_event(_, event) => (Some(TimelineMsg::RulerSeekTime(event.get_position().0)), Inhibit(false)),
                        },

                        #[name="box_viewer"]
                        BoxViewerWidget<Renderer>(self.model.height, Rc::new(scaler.clone())) {
                            Draw(ref cr) => TimelineMsg::DrawObjects(cr.clone()),
                        },
                    },
                },
            },
        },
    }
}

impl<Renderer: AsRef<BoxObject>> TimelineWidget<Renderer> {
    fn create_menu(&mut self) {
        /*
        let video_item = gtk::MenuItem::new_with_label("動画");
        let image_item = gtk::MenuItem::new_with_label("画像");
        let text_item = gtk::MenuItem::new_with_label("テキスト");
        self.model.menu.append(&video_item);
        self.model.menu.append(&image_item);
        self.model.menu.append(&text_item);

        let self_ = self as *mut Self;
        video_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
            dialog.add_button("追加", 0);

            {
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.mkv");
                dialog.add_filter(&filter);
            }
            dialog.run();

            (self.model.connect_new_component)(json!({
                "component_type": "Video",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                }
            }));

            self.queue_draw();
            dialog.destroy();
        });

        let self_ = self as *mut Self;
        image_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
            dialog.add_button("追加", 0);

            {
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.png");
                dialog.add_filter(&filter);
            }
            dialog.run();

            (self.model.connect_new_component)(json!({
                "component_type": "Image",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                }
            }));

            self.queue_draw();
            dialog.destroy();
        });

        let self_ = self as *mut Self;
        text_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            (self.model.connect_new_component)(json!({
                "component_type": "Text",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": "dummy entity",
                    "coordinate": [50, 50],
                }
            }));

            self.queue_draw();
        });
        */
    }

    pub fn queue_draw(&self) {
        /*
        self.overlay.queue_draw();
        self.box_viewer.queue_draw();
        */
    }
}

/*
// workaround for sharing a variable within callbacks
impl<Renderer: 'static + AsRef<BoxObject>> TimelineWidget<Renderer> {
    pub fn set_component_attr(&mut self, index: usize, name: &str, value: Attribute) {
        (self.connect_set_component_attr)(index, name, value);
    }

    pub fn get_component(&self, index: usize) -> component::Component {
        (self.connect_get_component)(index)
    }

    pub fn connect_get_objects(&mut self, cont: Box<Fn() -> Vec<Renderer>>) {
        self.box_viewer.connect_get_objects = cont;
    }

    pub fn connect_render_object(&mut self, cont: Box<Fn(Renderer, f64, &cairo::Context)>) {
        self.box_viewer.connect_render_object = cont;
    }

    fn notify_pointer_motion(&mut self, x: f64) {
        self.ruler.queue_draw();
        self.ruler.send_pointer_position(x);
    }

    fn create_menu(&mut self) {
        let video_item = gtk::MenuItem::new_with_label("動画");
        let image_item = gtk::MenuItem::new_with_label("画像");
        let text_item = gtk::MenuItem::new_with_label("テキスト");
        self.menu.append(&video_item);
        self.menu.append(&image_item);
        self.menu.append(&text_item);

        let self_ = self as *mut Self;
        video_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
            dialog.add_button("追加", 0);

            {
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.mkv");
                dialog.add_filter(&filter);
            }
            dialog.run();

            (self_.connect_new_component)(json!({
                "component_type": "Video",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                }
            }));

            self_.queue_draw();
            dialog.destroy();
        });

        let self_ = self as *mut Self;
        image_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
            dialog.add_button("追加", 0);

            {
                let filter = gtk::FileFilter::new();
                filter.add_pattern("*.png");
                dialog.add_filter(&filter);
            }
            dialog.run();

            (self_.connect_new_component)(json!({
                "component_type": "Image",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string(),
                }
            }));

            self_.queue_draw();
            dialog.destroy();
        });

        let self_ = self as *mut Self;
        text_item.connect_activate(move |_| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            (self_.connect_new_component)(json!({
                "component_type": "Text",
                "start_time": 0,
                "length": 100,
                "layer_index": 0,
                "prop": {
                    "entity": "dummy entity",
                    "coordinate": [50, 50],
                }
            }));

            self_.queue_draw();
        });
    }

    pub fn create_ui(&mut self) {
        self.create_menu();

        let self_ = self as *mut Self;
        self.box_viewer.connect_select_box = Box::new(move |_, index, event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            if event.get_button() == 1 {
                (self_.connect_select_component)(index);
            } else if event.get_button() == 3 {
                (self_.connect_select_component)(index);
                let length = (event.get_position().0 / self_.scaler.get_value()) as u64 * gst::MSECOND;
                let menu = (self_.connect_select_component_menu)(index, length);
                menu.popup_easy(0, gtk::get_current_event_time());
                menu.show_all();
            }
        });

        let self_ = self as *mut Self;
        self.box_viewer.connect_select_no_box = Box::new(move |event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            if event.get_button() == 3 {
                self_.menu.popup_easy(0, gtk::get_current_event_time());
                self_.menu.show_all();
            }
        });

        let self_ = self as *mut Self;
        self.box_viewer.connect_get_scale = Box::new(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };
            self_.scaler.get_value()
        });

        self.ruler.create_ui();

        let self_ = self as *mut Self;
        self.ruler.connect_get_scale = Box::new(move || {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.scaler.get_value()
        });

        self.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let self_ = self as *mut Self;
        self.ruler_box.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self.ruler_box.connect_motion_notify_event(move |_,event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.notify_pointer_motion(event.get_position().0);
            Inhibit(false)
        });

        let self_ = self as *mut Self;
        self.tracker.connect_draw(move |tracker,cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            cr.set_source_rgb(200f64, 0f64, 0f64);

            cr.move_to(self_.tracking_position as f64, 0f64);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });

        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.set_size_request(self.width, self.height);
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);
        scroll.add(&self.overlay);

        self.grid.attach(&self.scaler,0,0,1,1);
        self.grid.attach(&gtk::Label::new("layers here"),0,1,1,1);
        self.grid.attach(&scroll, 1, 0, 1, 2);

        self.overlay.set_size_request(self.length, -1);

        let length = self.length;
        let self_ = self as *mut Self;
        self.scaler.connect_value_changed(move |scaler| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.overlay.set_size_request((length as f64 / scaler.get_value()) as i32, -1);
            self_.as_widget().queue_draw();
        });

        self.box_viewer.setup();
    }

    pub fn connect_drag_component(&mut self) {
        let self_ = self as *mut Self;
        self.box_viewer.connect_drag_box(
            Box::new(move |index,distance,layer_index| {
                let self_ = unsafe { self_.as_mut().unwrap() };

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

                let component = (self_.connect_get_component)(index);
                self_.set_component_attr(
                    index,
                    "start_time",
                    Attribute::Time(add_time(component.start_time, distance as f64)),
                );
                self_.set_component_attr(
                    index,
                    "layer_index",
                    Attribute::Usize(cmp::max(layer_index, 0)),
                );

                self_.queue_draw();
            }),
            Box::new(move |index,distance| {
                let self_ = unsafe { self_.as_mut().unwrap() };

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

                let component = self_.get_component(index);
                self_.set_component_attr(
                    index,
                    "length",
                    Attribute::Time(add_time(component.length, distance as f64)),
                );

                self_.queue_draw();
            }),
        );
    }

    pub fn queue_draw(&self) {
        self.overlay.queue_draw();
        self.box_viewer.as_widget().queue_draw();
    }
}

impl<M: AsRef<BoxObject>> AsWidget for TimelineWidget<M> {
    type T = gtk::Grid;

    fn as_widget(&self) -> &Self::T {
        &self.grid

    }
}
 */
