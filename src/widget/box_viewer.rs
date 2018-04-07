extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate gstreamer as gst;
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

pub struct BoxViewerWidget<Renderer: AsRef<BoxObject>> {
    canvas: gtk::DrawingArea,
    offset: i32,
    selecting_box_index: Option<usize>,
    flag_resize: bool,
    pub connect_get_objects: Box<Fn() -> Vec<Renderer>>,
    pub connect_render_object: Box<Fn(Renderer, f64, &cairo::Context)>,
    pub connect_select_box: Box<Fn(usize, &gdk::EventButton)>,
    pub connect_select_no_box: Box<Fn(&gdk::EventButton)>,
    pub connect_motion_notify_event: Box<Fn(&gdk::EventMotion)>,
    pub connect_get_scale: Box<Fn() -> f64>,
}

impl<Renderer: 'static + AsRef<BoxObject>> BoxViewerWidget<Renderer> {
    pub fn new(height: i32) -> BoxViewerWidget<Renderer> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, height);

        BoxViewerWidget {
            canvas: canvas,
            offset: 0,
            selecting_box_index: None,
            flag_resize: false,
            connect_get_objects: Box::new(|| { vec![] }),
            connect_render_object: Box::new(|_,_,_| {}),
            connect_select_box: Box::new(|_,_| {}),
            connect_select_no_box: Box::new(|_| {}),
            connect_motion_notify_event: Box::new(|_| {}),
            connect_get_scale: Box::new(|| { 1.0 }),
        }
    }

    pub fn get_selected_object(&self) -> Option<BoxObject> {
        self.selecting_box_index.map(|u| (self.connect_get_objects)()[u].as_ref().clone())
    }

    pub fn setup(&mut self) {
        let self_ = self as *mut BoxViewerWidget<_>;
        self.canvas.connect_draw(move |_,cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let objects = (self_.connect_get_objects)();
            let scaler = (self_.connect_get_scale)();

            for wrapper in objects.into_iter() {
                (self_.connect_render_object)(wrapper, scaler, cr);
            }

            Inhibit(false)
        });

        let self_ = self as *mut BoxViewerWidget<Renderer>;
        self.canvas.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        self.canvas.connect_button_press_event(move |_,event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let scale = (self_.connect_get_scale)();

            if let Some(object) = (self_.connect_get_objects)().into_iter().find(|object| object.as_ref().clone().hscaled(scale).contains(x,y)) {
                self_.offset = x;
                self_.selecting_box_index = Some(object.as_ref().index);
                (self_.connect_select_box)(object.as_ref().index, event);
            } else {
                (self_.connect_select_no_box)(event);
            }

            Inhibit(false)
        });
    }

    pub fn connect_drag_box(&mut self, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        let self_ = self as *mut Self;
        self.canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self.canvas.connect_motion_notify_event(move |canvas,event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            (self_.connect_motion_notify_event)(event);

            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let window = canvas.get_window().unwrap();
            let scale = (self_.connect_get_scale)();

            if event.get_state().contains(gdk::ModifierType::BUTTON1_MASK) && self_.selecting_box_index.is_some() {
                let distance = ((x - self_.offset) as f64 * scale) as i32;
                let layer_index = y / BoxObject::HEIGHT;

                if self_.flag_resize {
                    cont_resize(self_.selecting_box_index.unwrap(), distance);
                } else {
                    cont_move(self_.selecting_box_index.unwrap(), distance, layer_index as usize);
                }
                self_.offset = event.get_position().0 as i32;
            } else {
                let objects = (self_.connect_get_objects)();

                match objects.iter().find(|object| object.as_ref().contains(x,y)).map(|x| x.as_ref().clone()) {
                    Some(ref object) if object.coordinate().0 + (object.size().0 as f64 / scale) as i32 - BoxObject::EDGE_WIDTH <= x && x <= object.coordinate().0 + (object.size().0 as f64 / scale) as i32 => {
                        window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "e-resize"));
                        self_.flag_resize = true;
                    },
                    _ => {
                        window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "default"));
                        self_.flag_resize = false;
                    },
                }
            }

            Inhibit(false)
        });
    }
}

impl<R: AsRef<BoxObject>> AsWidget for BoxViewerWidget<R> {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &Self::T {
        &self.canvas
    }
}

