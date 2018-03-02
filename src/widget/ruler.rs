extern crate gtk;
use gtk::prelude::*;

use widget::AsWidget;

pub struct RulerWidget(gtk::DrawingArea);

impl RulerWidget {
    pub fn new(width: i32) -> RulerWidget {
        let ruler = gtk::DrawingArea::new();
        let ruler_height = 20f64;

        ruler.set_size_request(width, ruler_height as i32);

        ruler.connect_draw(move |_, cr| {
            cr.set_line_width(1.0);
            cr.set_source_rgb(0f64, 0f64, 0f64);

            cr.move_to(0f64, ruler_height as f64);
            cr.line_to(width as f64, ruler_height);

            let interval_large = 100;
            let interval_large_height = ruler_height;

            let interval = 10;
            let interval_height = ruler_height * 0.6;

            for x in (0..(width / interval)).map(|x| x * interval) {
                cr.move_to(x as f64, ruler_height);

                let h = if x % interval_large == 0 { interval_large_height } else { interval_height };
                cr.rel_line_to(0f64, -h as f64);
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


