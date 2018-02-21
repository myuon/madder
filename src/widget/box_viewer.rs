use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate cairo;
use gtk::prelude::*;

use widget::WidgetWrapper;

#[derive(Clone, Debug)]
pub struct BoxObject {
    pub index: usize,
    coordinate: (i32,i32),
    size: (i32,i32),
    label: String,
    selected: bool,
}

impl BoxObject {
    pub fn new(x: i32, y: i32, width: i32, height: i32, index: usize, label: String, selected: bool) -> BoxObject {
        BoxObject {
            index: index,
            coordinate: (x,y),
            size: (width, height),
            label: label,
            selected: selected,
        }
    }

    fn renderer(&self, cr: &cairo::Context) {
        if self.selected {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.rectangle(self.coordinate.0 as f64 - 2.0, self.coordinate.1 as f64 - 2.0, self.size.0 as f64 + 4.0, self.size.1 as f64 + 4.0);
            cr.stroke();
        }

        cr.set_source_rgba(0.0, 0.5, 1.0, 0.5);
        cr.rectangle(self.coordinate.0.into(), self.coordinate.1.into(), self.size.0.into(), self.size.1.into());
        cr.fill();
        cr.stroke();

        cr.save();
        cr.rectangle(self.coordinate.0.into(), self.coordinate.1.into(), self.size.0.into(), self.size.1.into());
        cr.clip();

        let font_extent = cr.font_extents();
        cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(15.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(self.coordinate.0.into(), self.coordinate.1 as f64 - font_extent.descent + font_extent.height / 2.0 + self.size.1 as f64 / 2.0);
        cr.show_text(self.label.as_str());
        cr.stroke();
        cr.restore();
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        self.coordinate.0 <= x && x <= self.coordinate.0 + self.size.0
            && self.coordinate.1 <= y && y <= self.coordinate.1 + self.size.1
    }
}

pub struct BoxViewerWidget {
    canvas: gtk::DrawingArea,
    objects: Vec<BoxObject>,
    requester: Box<Fn() -> Vec<BoxObject>>,
    offset: i32,
    selecting_box_index: Option<usize>,
}

impl BoxViewerWidget {
    pub fn new(width: i32, height: i32) -> Rc<RefCell<BoxViewerWidget>> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(width, height);

        Rc::new(RefCell::new(BoxViewerWidget {
            canvas: canvas,
            objects: vec![],
            requester: Box::new(|| vec![]),
            offset: 0,
            selecting_box_index: None,
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

    pub fn connect_select_box(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(usize)>) {
        let self__ = self_.clone();
        self_.as_ref().borrow().canvas.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        self_.as_ref().borrow().canvas.connect_button_press_event(move |_,event| {
            let (x,y) = event.get_position();
            let objects = self__.as_ref().borrow().objects.clone();
            if let Some(object) = objects.iter().find(|&object| object.contains(x as i32, y as i32)) {
                self__.as_ref().borrow_mut().offset = event.get_position().0 as i32;
                self__.as_ref().borrow_mut().selecting_box_index = Some(object.index);
                cont(object.index);
            }
            Inhibit(false)
        });
    }

    pub fn connect_drag_box(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(usize, i32)>) {
        let self__ = self_.clone();
        self_.as_ref().borrow().canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self_.as_ref().borrow().canvas.connect_motion_notify_event(move |_,event| {
            if event.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
                cont(self__.as_ref().borrow().selecting_box_index.unwrap(), event.get_position().0 as i32 - self__.as_ref().borrow().offset);
                self__.as_ref().borrow_mut().offset = event.get_position().0 as i32;
            }

            Inhibit(false)
        });
    }
}

impl WidgetWrapper for BoxViewerWidget {
    type T = gtk::DrawingArea;

    fn to_widget(&self) -> &Self::T {
        &self.canvas
    }
}

