use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
use gtk::prelude::*;
use gdk::prelude::*;

use widget::AsWidget;

#[derive(Clone, Debug)]
pub struct BoxObject {
    pub index: usize,
    pub x: i32,
    pub width: i32,
    pub label: String,
    pub layer_index: usize,
    pub selected: bool,
}

impl BoxObject {
    pub const HEIGHT: i32 = 50;
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

    pub fn coordinate(&self) -> (i32,i32) { (self.x, self.layer_index as i32 * BoxObject::HEIGHT) }
    pub fn size(&self) -> (i32,i32) { (self.width, BoxObject::HEIGHT) }

    pub fn hscaled(self, scaler: f64) -> Self {
        BoxObject {
            index: self.index,
            x: (*&self.x as f64 / scaler) as i32,
            width: (self.width as f64 / scaler) as i32,
            label: self.label,
            layer_index: self.layer_index,
            selected: self.selected,
        }
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        self.coordinate().0 <= x && x <= self.coordinate().0 + self.size().0
            && self.coordinate().1 <= y && y <= self.coordinate().1 + self.size().1
    }
}

pub struct BoxViewerWidget {
    canvas: gtk::DrawingArea,
    objects: Vec<BoxObject>,
    offset: i32,
    selecting_box_index: Option<usize>,
    flag_resize: bool,
    cb_click_no_box: Box<Fn(&gdk::EventButton)>,
    cb_get_scale: Box<Fn() -> f64>,
    cb_motion_notify: Box<Fn(&gdk::EventMotion)>,
}

impl BoxViewerWidget {
    pub fn new(height: i32) -> Rc<RefCell<BoxViewerWidget>> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, height);

        Rc::new(RefCell::new(BoxViewerWidget {
            canvas: canvas,
            objects: vec![],
            offset: 0,
            selecting_box_index: None,
            flag_resize: false,
            cb_click_no_box: Box::new(|_| {}),
            cb_get_scale: Box::new(|| { 1.0 }),
            cb_motion_notify: Box::new(|_| {}),
        }))
    }

    pub fn set_model(&mut self, objects: Vec<BoxObject>) {
        self.objects = objects
    }

    pub fn get_selected_object(&self) -> Option<&BoxObject> {
        self.selecting_box_index.map(|u| &self.objects[u])
    }

    pub fn setup<T: 'static + AsRef<BoxObject>>(self_: Rc<RefCell<BoxViewerWidget>>, requester: Box<Fn() -> Vec<T>>, renderer: Box<Fn(&T, f64, &cairo::Context)>) {
        let self__ = self_.clone();
        let req = Rc::new(requester);
        let renderer = Rc::new(renderer);

        self_.borrow().canvas.connect_draw(move |_,cr| {
            let objects = (*req)();
            let scaler = (self__.borrow().cb_get_scale)();
            self__.borrow_mut().objects = objects.into_iter().map(|wrapper| {
                let object = wrapper.as_ref().clone().hscaled(scaler);
                renderer(&wrapper, scaler, cr);
                object
            }).collect();

            Inhibit(false)
        });
    }

    pub fn connect_select_box(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(usize, &gdk::EventButton)>) {
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
                cont(object.index, event);
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

    pub fn connect_motion_notify_event(self_: Rc<RefCell<BoxViewerWidget>>, cont: Box<Fn(&gdk::EventMotion)>) {
        self_.borrow_mut().cb_motion_notify = cont;
    }

    pub fn connect_drag_box(self_: Rc<RefCell<BoxViewerWidget>>, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        let self__ = self_.clone();
        self_.borrow().canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self_.borrow().canvas.connect_motion_notify_event(move |canvas,event| {
            (self__.borrow().cb_motion_notify)(event);

            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let objects = self__.borrow().objects.clone();
            let window = canvas.get_window().unwrap();
            let scale = (self__.borrow().cb_get_scale)();

            if event.get_state().contains(gdk::ModifierType::BUTTON1_MASK) && self__.borrow().selecting_box_index.is_some() {
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

