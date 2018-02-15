use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;

extern crate gtk;
use gtk::prelude::*;

extern crate gdk;
use gdk::prelude::*;

extern crate cairo;
extern crate pango;

use widget::WidgetWrapper;
use widget::RulerWidget;

pub struct TimelineWidget {
    fixed: gtk::Fixed,
    fixed_box: gtk::EventBox,
    tracker: gtk::DrawingArea,
    container: gtk::Overlay,
    offset: i32,
}

// workaround for sharing a variable within callbacks
impl TimelineWidget {
    pub fn new(width: i32) -> Rc<RefCell<TimelineWidget>> {
        let fixed_box = gtk::EventBox::new();
        let fixed = gtk::Fixed::new();
        fixed.set_size_request(width, 50);

        fixed_box.add(&fixed);

        let overlay = gtk::Overlay::new();
        {
            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
            overlay.add(&vbox);

            let ruler = RulerWidget::new(width);
            vbox.pack_start(ruler.to_widget(), true, true, 10);

            vbox.pack_start(&fixed_box, true, true, 0);
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
            fixed: fixed,
            fixed_box: fixed_box,
            container: overlay,
            tracker: tracker,
            offset: 0
        }))
    }

    pub fn connect_button_press_event<F: Fn(&gdk::EventButton) -> gtk::Inhibit + 'static>(&self, cont: F) {
        self.fixed_box.connect_button_press_event(move |_, event| {
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

    pub fn add_component_widget(self_: Rc<RefCell<TimelineWidget>>, label_text: &str, offset_x: i32, width: i32) {
        let evbox = gtk::EventBox::new();
        evbox.show();
        evbox.connect_realize(move |evbox| {
            let window = evbox.get_window().unwrap();
            window.set_pass_through(false);
        });

        {
            let builder: &RefCell<TimelineWidget> = self_.borrow();
            let builder: &TimelineWidget = &builder.borrow();
            builder.fixed.put(&evbox, offset_x, 0);
        }

        let label = gtk::Label::new(label_text);
        evbox.add(&label);
        label.override_background_color(gtk::StateFlags::NORMAL, &gdk::RGBA::red());
        label.set_ellipsize(pango::EllipsizeMode::End);
        label.set_size_request(width,30);
        label.show();

        {
            let self_: Rc<RefCell<TimelineWidget>> = self_.clone();
            evbox.connect_button_press_event(move |evbox,button| {
                let (rx,_) = evbox.get_parent().unwrap().get_window().unwrap().get_position();
                let (x,_) = button.get_position();

                let builder: &RefCell<TimelineWidget> = self_.borrow();
                builder.borrow_mut().offset = rx + x as i32;
                Inhibit(false)
            });
        }

        {
            let self_: Rc<RefCell<TimelineWidget>> = self_.clone();
            evbox.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
            evbox.connect_motion_notify_event(move |evbox,motion| {
                let (x,_) = motion.get_position();
                let evbox_window = motion.get_window().unwrap();
                let (rx,_) = evbox_window.get_position();

                let grab_edge = 5;
                if (evbox_window.get_width() - x as i32) <= grab_edge {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "e-resize"));
                } else if (x as i32) <= grab_edge {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "w-resize"));
                } else {
                    evbox_window.set_cursor(&gdk::Cursor::new_from_name(&evbox_window.get_display(), "default"));
                }

                if motion.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
                    let x_max = evbox.get_parent().unwrap().get_allocation().width - evbox.get_allocation().width;

                    let builder: &RefCell<TimelineWidget> = self_.borrow();
                    builder.borrow().fixed.move_(evbox, cmp::max(cmp::min(rx + x as i32 - builder.borrow().offset, x_max), 0), 0);
                }

                Inhibit(false)
            });
        }
    }
}

impl WidgetWrapper for TimelineWidget {
    type T = gtk::Overlay;

    fn to_widget(&self) -> &Self::T {
        &self.container
    }
}
