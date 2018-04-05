use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gtk;
extern crate cairo;
use gtk::prelude::*;

use widget::AsWidget;

pub trait RulerWidgetI {
    fn connect_get_scale(&self) -> f64 {
        1.0
    }
}

pub struct RulerWidget<M: RulerWidgetI> {
    canvas: gtk::DrawingArea,
    pointer: f64,
    model: Option<Rc<RefCell<M>>>,
    width: i32,
    height: i32,
}

impl<M: 'static + RulerWidgetI> RulerWidget<M> {
    pub fn new(width: i32, height: i32) -> RulerWidget<M> {
        let ruler = gtk::DrawingArea::new();
        ruler.set_size_request(-1, height);

        RulerWidget {
            canvas: ruler,
            pointer: 0.0,
            model: None,
            width: width,
            height: height,
        }
    }

    pub fn set_model(&mut self, model: Rc<RefCell<M>>) {
        self.model = Some(model);
    }

    pub fn create_ui(&mut self, width: i32, height: i32) {
        let self_ = self as *mut Self;
        self.canvas.connect_draw(move |_, cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

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

            let scaler = self_.model.as_ref().unwrap().borrow().connect_get_scale();

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

            let width = 10.0;
            let height = 5.0;
            cr.move_to(self_.pointer, interval_large_height);
            cr.rel_line_to(-width/2.0, -height);
            cr.rel_line_to(width, 0.0);
            cr.rel_line_to(-width/2.0, height);
            cr.fill();

            cr.stroke();
            Inhibit(false)
        });
    }

    pub fn send_pointer_position(&mut self, x: f64) {
        self.pointer = x;
    }

    pub fn queue_draw(&self) {
        self.canvas.queue_draw();
    }
}

impl<M: RulerWidgetI> AsWidget for RulerWidget<M> {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &gtk::DrawingArea {
        &self.canvas
    }
}


