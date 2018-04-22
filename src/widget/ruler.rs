use std::rc::Rc;
use std::cell::RefCell;

extern crate gstreamer as gst;
extern crate gtk;
extern crate cairo;
use gtk::prelude::*;

extern crate relm;
use relm::*;

#[derive(Clone)]
pub struct Model {
    pointer: Rc<RefCell<f64>>,
    width: i32,
    height: i32,
    scale: Rc<gtk::Scale>,
}

#[derive(Msg)]
pub enum RulerMsg {
    MovePointer(f64),
}

pub struct RulerWidget {
    model: Model,
    canvas: gtk::DrawingArea,
}

impl Update for RulerWidget {
    type Model = Model;
    type ModelParam = (i32, i32, Rc<gtk::Scale>);
    type Msg = RulerMsg;

    fn model(_: &Relm<Self>, (width, height, scale): (i32, i32, Rc<gtk::Scale>)) -> Model {
        Model {
            pointer: Rc::new(RefCell::new(0.0)),
            width: width,
            height: height,
            scale: scale,
        }
    }

    fn update(&mut self, event: RulerMsg) {
        use self::RulerMsg::*;

        match event {
            MovePointer(pos) => {
                *self.model.pointer.borrow_mut() = pos;
            },
        }
    }
}

impl Widget for RulerWidget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.canvas.clone()
    }

    fn view(_relm: &Relm<Self>, model: Self::Model) -> Self {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, model.height);

        let model_ = Rc::new(model.clone());
        canvas.connect_draw(move |_,cr| {
            let model = model_.as_ref();
            cr.set_line_width(1.0);
            cr.set_source_rgb(0.0, 0.0, 0.0);

            cr.move_to(0.0, model.height as f64);
            cr.line_to(model.width as f64, model.height as f64);

            cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(10 as f64);

            let interval_large_height = model.height as f64;
            let interval_height = model.height as f64 * 0.5;
            let interval_small_height = model.height as f64 * 0.25;
            let interval = 10;

            let scaler = model.scale.get_value();

            for x in (0..(((model.width / interval) as f64) / scaler) as i32).map(|x| x * interval) {
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
            cr.move_to(*model.pointer.borrow(), interval_large_height);
            cr.rel_line_to(-width/2.0, -height);
            cr.rel_line_to(width, 0.0);
            cr.rel_line_to(-width/2.0, height);
            cr.fill();

            cr.stroke();

            Inhibit(false)
        });

        RulerWidget {
            model: model,
            canvas: canvas,
        }
    }
}



