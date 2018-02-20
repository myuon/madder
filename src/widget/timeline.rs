use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
use gtk::prelude::*;

extern crate gdk;
use gdk::prelude::*;

extern crate cairo;
extern crate pango;

use widget::WidgetWrapper;
use widget::RulerWidget;
use widget::BoxViewerWidget;
use widget::BoxObject;

pub struct TimelineWidget {
    timeline: Rc<RefCell<BoxViewerWidget>>,
    ruler_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    container: gtk::Overlay,
}

// workaround for sharing a variable within callbacks
impl TimelineWidget {
    pub fn new(width: i32) -> Rc<RefCell<TimelineWidget>> {
        let timeline = BoxViewerWidget::new(width, 50);

        let ruler_box = gtk::EventBox::new();

        let overlay = gtk::Overlay::new();
        {
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            overlay.add(&vbox);

            let ruler = RulerWidget::new(width);
            ruler_box.add(ruler.to_widget());
            vbox.pack_start(&ruler_box, true, true, 10);

            vbox.pack_start(timeline.as_ref().borrow().to_widget(), true, true, 0);
        }

        let tracker = gtk::DrawingArea::new();
        overlay.add_overlay(&tracker);
        tracker.set_size_request(width, 50);

        overlay.set_overlay_pass_through(&tracker, true);
        tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        Rc::new(RefCell::new(TimelineWidget {
            timeline: timeline,
            ruler_box: ruler_box,
            container: overlay,
            tracker: tracker,
        }))
    }

    pub fn create_ui(&self) {
        BoxViewerWidget::create_ui(self.timeline.clone());
    }

    pub fn connect_request_objects(&self, cont: Box<Fn() -> Vec<BoxObject>>) {
        self.timeline.as_ref().borrow_mut().connect_request_objects(cont);
    }

    pub fn connect_select_component(&self, cont: Box<Fn(BoxObject)>) {
        BoxViewerWidget::connect_select_box(self.timeline.clone(), cont);
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

impl WidgetWrapper for TimelineWidget {
    type T = gtk::Overlay;

    fn to_widget(&self) -> &Self::T {
        &self.container
    }
}
