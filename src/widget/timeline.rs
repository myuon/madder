use std::cmp;
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

pub struct TimelineWidget<Renderer: AsRef<BoxObject>> {
    box_viewer: BoxViewerWidget<Renderer>,
    ruler: RulerWidget,
    ruler_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    grid: gtk::Grid,
    overlay: gtk::Overlay,
    scaler: gtk::Scale,
    tracking_position: i32,
    width: i32,
    height: i32,
    length: i32,
    pub connect_get_component: Box<Fn(usize) -> component::Component>,
    pub connect_select_component: Box<Fn(usize)>,
    pub connect_select_component_menu: Box<Fn(usize, gst::ClockTime) -> gtk::Menu>,
    pub create_timeline_menu: Box<Fn() -> gtk::Menu>,
    pub connect_set_component_attr: Box<Fn(usize, &str, Attribute)>,
}

// workaround for sharing a variable within callbacks
impl<Renderer: 'static + AsRef<BoxObject>> TimelineWidget<Renderer> {
    pub fn new(width: i32, height: i32, length: i32) -> TimelineWidget<Renderer> {
        let box_viewer: BoxViewerWidget<Renderer> = BoxViewerWidget::new(height);

        let ruler_box = gtk::EventBox::new();

        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);

        let ruler = RulerWidget::new(length, 20);
        ruler_box.add(ruler.as_widget());

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
        vbox.pack_start(box_viewer.as_widget(), true, true, 0);
        overlay.add(&vbox);

        TimelineWidget {
            box_viewer: box_viewer,
            ruler: ruler,
            ruler_box: ruler_box,
            grid: grid,
            tracker: tracker,
            overlay: overlay,
            scaler: gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 0.1),
            tracking_position: 0,
            width: width,
            height: height,
            length: length,
            connect_get_component: Box::new(|_| unreachable!()),
            connect_select_component: Box::new(|_| unreachable!()),
            connect_select_component_menu: Box::new(|_,_| unreachable!()),
            create_timeline_menu: Box::new(|| unreachable!()),
            connect_set_component_attr: Box::new(|_,_,_| unreachable!()),
        }
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

    pub fn on_select_box(&mut self, index: usize, event: &gdk::EventButton) {
        if event.get_button() == 1 {
            (self.connect_select_component)(index);
        } else if event.get_button() == 3 {
            let length = (event.get_position().0 / self.scaler.get_value()) as u64 * gst::MSECOND;
            let menu = (self.connect_select_component_menu)(index, length);
            menu.popup_easy(0, gtk::get_current_event_time());
            menu.show_all();
        }
    }

    pub fn on_select_no_box(&self, event: &gdk::EventButton) {
        let menu = (self.create_timeline_menu)();

        if event.get_button() == 3 {
            menu.popup_easy(0, gtk::get_current_event_time());
            menu.show_all();
        }
    }

    fn on_get_scale(&self) -> f64 {
        self.scaler.get_value()
    }

    fn notify_pointer_motion(&mut self, x: f64) {
        self.ruler.queue_draw();
        self.ruler.send_pointer_position(x);
    }

    pub fn create_ui(&mut self) {
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

    pub fn connect_ruler_seek_time<F: Fn(gst::ClockTime) -> gtk::Inhibit + 'static>(&mut self, cont: F) {
        let self_ = self as *mut Self;
        self.ruler_box.connect_button_press_event(move |_, event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            self_.tracking_position = event.get_position().0 as i32;
            cont((event.get_position().0 * self_.scaler.get_value()) as u64 * gst::MSECOND)
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
