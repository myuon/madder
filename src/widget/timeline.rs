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
use relm::*;
use relm_attributes::widget;

extern crate madder_core;
use madder_core::*;
use widget::*;

pub struct Model<Renderer: AsRef<BoxObject> + 'static> {
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
    relm: Relm<TimelineWidget<Renderer>>,
    tracker: gtk::DrawingArea,
}

#[derive(Msg)]
pub enum TimelineMsg {
    RulerSeekTime(f64),
    RulerMotionNotify(f64),
    RulerQueueDraw,
    DrawTracker(gtk::DrawingArea),
    OnDrawObjects(gdk::Window),
    OnSelect(usize),
}

use self::BoxViewerMsg::*;

#[widget]
impl<Renderer> Widget for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    fn model(relm: &Relm<Self>, (width, height, length): (i32, i32, i32)) -> Model<Renderer> {
        Model {
            tracking_position: 0,
            width: width,
            height: height,
            length: length,
            connect_get_component: Box::new(|_| unreachable!()),
            connect_select_component: Box::new(|_| unreachable!()),
            connect_select_component_menu: Box::new(|_,_| unreachable!()),
            connect_set_component_attr: Box::new(|_,_,_| unreachable!()),
            connect_new_component: Box::new(|_| unreachable!()),
            menu: gtk::Menu::new(),
            relm: relm.clone(),
            tracker: gtk::DrawingArea::new(),
        }
    }

    fn update(&mut self, event: TimelineMsg) {
        use self::TimelineMsg::*;

        match event {
            RulerSeekTime(time) => {
                self.model.tracking_position = time as i32;
                self.model.tracker.queue_draw();
            },
            RulerMotionNotify(pos) => {
                self.ruler.stream().emit(RulerMsg::MovePointer(pos));
                self.model.relm.stream().emit(TimelineMsg::RulerQueueDraw);
            },
            RulerQueueDraw => {
                self.ruler.widget().queue_draw();
            },
            DrawTracker(tracker) => {
                let cr = cairo::Context::create_from_window(&self.model.tracker.get_window().unwrap());
                cr.set_source_rgb(200f64, 0f64, 0f64);

                cr.move_to(self.model.tracking_position as f64, 0f64);
                cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
                cr.stroke();
            },
            _ => (),
        }
    }

    fn init_view(&mut self) {
        self.model.tracker.set_size_request(self.model.length, -1);
        self.model.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });
        self.model.tracker.show();

        connect!(self.model.relm, self.model.tracker, connect_draw(tracker,_), return (Some(TimelineMsg::DrawTracker(tracker.clone())), Inhibit(false)));

        self.overlay.set_size_request(self.model.length, -1);
        self.overlay.add_overlay(&self.model.tracker);
        self.overlay.set_overlay_pass_through(&self.model.tracker, true);

        self.scrolled.set_size_request(self.model.width, self.model.height);
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

            #[name="scrolled"]
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
                            #[name="ruler"]
                            RulerWidget(self.model.length, 20, Rc::new(scaler.clone())) {
                            },

                            realize(ruler_box) => {
                                ruler_box.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
                            },
                            button_press_event(_, event) => (Some(TimelineMsg::RulerSeekTime(event.get_position().0)), Inhibit(false)),
                            motion_notify_event(_, event) => (Some(TimelineMsg::RulerMotionNotify(event.get_position().0)), Inhibit(false))
                        },

                        #[name="box_viewer"]
                        BoxViewerWidget<Renderer>(self.model.height, Rc::new(scaler.clone())) {
                            OnDraw(ref window) => TimelineMsg::OnDrawObjects(window.clone()),
                            OnSelect(ref index, ref event) => {
                                if event.get_button() == 3 {
                                    let menu = gtk::Menu::new();
                                    menu.append(&gtk::MenuItem::new_with_label("puyo"));
                                    menu.popup_easy(0, gtk::get_current_event_time());
                                    menu.show_all();
                                }

                                TimelineMsg::OnSelect(*index)
                            },
                            OnSelectNoBox(ref event) => {
                                if event.get_button() == 3 {
                                    let menu = gtk::Menu::new();
                                    menu.append(&gtk::MenuItem::new_with_label("puyo back"));
                                    menu.popup_easy(0, gtk::get_current_event_time());
                                    menu.show_all();
                                }
                            },
                            Motion(ref event) => TimelineMsg::RulerMotionNotify(event.get_position().0),
                        },
                    },
                },
            },
        },
    }
}

/*
// workaround for sharing a variable within callbacks
impl<Renderer: 'static + AsRef<BoxObject>> TimelineWidget<Renderer> {
    pub fn create_ui(&mut self) {
        self.create_menu();

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

