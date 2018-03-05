extern crate gstreamer as gst;
extern crate gtk;
extern crate cairo;
use gtk::prelude::*;

use widget::AsWidget;

pub struct RulerWidget(gtk::DrawingArea);

impl RulerWidget {
    pub fn new(width: i32) -> RulerWidget {
        let ruler = gtk::DrawingArea::new();
        let ruler_height = 30f64;

        ruler.set_size_request(width, ruler_height as i32);

        ruler.connect_draw(move |_, cr| {
            cr.set_line_width(1.0);
            cr.set_source_rgb(0f64, 0f64, 0f64);

            cr.move_to(0f64, ruler_height as f64);
            cr.line_to(width as f64, ruler_height);

            cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(10 as f64);

            let interval_large = 100;
            let interval_large_height = ruler_height;

            let interval = 10;
            let interval_height = ruler_height * 0.6;

            for x in (0..(width / interval)).map(|x| x * interval) {
                cr.move_to(x as f64, ruler_height);

                let h = if x % interval_large == 0 { interval_large_height } else { interval_height };
                cr.rel_line_to(0f64, -h as f64);

                if x % interval_large == 0 {
                    cr.move_to(x as f64 + 2.0, interval_height as f64 - 10.0);
                    cr.show_text(&gst::ClockTime::from_mseconds(x as u64).to_string()[0..10]);
                }
            }

            cr.stroke();
            Inhibit(false)
        });

        RulerWidget(ruler)
    }
}

impl AsWidget for RulerWidget {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &gtk::DrawingArea {
        &self.0
    }
}


