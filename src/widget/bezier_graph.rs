extern crate gtk;
extern crate glib;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;

extern crate relm;
use relm::*;

extern crate madder_core;

pub struct Model {
}

#[derive(Msg)]
pub enum BezierGraphMsg {
}

pub struct BezierGraphWidget {
    canvas: gtk::DrawingArea,
}

impl Update for BezierGraphWidget {
    type Model = Model;
    type ModelParam = ();
    type Msg = BezierGraphMsg;

    fn model(_relm: &Relm<Self>, _: Self::ModelParam) -> Model {
        Model {
        }
    }

    fn update(&mut self, event: BezierGraphMsg) {
        match event {
        }
    }
}

impl Widget for BezierGraphWidget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.canvas.clone()
    }

    fn view(_relm: &Relm<Self>, _model: Self::Model) -> Self {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, 300);
        canvas.connect_draw(move |_,cr| {
            cr.move_to(0.0, 150.0);
            cr.set_line_width(1.0);
            cr.set_source_rgb(0.8, 0.8, 0.8);
            cr.rel_line_to(500.0, 0.0);
            cr.stroke();

            cr.move_to(0.0, 150.0);
            cr.set_line_width(2.0);
            cr.set_source_rgb(0.4, 0.4, 0.4);
            cr.curve_to(50.0, 150.0, 400.0, 50.0, 500.0, 0.0);
            cr.stroke();

            Inhibit(false)
        });

        BezierGraphWidget {
            canvas: canvas,
        }
    }
}

