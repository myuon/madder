use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
use gtk::prelude::*;

extern crate gdk;
use gdk::prelude::*;

extern crate cairo;
extern crate pango;

use widget::AsWidget;
use widget::RulerWidget;
use widget::BoxViewerWidget;
use widget::BoxObject;

pub struct TimelineWidget {
    timeline: Rc<RefCell<BoxViewerWidget>>,
    ruler: Rc<RefCell<RulerWidget>>,
    ruler_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    container: gtk::Grid,
    scaler: gtk::Scale,
}

// workaround for sharing a variable within callbacks
impl TimelineWidget {
    pub fn new(width: i32) -> Rc<RefCell<TimelineWidget>> {
        let timeline = BoxViewerWidget::new(width, 100);

        let ruler_box = gtk::EventBox::new();

        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);

        let overlay = gtk::Overlay::new();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        overlay.add(&vbox);

        let ruler = RulerWidget::new(width, 30);
        ruler_box.add(ruler.borrow().as_widget());
        vbox.pack_start(&ruler_box, true, true, 10);

        vbox.pack_start(timeline.borrow().as_widget(), true, true, 0);

        let tracker = gtk::DrawingArea::new();
        overlay.add_overlay(&tracker);
        tracker.set_size_request(width, 100);

        overlay.set_overlay_pass_through(&tracker, true);
        tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let scaler = gtk::Scale::new_with_range(gtk::Orientation::Horizontal, 1.0, 10.0, 0.1);

        grid.attach(&scaler, 0, 0, 1, 1);
        grid.attach(&overlay, 1, 0, 1, 2);

        let w = Rc::new(RefCell::new(TimelineWidget {
            timeline: timeline,
            ruler: ruler,
            ruler_box: ruler_box,
            container: grid,
            tracker: tracker,
            scaler: scaler,
        }));
        TimelineWidget::create_ui(w.clone());

        w
    }

    fn create_ui(self_: Rc<RefCell<TimelineWidget>>) {
        let self__ = self_.clone();
        self_.borrow().scaler.connect_value_changed(move |scaler| {
            RulerWidget::change_scale(self__.borrow().ruler.clone(), scaler.get_value());
        });
    }

    pub fn create_menu(&self, menu: &gtk::Menu) {
        let menu = menu.clone();
        BoxViewerWidget::connect_click_no_box(self.timeline.clone(), Box::new(move || {
            menu.popup_easy(0, gtk::get_current_event_time());
            menu.show_all();
        }));
    }

    pub fn connect_request_objects(&self, cont: Box<Fn() -> Vec<BoxObject>>) {
        self.timeline.borrow_mut().connect_request_objects(cont);
    }

    pub fn connect_select_component(&self, cont: Box<Fn(usize)>) {
        BoxViewerWidget::connect_select_box(self.timeline.clone(), cont);
    }

    pub fn connect_drag_component(&self, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        BoxViewerWidget::connect_drag_box(self.timeline.clone(), cont_move, cont_resize);
    }

    pub fn ruler_connect_button_press_event<F: Fn(&gdk::EventButton) -> gtk::Inhibit + 'static>(&self, cont: F) {
        self.ruler_box.connect_button_press_event(move |_, event| {
            cont(event)
        });
    }

    pub fn tracker_connect_draw<F: Fn(&cairo::Context) -> gtk::Inhibit + 'static>(&self, cont: F) {
        self.tracker.connect_draw(move |_, cr| {
            cont(cr)
        });
    }

    pub fn queue_draw(&self) {
        self.container.queue_draw();
    }
}

impl AsWidget for TimelineWidget {
    type T = gtk::Grid;

    fn as_widget(&self) -> &Self::T {
        &self.container
    }
}
