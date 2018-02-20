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
        cr.set_source_rgb(0.0, 0.5, 1.0);
        cr.rectangle(self.coordinate.0.into(), self.coordinate.1.into(), self.size.0.into(), self.size.1.into());
        cr.fill();
        cr.stroke();
    }
}

pub struct BoxViewerWidget {
    canvas: gtk::DrawingArea,
    objects: Vec<BoxObject>,
    requester: Box<Fn() -> Vec<BoxObject>>,
}

impl BoxViewerWidget {
    pub fn new(width: i32, height: i32) -> Rc<RefCell<BoxViewerWidget>> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(width, height);

        Rc::new(RefCell::new(BoxViewerWidget {
            canvas: canvas,
            objects: vec![],
            requester: Box::new(|| vec![]),
        }))
    }

    pub fn set_model(&mut self, objects: Vec<BoxObject>) {
        self.objects = objects
    }

    pub fn connect_request_objects(&mut self, cont: Box<Fn() -> Vec<BoxObject>>) {
        self.requester = cont;
    }

    pub fn create_ui(self_: Rc<RefCell<BoxViewerWidget>>) {
        let self__ = self_.clone();

        self_.as_ref().borrow().canvas.connect_draw(move |_,cr| {
            let objects = (self__.as_ref().borrow().requester)();
            BoxViewerWidget::renderer(objects.clone(), cr);
            self__.as_ref().borrow_mut().objects = objects;

            Inhibit(false)
        });
    }

    fn renderer(objects: Vec<BoxObject>, cr: &cairo::Context) {
        objects.iter().for_each(|object| {
            object.renderer(cr);
        });
    }
}

impl WidgetWrapper for BoxViewerWidget {
    type T = gtk::DrawingArea;

    fn to_widget(&self) -> &Self::T {
        &self.canvas
    }
}

