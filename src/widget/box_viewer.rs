use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate cairo;
use gtk::prelude::*;

use widget::WidgetWrapper;

#[derive(Clone)]
pub struct BoxObject {
    coordinate: (i32,i32),
    size: (i32,i32),
}

impl BoxObject {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> BoxObject {
        BoxObject {
            coordinate: (x,y),
            size: (width, height),
        }
    }

    fn renderer(&self, cr: &cairo::Context) {
        cr.rectangle(self.coordinate.0.into(), self.coordinate.1.into(), self.size.0.into(), self.size.1.into());
        cr.fill();
        cr.stroke();
    }
}

pub struct BoxViewerWidget {
    canvas: gtk::DrawingArea,
}

impl BoxViewerWidget {
    pub fn new(width: i32, height: i32) -> BoxViewerWidget {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(width, height);

        BoxViewerWidget {
            canvas: canvas,
        }
    }

    pub fn connect_draw(&self, objects: Box<Fn() -> Vec<BoxObject>>) {
        self.canvas.connect_draw(move |_,cr| {
            BoxViewerWidget::renderer(objects(), cr)
        });
    }

    fn renderer(objects: Vec<BoxObject>, cr: &cairo::Context) -> gtk::Inhibit {
        objects.iter().for_each(|object| {
            object.renderer(cr);
        });

        Inhibit(false)
    }
}

impl WidgetWrapper for BoxViewerWidget {
    type T = gtk::DrawingArea;

    fn to_widget(&self) -> &Self::T {
        &self.canvas
    }
}

