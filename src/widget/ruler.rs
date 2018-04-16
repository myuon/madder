use std::rc::Rc;

extern crate gstreamer as gst;
extern crate gtk;
extern crate cairo;
use gtk::prelude::*;
use gdk::ContextExt;

extern crate relm;
extern crate relm_attributes;
extern crate relm_derive;
use relm_attributes::widget;
use relm::*;

#[derive(Clone)]
pub struct Model {
    pointer: f64,
    width: i32,
    height: i32,
    scale: Rc<gtk::Scale>,
}

#[derive(Msg)]
pub enum RulerMsg {
    Draw,
    MovePointer(f64),
}

#[widget]
impl Widget for RulerWidget {
    fn model(_: &Relm<Self>, (width, height, scale): (i32, i32, Rc<gtk::Scale>)) -> Model {
        Model {
            pointer: 0.0,
            width: width,
            height: height,
            scale: scale,
        }
    }

    fn update(&mut self, event: RulerMsg) {
        use self::RulerMsg::*;

        match event {
            Draw => {
                let cr = cairo::Context::create_from_window(&self.canvas.get_window().unwrap());
                cr.set_line_width(1.0);
                cr.set_source_rgb(0.0, 0.0, 0.0);

                cr.move_to(0.0, self.model.height as f64);
                cr.line_to(self.model.width as f64, self.model.height as f64);

                cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
                cr.set_font_size(10 as f64);

                let interval_large_height = self.model.height as f64;
                let interval_height = self.model.height as f64 * 0.5;
                let interval_small_height = self.model.height as f64 * 0.25;
                let interval = 10;

                let scaler = self.model.scale.get_value();

                for x in (0..(((self.model.width / interval) as f64) / scaler) as i32).map(|x| x * interval) {
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
                cr.move_to(self.model.pointer, interval_large_height);
                cr.rel_line_to(-width/2.0, -height);
                cr.rel_line_to(width, 0.0);
                cr.rel_line_to(-width/2.0, height);
                cr.fill();

                cr.stroke();
            },
            MovePointer(pos) => {
                self.model.pointer = pos;
            },
        }
    }

    fn init_view(&mut self) {
        self.canvas.set_size_request(-1, self.model.height);
    }

    view! {
        #[name="canvas"]
        gtk::DrawingArea {
            draw(_,_) => (RulerMsg::Draw, Inhibit(false)),
        }
    }
}

