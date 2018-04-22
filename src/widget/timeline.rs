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
use relm::*;

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
    on_get_object: Rc<Box<Fn() -> Vec<Renderer>>>,
    on_render: Rc<Box<Fn(Renderer, f64, &cairo::Context)>>,
}

#[derive(Msg)]
pub enum TimelineMsg {
    RulerSeekTime(f64),
    RulerMotionNotify(f64),
    RulerQueueDraw,
    DrawTracker(gtk::DrawingArea),
    OpenMenu(gdk::EventButton),
    ChangeScale,
    SelectComponent(usize, gdk::EventButton),
    DragComponent(usize, i32, usize),
    ResizeComponent(usize, i32),
    ConnectGetComponent(Box<Fn(usize) -> component::Component>),
    OpenVideoItem,
    OpenImageItem,
    OpenTextItem,
    OnSetComponentAttr(usize, &'static str, Attribute),
    OnNewComponent(serde_json::Value),
    OnSelectComponent(usize),
}

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

pub struct TimelineWidget<Renderer: AsRef<BoxObject> + 'static> {
    model: Model<Renderer>,
    grid: gtk::Grid,
    overlay: gtk::Overlay,
    scaler: gtk::Scale,
    ruler: relm::Component<RulerWidget>,
}

impl<Renderer> Update for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Model = Model<Renderer>;
    type ModelParam = (i32, i32, i32, Rc<Box<Fn() -> Vec<Renderer>>>, Rc<Box<Fn(Renderer, f64, &cairo::Context)>>);
    type Msg = TimelineMsg;

    fn model(relm: &Relm<Self>, (width, height, length, on_get_object, on_render): (i32, i32, i32, Rc<Box<Fn() -> Vec<Renderer>>>, Rc<Box<Fn(Renderer, f64, &cairo::Context)>>)) -> Model<Renderer> {
        Model {
            tracking_position: 0,
            width: width,
            height: height,
            length: length,
            get_component: Box::new(|_| unreachable!()),
            menu: gtk::Menu::new(),
            relm: relm.clone(),
            tracker: gtk::DrawingArea::new(),
            on_get_object: on_get_object,
            on_render: on_render,
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
            SelectComponent(index, event) => {
                if event.get_button() == 1 {
                    self.model.relm.stream().emit(TimelineMsg::OnSelectComponent(index));
                } else if event.get_button() == 3 {
                    let _length = (event.get_position().0 / self.scaler.get_value()) as u64 * gst::MSECOND;
                    let menu = gtk::Menu::new();
                    menu.append(&gtk::MenuItem::new_with_label("piyo"));
                    menu.popup_easy(0, gtk::get_current_event_time());
                    menu.show_all();
                }
            },
            _ => (),
        }
    }
}

impl<Renderer> Widget for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Root = gtk::Grid;

    fn root(&self) -> Self::Root {
        self.grid.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);

        let scaler = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 0.1);
        connect!(relm, scaler, connect_value_changed(_), TimelineMsg::ChangeScale);

        let scrolled = gtk::ScrolledWindow::new(None, None);
        scrolled.set_size_request(model.width, model.height);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        grid.attach(&scaler,0,0,1,1);
        grid.attach(&gtk::Label::new("Layers here"),0,1,1,1);
        grid.attach(&scrolled,1,0,1,2);

        let overlay = gtk::Overlay::new();
        overlay.set_size_request(model.length, -1);
        overlay.add_overlay(&model.tracker);
        overlay.set_overlay_pass_through(&model.tracker, true);
        scrolled.add(&overlay);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        overlay.add(&vbox);

        let ruler_box = gtk::EventBox::new();
        ruler_box.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        connect!(relm, ruler_box, connect_button_press_event(_, event), return (Some(TimelineMsg::RulerSeekTime(event.get_position().0)), Inhibit(false)));
        connect!(relm, ruler_box, connect_motion_notify_event(_, event), return (Some(TimelineMsg::RulerMotionNotify(event.get_position().0)), Inhibit(false)));
        let ruler = ruler_box.add_widget::<RulerWidget>((model.length, 20, Rc::new(scaler.clone())));
        vbox.pack_start(&ruler_box, true, true, 10);

        let box_viewer = vbox.add_widget::<BoxViewerWidget<Renderer>>((
            model.height,
            Rc::new(scaler.clone()),
            model.on_get_object.clone(),
            model.on_render.clone(),
        ));

        {
            use self::BoxViewerMsg::*;
            connect!(box_viewer@OnSelect(ref index, ref event), relm, TimelineMsg::SelectComponent(*index, event.clone()));
            connect!(box_viewer@OnSelectNoBox(ref event), relm, TimelineMsg::OpenMenu(event.clone()));
            connect!(box_viewer@Motion(ref event), relm, TimelineMsg::RulerMotionNotify(event.get_position().0));
            connect!(box_viewer@OnDrag(index, distance, layer_index), relm, TimelineMsg::DragComponent(index, distance, layer_index));
            connect!(box_viewer@OnResize(index, distance), relm, TimelineMsg::ResizeComponent(index, distance));
        }

        model.tracker.set_size_request(model.length, -1);
        model.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });
        model.tracker.show();

        connect!(relm, model.tracker, connect_draw(tracker,_), return (Some(TimelineMsg::DrawTracker(tracker.clone())), Inhibit(false)));

        {
            let video_item = gtk::MenuItem::new_with_label("動画");
            let image_item = gtk::MenuItem::new_with_label("画像");
            let text_item = gtk::MenuItem::new_with_label("テキスト");
            model.menu.append(&video_item);
            model.menu.append(&image_item);
            model.menu.append(&text_item);

            connect!(relm, video_item, connect_activate(_), return (TimelineMsg::OpenVideoItem, ()));
            connect!(relm, image_item, connect_activate(_), return (TimelineMsg::OpenImageItem, ()));
            connect!(relm, text_item, connect_activate(_), return (TimelineMsg::OpenTextItem, ()));
        }

        TimelineWidget {
            model: model,
            grid: grid,
            scaler: scaler,
            overlay: overlay,
            ruler: ruler,
        }
    }
}

