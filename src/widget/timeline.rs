use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gtk;
extern crate gdk;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate cairo;
extern crate pango;

extern crate madder_core;
use madder_core::*;
use widget::*;

pub trait TimelineWidgetI {
    fn get_component(&self, usize) -> component::Component;
    fn set_component_attr(&mut self, usize, &str, Attribute);
}

pub struct TimelineWidget<M: TimelineWidgetI + BoxViewerWidgetI> {
    box_viewer: Rc<RefCell<BoxViewerWidget<M>>>,
    ruler: Rc<RefCell<RulerWidget>>,
    ruler_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    grid: gtk::Grid,
    overlay: gtk::Overlay,
    scaler: gtk::Scale,
    tracking_position: i32,
    model: Option<Rc<RefCell<M>>>
}

// workaround for sharing a variable within callbacks
impl<M: 'static + TimelineWidgetI + BoxViewerWidgetI> TimelineWidget<M> {
    pub fn new(width: i32, height: i32, length: i32) -> Rc<RefCell<TimelineWidget<M>>> {
        let box_viewer = BoxViewerWidget::new(height);

        let ruler_box = gtk::EventBox::new();

        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);

        let ruler = RulerWidget::new(length, 20);
        ruler_box.add(ruler.borrow().as_widget());

        let tracker = gtk::DrawingArea::new();
        tracker.set_size_request(length, -1);

        let overlay = {
            let overlay = gtk::Overlay::new();
            overlay.add_overlay(&tracker);
            overlay.set_overlay_pass_through(&tracker, true);
            overlay
        };
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.pack_start(&ruler_box, true, true, 10);
        vbox.pack_start(box_viewer.borrow().as_widget(), true, true, 0);
        overlay.add(&vbox);

        let w = Rc::new(RefCell::new(TimelineWidget {
            box_viewer: box_viewer,
            ruler: ruler,
            ruler_box: ruler_box,
            grid: grid,
            tracker: tracker,
            overlay: overlay,
            scaler: gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 0.1),
            tracking_position: 0,
            model: None,
        }));
        TimelineWidget::create_ui(w.clone(), width, height, length);

        w
    }

    pub fn set_model(&mut self, model: Rc<RefCell<M>>) {
        self.box_viewer.borrow_mut().set_model(model.clone());
        self.model = Some(model);
    }

    fn notify_pointer_motion(&self, x: f64) {
        let ruler = self.ruler.clone();
        ruler.borrow().queue_draw();
        RulerWidget::send_pointer_position(ruler, x);
    }

    fn create_ui(self_: Rc<RefCell<TimelineWidget<M>>>, width: i32, height: i32, length: i32) {
        let timeline = self_.borrow();
        timeline.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let self__ = self_.clone();
        timeline.ruler_box.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        timeline.ruler_box.connect_motion_notify_event(move |_,event| {
            self__.borrow().notify_pointer_motion(event.get_position().0);
            Inhibit(false)
        });

        let self__ = self_.clone();
        timeline.tracker.connect_draw(move |tracker,cr| {
            cr.set_source_rgb(200f64, 0f64, 0f64);

            cr.move_to(self__.borrow().tracking_position as f64, 0f64);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });

        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.set_size_request(width, height);
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);
        scroll.add(&timeline.overlay);

        timeline.grid.attach(&timeline.scaler,0,0,1,1);
        timeline.grid.attach(&gtk::Label::new("layers here"),0,1,1,1);
        timeline.grid.attach(&scroll, 1, 0, 1, 2);

        timeline.overlay.set_size_request(length, -1);

        let self__ = self_.clone();
        let ruler_ = self_.borrow().ruler.clone();
        RulerWidget::connect_get_scale(ruler_, Box::new(move || {
            self__.borrow().scaler.get_value()
        }));

        let self__ = self_.clone();
        BoxViewerWidget::connect_motion_notify_event(timeline.box_viewer.clone(), Box::new(move |event| {
            self__.borrow().notify_pointer_motion(event.get_position().0);
        }));

        let self__ = self_.clone();
        let box_viewer_ = self__.borrow().box_viewer.clone();
        BoxViewerWidget::connect_get_scale(box_viewer_, Box::new(move || {
            self__.borrow().scaler.get_value()
        }));

        let self__ = self_.clone();
        self_.borrow().scaler.connect_value_changed(move |scaler| {
            self__.borrow().overlay.set_size_request((length as f64 / scaler.get_value()) as i32, -1);
            self__.borrow().as_widget().queue_draw();
        });
    }

    pub fn create_menu(&self, menu: &gtk::Menu) {
        let menu = menu.clone();
        BoxViewerWidget::connect_click_no_box(self.box_viewer.clone(), Box::new(move |event| {
            if event.get_button() == 3 {
                menu.popup_easy(0, gtk::get_current_event_time());
                menu.show_all();
            }
        }));
    }

    pub fn setup_object_renderer(&self) {
        BoxViewerWidget::setup(self.box_viewer.clone());
    }

    pub fn connect_select_component(self_: Rc<RefCell<TimelineWidget<M>>>, cont: Box<Fn(usize)>, cont_menu: Box<Fn(usize, gst::ClockTime) -> gtk::Menu>) {
        let self__ = self_.clone();
        BoxViewerWidget::connect_select_box(self_.borrow().box_viewer.clone(), Box::new(move |index, event| {
            if event.get_button() == 1 {
                cont(index)
            } else if event.get_button() == 3 {
                let length = (event.get_position().0 / self__.borrow().scaler.get_value()) as u64 * gst::MSECOND;
                let menu = cont_menu(index, length);
                menu.popup_easy(0, gtk::get_current_event_time());
                menu.show_all();
            }
        }));
    }

    pub fn connect_drag_component(self_: Rc<RefCell<TimelineWidget<M>>>) {
        let self__ = self_.clone();
        let self___ = self_.clone();
        let self____ = self_.clone();
        let inst = self_.borrow().model.as_ref().unwrap().clone();
        let inst_ = inst.clone();
        BoxViewerWidget::connect_drag_box(
            self__.borrow().box_viewer.clone(),
            Box::new(move |index,distance,layer_index| {
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

                let component = inst.borrow().get_component(index);
                inst.borrow_mut().set_component_attr(
                    index,
                    "start_time",
                    Attribute::Time(add_time(component.start_time, distance as f64)),
                );
                inst.borrow_mut().set_component_attr(
                    index,
                    "layer_index",
                    Attribute::Usize(cmp::max(layer_index, 0)),
                );

                self___.borrow().queue_draw();
            }),
            Box::new(move |index,distance| {
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

                let component = inst_.borrow().get_component(index);
                inst_.borrow_mut().set_component_attr(
                    index,
                    "length",
                    Attribute::Time(add_time(component.length, distance as f64)),
                );

                self____.borrow().queue_draw();
            }),
        );
    }

    pub fn connect_ruler_seek_time<F: Fn(gst::ClockTime) -> gtk::Inhibit + 'static>(self_: Rc<RefCell<TimelineWidget<M>>>, cont: F) {
        let scaler = self_.borrow().scaler.clone();
        let self__ = self_.clone();
        self_.borrow().ruler_box.connect_button_press_event(move |_, event| {
            self__.borrow_mut().tracking_position = event.get_position().0 as i32;
            cont((event.get_position().0 * scaler.get_value()) as u64 * gst::MSECOND)
        });
    }

    pub fn queue_draw(&self) {
        self.overlay.queue_draw();
        self.box_viewer.borrow().as_widget().queue_draw();
    }
}

impl<M: TimelineWidgetI + BoxViewerWidgetI> AsWidget for TimelineWidget<M> {
    type T = gtk::Grid;

    fn as_widget(&self) -> &Self::T {
        &self.grid

    }
}
