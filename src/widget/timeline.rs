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
    box_viewer: Rc<RefCell<BoxViewerWidget>>,
    ruler: Rc<RefCell<RulerWidget>>,
    ruler_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    grid: gtk::Grid,
    overlay: gtk::Overlay,
    scaler: gtk::Scale,
}

// workaround for sharing a variable within callbacks
impl TimelineWidget {
    pub fn new(width: i32, height: i32, length: i32) -> Rc<RefCell<TimelineWidget>> {
        let box_viewer = BoxViewerWidget::new(length, height);

        let ruler_box = gtk::EventBox::new();

        let grid = gtk::Grid::new();
        grid.set_column_spacing(4);

        let ruler = RulerWidget::new(length, 30);
        ruler_box.add(ruler.borrow().as_widget());

        let tracker = gtk::DrawingArea::new();
        tracker.set_size_request(length, 100);

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
        }));
        TimelineWidget::create_ui(w.clone(), width, length);

        w
    }

    fn create_ui(self_: Rc<RefCell<TimelineWidget>>, width: i32, length: i32) {
        let timeline = self_.borrow();
        timeline.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.set_size_request(width, 100);
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);
        scroll.add(&timeline.overlay);

        timeline.grid.attach(&timeline.scaler,0,0,1,1);
        timeline.grid.attach(&gtk::Label::new("layers here"),0,1,1,1);
        timeline.grid.attach(&scroll, 1, 0, 1, 2);

        let self__ = self_.clone();
        self_.borrow().scaler.connect_value_changed(move |scaler| {
            RulerWidget::change_scale(self__.borrow().ruler.clone(), scaler.get_value());

            let mut alloc = self__.borrow().overlay.get_allocation();
            alloc.width = (length as f64 / scaler.get_value()) as i32;
            self__.borrow().overlay.size_allocate(&mut alloc);

            self__.borrow().as_widget().queue_draw();
        });
    }

    pub fn create_menu(&self, menu: &gtk::Menu) {
        let menu = menu.clone();
        BoxViewerWidget::connect_click_no_box(self.box_viewer.clone(), Box::new(move || {
            menu.popup_easy(0, gtk::get_current_event_time());
            menu.show_all();
        }));
    }

    pub fn connect_request_objects(&self, cont: Box<Fn() -> Vec<BoxObject>>) {
        self.box_viewer.borrow_mut().connect_request_objects(cont);
    }

    pub fn connect_select_component(&self, cont: Box<Fn(usize)>) {
        BoxViewerWidget::connect_select_box(self.box_viewer.clone(), cont);
    }

    pub fn connect_drag_component(&self, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        BoxViewerWidget::connect_drag_box(self.box_viewer.clone(), cont_move, cont_resize);
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
        self.as_widget().queue_draw();
    }
}

impl AsWidget for TimelineWidget {
    type T = gtk::Grid;

    fn as_widget(&self) -> &Self::T {
        &self.grid
    }
}
