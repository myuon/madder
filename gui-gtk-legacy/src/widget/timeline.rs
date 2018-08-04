use std::rc::Rc;
use std::cell::RefCell;
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

pub struct Model<Renderer: AsRef<BoxObject> + Clone + 'static> {
    tracking_position: Rc<RefCell<f64>>,
    width: i32,
    height: i32,
    length: i32,
    get_component: Box<Fn(usize) -> component::Component>,
    relm: Relm<TimelineWidget<Renderer>>,
    on_get_object: Rc<Box<Fn() -> Vec<Renderer>>>,
    on_render: Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>,
    menu: gtk::Menu,
}

#[derive(Msg)]
pub enum TimelineMsg {
    RulerSeekTime(f64),
    RulerMotionNotify(f64),
    RulerQueueDraw,
    OpenMenu(gdk::EventButton),
    ChangeScale,
    DragComponent(usize, i32, usize),
    ResizeComponent(usize, i32),
    ConnectGetComponent(Box<Fn(usize) -> component::Component>),
    SelectNoBox(gdk::EventButton),
    OnSetComponentAttr(usize, &'static str, Attribute),
    OnSelectBox(usize, gdk::EventButton),
    OnNewComponent(serde_json::Value),
    QueueDraw,
    SplitComponent(usize, gst::ClockTime),
}

pub struct TimelineWidget<Renderer: AsRef<BoxObject> + Clone + 'static> {
    model: Model<Renderer>,
    grid: gtk::Grid,
    overlay: gtk::Overlay,
    scaler: gtk::Scale,
    tracker: gtk::DrawingArea,
    ruler: relm::Component<RulerWidget>,
    box_viewer: relm::Component<BoxViewerWidget<Renderer>>,
}

impl<Renderer> Update for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + Clone + 'static {
    type Model = Model<Renderer>;
    type ModelParam = (i32, i32, i32, Rc<Box<Fn() -> Vec<Renderer>>>, Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>, gtk::Menu);
    type Msg = TimelineMsg;

    fn model(relm: &Relm<Self>, (width, height, length, on_get_object, on_render, menu): Self::ModelParam) -> Model<Renderer> {
        Model {
            tracking_position: Rc::new(RefCell::new(0.0)),
            width: width,
            height: height,
            length: length,
            get_component: Box::new(|_| unreachable!()),
            relm: relm.clone(),
            on_get_object: on_get_object,
            on_render: on_render,
            menu: menu,
        }
    }

    fn update(&mut self, event: TimelineMsg) {
        use self::TimelineMsg::*;

        match event {
            RulerSeekTime(time) => {
                *self.model.tracking_position.borrow_mut() = time;
                self.tracker.queue_draw();
            },
            RulerMotionNotify(pos) => {
                self.ruler.stream().emit(RulerMsg::MovePointer(pos));
                self.model.relm.stream().emit(TimelineMsg::RulerQueueDraw);
            },
            RulerQueueDraw => {
                self.ruler.widget().queue_draw();
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
                    Attribute::Time(add_time(component.as_component().start_time, distance as f64)),
                ));
                self.model.relm.stream().emit(TimelineMsg::OnSetComponentAttr(
                    index,
                    "layer_index",
                    Attribute::Usize(cmp::max(layer_index, 0)),
                ));

                self.box_viewer.widget().queue_draw();
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
                    Attribute::Time(add_time(component.as_component().length, distance as f64)),
                ));

                self.box_viewer.widget().queue_draw();
            },
            ConnectGetComponent(callback) => {
                self.model.get_component = callback;
            },
            QueueDraw => {
                self.tracker.queue_draw();
            },
            _ => (),
        }
    }
}

impl<Renderer> Widget for TimelineWidget<Renderer> where Renderer: AsRef<BoxObject> + Clone + 'static {
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

        let tracker = gtk::DrawingArea::new();
        tracker.set_size_request(model.length, -1);
        tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let overlay = gtk::Overlay::new();
        overlay.set_size_request(model.length, -1);
        overlay.add_overlay(&tracker);
        overlay.set_overlay_pass_through(&tracker, true);
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
            let menu = model.menu.clone();

            use self::BoxViewerMsg::*;
            connect!(box_viewer@OnSelect(ref object, ref event), relm, TimelineMsg::OnSelectBox(object.as_ref().index, event.clone()));
            connect!(box_viewer@OnSelectNoBox(ref _event), relm, {
                menu.popup_easy(0, gtk::get_current_event_time());
                menu.show_all();
            });
            connect!(box_viewer@Motion(ref event), relm, TimelineMsg::RulerMotionNotify(event.get_position().0));
            connect!(box_viewer@OnDrag(index, distance, layer_index), relm, TimelineMsg::DragComponent(index, distance, layer_index));
            connect!(box_viewer@OnResize(index, distance), relm, TimelineMsg::ResizeComponent(index, distance));
        }

        let tracking_position = model.tracking_position.clone();
        connect!(relm, tracker, connect_draw(tracker,cr), return {
            cr.set_source_rgb(200.0, 0.0, 0.0);

            cr.move_to(*tracking_position.borrow() as f64, 0.0);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });

        TimelineWidget {
            model: model,
            grid: grid,
            scaler: scaler,
            overlay: overlay,
            ruler: ruler,
            tracker: tracker,
            box_viewer: box_viewer,
        }
    }
}

