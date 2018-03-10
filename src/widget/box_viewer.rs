use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate cairo;
use gtk::prelude::*;
use gdk::prelude::*;

use widget::AsWidget;

#[derive(Clone, Debug)]
pub struct BoxObject {
    pub index: usize,
    x: i32,
    width: i32,
    label: String,
    layer_index: usize,
    selected: bool,
}

impl BoxObject {
    const HEIGHT: i32 = 30;
    const EDGE_WIDTH: i32 = 15;

    pub fn new(x: i32, width: i32, index: usize) -> BoxObject {
        BoxObject {
            index: index,
            x: x,
            width: width,
            label: "".to_string(),
            layer_index: 0,
            selected: false,
        }
    }

    pub fn label(mut self: BoxObject, label: String) -> BoxObject { self.label = label; self }
    pub fn selected(mut self: BoxObject, selected: bool) -> BoxObject { self.selected = selected; self }
    pub fn layer_index(mut self: BoxObject, layer_index: usize) -> BoxObject { self.layer_index = layer_index; self }

    fn coordinate(&self) -> (i32,i32) { (self.x, self.layer_index as i32 * BoxObject::HEIGHT) }
    fn size(&self) -> (i32,i32) { (self.width, BoxObject::HEIGHT) }

    fn hscaled(self, scaler: f64) -> Self {
        BoxObject {
            index: self.index,
            x: (*&self.x as f64 / scaler) as i32,
            width: (self.width as f64 / scaler) as i32,
            label: self.label,
            layer_index: self.layer_index,
            selected: self.selected,
        }
    }

    fn renderer(&self, cr: &cairo::Context) {
        if self.selected {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.rectangle(self.coordinate().0 as f64 - 2.0, self.coordinate().1 as f64 - 2.0, self.size().0 as f64 + 4.0, self.size().1 as f64 + 4.0);
            cr.stroke();
        }

        cr.set_source_rgba(0.0, 0.5, 1.0, 0.5);
        cr.rectangle(self.coordinate().0 as f64, self.coordinate().1.into(), self.size().0 as f64 - BoxObject::EDGE_WIDTH as f64, self.size().1.into());
        cr.fill();
        cr.stroke();
        cr.set_source_rgba(0.5, 0.5, 0.5, 0.5);
        cr.rectangle(self.coordinate().0 as f64 + self.size().0 as f64 - BoxObject::EDGE_WIDTH as f64, self.coordinate().1.into(), BoxObject::EDGE_WIDTH as f64, self.size().1.into());
        cr.fill();
        cr.stroke();

        cr.save();
        cr.rectangle(self.coordinate().0.into(), self.coordinate().1.into(), self.size().0 as f64, self.size().1.into());
        cr.clip();

        let font_extent = cr.font_extents();
        cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(15.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(self.coordinate().0.into(), self.coordinate().1 as f64 - font_extent.descent + font_extent.height / 2.0 + self.size().1 as f64 / 2.0);
        cr.show_text(self.label.as_str());
        cr.stroke();
        cr.restore();
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        self.coordinate().0 <= x && x <= self.coordinate().0 + self.size().0
            && self.coordinate().1 <= y && y <= self.coordinate().1 + self.size().1
    }
}

pub struct BoxViewerWidget {
    canvas: gtk::DrawingArea,
    objects: Vec<BoxObject>,
    requester: Box<Fn() -> Vec<BoxObject>>,
    offset: i32,
    selecting_box_index: Option<usize>,
    flag_resize: bool,
    cb_click_no_box: Box<Fn(&gdk::EventButton)>,
    cb_get_scale: Box<Fn() -> f64>,
}

impl BoxViewerWidget {
    pub fn new(height: i32) -> Rc<RefCell<BoxViewerWidget>> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, height);

        let self_ = Rc::new(RefCell::new(BoxViewerWidget {
            canvas: canvas,
            objects: vec![],
            requester: Box::new(|| vec![]),
            offset: 0,
            selecting_box_index: None,
            flag_resize: false,
            cb_click_no_box: Box::new(|_| {}),
            cb_get_scale: Box::new(|| { 1.0 }),
        }));
        BoxViewerWidget::create_ui(self_.clone());

        self_
    }

    pub fn set_model(&mut self, objects: Vec<BoxObject>) {
        self.objects = objects
    }

    pub fn connect_request_objects(&mut self, cont: Box<Fn() -> Vec<BoxObject>>) {
        self.requester = cont;
    }

    fn create_ui(self_: Rc<RefCell<BoxViewerWidget>>) {
        let self__ = self_.clone();

        self_.borrow().canvas.connect_draw(move |_,cr| {
            let objects = (self__.borrow().requester)();
            let scale = (self__.borrow().cb_get_scale)();
            BoxViewerWidget::renderer(objects.clone(), cr, scale);
            self__.borrow_mut().objects = objects;

            Inhibit(false)
        });
    }

    fn renderer(objects: Vec<BoxObject>, cr: &cairo::Context, scaler: f64) {
        objects.into_iter().for_each(|object| {
            object.hscaled(scaler).renderer(cr);
        });
    }

    pub fn connect_select_box(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(usize)>) {
        let self__ = self_.clone();
        self_.borrow().canvas.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        self_.borrow().canvas.connect_button_press_event(move |_,event| {
            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let objects = self__.borrow().objects.clone();
            let scale = (self__.borrow().cb_get_scale)();
            if let Some(object) = objects.into_iter().find(|object| object.clone().hscaled(scale).contains(x,y)) {
                self__.borrow_mut().offset = x;
                self__.borrow_mut().selecting_box_index = Some(object.index);
                cont(object.index);
            } else {
                (self__.borrow().cb_click_no_box)(event);
            }

            Inhibit(false)
        });
    }

    pub fn connect_click_no_box(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(&gdk::EventButton)>) {
        self_.borrow_mut().cb_click_no_box = cont;
    }

    pub fn connect_get_scale(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn() -> f64>) {
        self_.borrow_mut().cb_get_scale = cont;
    }

    pub fn connect_drag_box(self_: Rc<RefCell<BoxViewerWidget>>, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        let self__ = self_.clone();
        self_.borrow().canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self_.borrow().canvas.connect_motion_notify_event(move |canvas,event| {
            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let objects = self__.borrow().objects.clone();
            let window = canvas.get_window().unwrap();
            let scale = (self__.borrow().cb_get_scale)();

            if event.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
                let distance = ((x - self__.borrow().offset) as f64 * scale) as i32;
                let layer_index = y / BoxObject::HEIGHT;

                if self__.borrow().flag_resize {
                    cont_resize(self__.borrow().selecting_box_index.unwrap(), distance);
                } else {
                    cont_move(self__.borrow().selecting_box_index.unwrap(), distance, layer_index as usize);
                }
                self__.borrow_mut().offset = event.get_position().0 as i32;
            } else {
                match objects.iter().find(|&object| object.contains(x,y)) {
                    Some(object) if object.coordinate().0 + (object.size().0 as f64 / scale) as i32 - BoxObject::EDGE_WIDTH <= x
                                 && x <= object.coordinate().0 + (object.size().0 as f64 / scale) as i32 => {
                        window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "e-resize"));
                        self__.borrow_mut().flag_resize = true;
                    },
                    _ => {
                        window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "default"));
                        self__.borrow_mut().flag_resize = false;
                    },
                }
            }

            Inhibit(false)
        });
    }
}

impl AsWidget for BoxViewerWidget {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &Self::T {
        &self.canvas
    }
}

