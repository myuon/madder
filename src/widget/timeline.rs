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
    get_component: Box<Fn(usize) -> component::Component>,
    menu: gtk::Menu,
    relm: Relm<TimelineWidget<Renderer>>,
    tracker: gtk::DrawingArea,
}

#[derive(Msg)]
pub enum TimelineMsg {
    RulerSeekTime(f64),
    RulerMotionNotify(f64),
    RulerQueueDraw,
    DrawTracker(gtk::DrawingArea),
    OpenMenu(gdk::EventButton),
    ChangeScale,
    OnSelect(usize),
    DragComponent(usize, i32, usize),
    ResizeComponent(usize, i32),
    ConnectDraw(Rc<Box<Fn(&cairo::Context, f64)>>),
    ConnectGetComponent(Box<Fn(usize) -> component::Component>),
    OpenVideoItem,
    OpenImageItem,
    OpenTextItem,
    OnSetComponentAttr(usize, &'static str, Attribute),
    OnNewComponent(serde_json::Value),
}

use self::BoxViewerMsg::*;

fn json_entity(component_type: &str, entity: &str) -> serde_json::Value {
    json!({
        "component_type": component_type,
        "start_time": 0,
        "length": 100,
        "layer_index": 0,
        "prop": {
            "entity": entity,
        },
    })
}

#[widget]
impl<Renderer> Widget for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    fn model(relm: &Relm<Self>, (width, height, length): (i32, i32, i32)) -> Model<Renderer> {
        Model {
            tracking_position: 0,
            width: width,
            height: height,
            length: length,
            get_component: Box::new(|_| unreachable!()),
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
            ChangeScale => {
                self.overlay.set_size_request((self.model.length as f64 / self.scaler.get_value()) as i32, -1);
                self.grid.queue_draw();
            },
            ConnectDraw(callback) => {
                self.box_viewer.stream().emit(BoxViewerMsg::ConnectDraw(callback));
            },
            DragComponent(index, distance, layer_index) => {
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

                let component = (self.model.get_component)(index);

                self.model.relm.stream().emit(TimelineMsg::OnSetComponentAttr(
                    index,
                    "start_time",
                    Attribute::Time(add_time(component.start_time, distance as f64)),
                ));
                self.model.relm.stream().emit(TimelineMsg::OnSetComponentAttr(
                    index,
                    "layer_index",
                    Attribute::Usize(cmp::max(layer_index, 0)),
                ));

                self.grid.queue_draw();
            },
            ResizeComponent(index, distance) => {
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

                let component = (self.model.get_component)(index);

                self.model.relm.stream().emit(TimelineMsg::OnSetComponentAttr(
                    index,
                    "length",
                    Attribute::Time(add_time(component.length, distance as f64)),
                ));

                self.grid.queue_draw();
            },
            ConnectGetComponent(callback) => {
                self.model.get_component = callback;
            },
            OpenMenu(event) => {
                if event.get_button() == 3 {
                    self.model.menu.popup_easy(0, gtk::get_current_event_time());
                    self.model.menu.show_all();
                }
            },
            OpenVideoItem => {
                let dialog = gtk::FileChooserDialog::new(Some("動画を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.mkv");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let entity = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                self.model.relm.stream().emit(TimelineMsg::OnNewComponent(json_entity("Video", &entity)));

                self.grid.queue_draw();
                dialog.destroy();
            },
            OpenImageItem => {
                let dialog = gtk::FileChooserDialog::new(Some("画像を選択"), None as Option<&gtk::Window>, gtk::FileChooserAction::Open);
                dialog.add_button("追加", 0);

                {
                    let filter = gtk::FileFilter::new();
                    filter.add_pattern("*.png");
                    dialog.add_filter(&filter);
                }
                dialog.run();

                let entity = dialog.get_filename().unwrap().as_path().to_str().unwrap().to_string();
                self.model.relm.stream().emit(TimelineMsg::OnNewComponent(json_entity("Image", &entity)));

                self.grid.queue_draw();
                dialog.destroy();
            },
            OpenTextItem => {
                self.model.relm.stream().emit(TimelineMsg::OnNewComponent(json_entity("Text", "dummy entity")));
                self.grid.queue_draw();
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

        {
            let video_item = gtk::MenuItem::new_with_label("動画");
            let image_item = gtk::MenuItem::new_with_label("画像");
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            self.model.menu.append(&video_item);
            self.model.menu.append(&image_item);
            self.model.menu.append(&text_item);

            connect!(self.model.relm, video_item, connect_activate(_), return (TimelineMsg::OpenVideoItem, ()));
            connect!(self.model.relm, image_item, connect_activate(_), return (TimelineMsg::OpenImageItem, ()));
            connect!(self.model.relm, text_item, connect_activate(_), return (TimelineMsg::OpenTextItem, ()));
        }
    }

    view! {
        #[name="grid"]
        gtk::Grid {
            column_spacing: 4,

            #[name="scaler"]
            gtk::Scale {
                orientation: gtk::Orientation::Horizontal,
                adjustment: &gtk::Adjustment::new(1.0, 0.1, 100.0, 0.1, 0.1, 0.1),
                value_changed(_) => TimelineMsg::ChangeScale,

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
                            OnSelect(ref index, _) => TimelineMsg::OnSelect(*index),
                            OnSelectNoBox(ref event) => TimelineMsg::OpenMenu(event.clone()),
                            Motion(ref event) => TimelineMsg::RulerMotionNotify(event.get_position().0),
                            OnDrag(index, distance, layer_index) => TimelineMsg::DragComponent(index, distance, layer_index),
                            OnResize(index, distance) => TimelineMsg::ResizeComponent(index, distance),
                        },
                    },
                },
            },
        },
    }
}

