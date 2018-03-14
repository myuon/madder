use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gtk;
extern crate cairo;
use gtk::prelude::*;

use widget::AsWidget;

pub struct RulerWidget {
    canvas: gtk::DrawingArea,
    cb_get_scale: Box<Fn() -> f64>,
}

impl RulerWidget {
    pub fn new(width: i32, height: i32) -> Rc<RefCell<RulerWidget>> {
        let ruler = gtk::DrawingArea::new();
        ruler.set_size_request(-1, height);

        let w = Rc::new(RefCell::new(RulerWidget {
            canvas: ruler,
            cb_get_scale: Box::new(|| { 1.0 }),
        }));
        RulerWidget::create_ui(w.clone(), width, height);

        w
    }

    fn create_ui(self_: Rc<RefCell<RulerWidget>>, width: i32, height: i32) {
        let self__ = self_.clone();
        self_.borrow().canvas.connect_draw(move |_, cr| {
            cr.set_line_width(1.0);
            cr.set_source_rgb(0f64, 0f64, 0f64);

            cr.move_to(0f64, height as f64);
            cr.line_to(width as f64, height as f64);

            cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(10 as f64);

            let interval_large_height = height as f64;
            let interval_height = height as f64 * 0.5;
            let interval_small_height = height as f64 * 0.25;
            let interval = 10;

            let scaler = (self__.borrow().cb_get_scale)();

            for x in (0..(((width / interval) as f64) / scaler) as i32).map(|x| x * interval) {
                cr.move_to(x as f64, interval_large_height);

                let h = if x % (interval * 10) == 0 {
                    interval_large_height
                } else if x % (interval * 2) == 0 {
                    interval_height
                } else {
                    interval_small_height
                };

                cr.rel_line_to(0f64, -h as f64);

                if x % (interval * 10) == 0 {
                    cr.move_to(x as f64 + 2.0, interval_height as f64);
                    cr.show_text(&gst::ClockTime::from_mseconds(x as u64 * scaler as u64).to_string()[0..10]);
                }
            }

            cr.stroke();
            Inhibit(false)
        });
    }

    pub fn connect_get_scale(self_: Rc<RefCell<RulerWidget>>, cb: Box<Fn() -> f64>) {
        self_.borrow_mut().cb_get_scale = cb;
    }
}

impl AsWidget for RulerWidget {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &gtk::DrawingArea {
        &self.canvas
    }
}


