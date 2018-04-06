extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;
use gdk::prelude::*;

use widget::AsWidget;
use util::self_wrapper::*;

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

pub trait BoxViewerWidgetI {
    type Renderer : AsRef<BoxObject>;

    fn get_objects(&self) -> Vec<Self::Renderer>;
    fn do_render(&self, Self::Renderer, f64, &cairo::Context);
    fn connect_select_box(&mut self, usize, &gdk::EventButton) {}
    fn connect_select_no_box(&self, &gdk::EventButton) {}
    fn connect_motion_notify_event(&mut self, &gdk::EventMotion) {}
    fn connect_get_scale(&self) -> f64 { 1.0 }
}

#[derive(Clone)]
pub struct BoxViewerWidget<M: BoxViewerWidgetI> {
    canvas: gtk::DrawingArea,
    offset: i32,
    selecting_box_index: Option<usize>,
    flag_resize: bool,
    model: Option<Model<M>>,
}

impl<M: 'static + BoxViewerWidgetI> BoxViewerWidget<M> {
    pub fn new(height: i32) -> BoxViewerWidget<M> {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, height);

        BoxViewerWidget {
            canvas: canvas,
            offset: 0,
            selecting_box_index: None,
            flag_resize: false,
            model: None,
        }
    }

    pub fn set_model(&mut self, model: &mut M) {
        self.model = Some(Model::new(model));
    }

    pub fn get_selected_object(&self) -> Option<BoxObject> {
        let model = self.model.as_ref().unwrap().as_ref();
        self.selecting_box_index.map(|u| model.get_objects()[u].as_ref().clone())
    }

    pub fn setup(&mut self) {
        let self_ = self as *mut BoxViewerWidget<M>;
        self.canvas.connect_draw(move |_,cr| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let model = self_.model.as_ref().unwrap().as_ref();
            let objects = model.get_objects();
            let scaler = model.connect_get_scale();

            for wrapper in objects.into_iter() {
                model.do_render(wrapper, scaler, cr);
            }

            Inhibit(false)
        });

        let self_ = self as *mut BoxViewerWidget<M>;
        self.canvas.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        self.canvas.connect_button_press_event(move |_,event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let model = self_.model.as_ref().unwrap().as_ref();
            let scale = model.connect_get_scale();

            if let Some(object) = model.get_objects().into_iter().find(|object| object.as_ref().clone().hscaled(scale).contains(x,y)) {
                self_.offset = x;
                self_.selecting_box_index = Some(object.as_ref().index);
                model.connect_select_box(object.as_ref().index, event);
            } else {
                model.connect_select_no_box(event);
            }

            Inhibit(false)
        });
    }

    pub fn connect_drag_box(&mut self, cont_move: Box<Fn(usize, i32, usize)>, cont_resize: Box<Fn(usize, i32)>) {
        let self_ = self as *mut Self;
        self.canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);
        self.canvas.connect_motion_notify_event(move |canvas,event| {
            let self_ = unsafe { self_.as_mut().unwrap() };

            let model = self_.model.as_ref().unwrap().as_ref();
            model.connect_motion_notify_event(event);

            let (x,y) = event.get_position();
            let x = x as i32;
            let y = y as i32;

            let window = canvas.get_window().unwrap();
            let scale = model.connect_get_scale();

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
                let objects = model.get_objects();

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

impl<M: BoxViewerWidgetI> AsWidget for BoxViewerWidget<M> {
    type T = gtk::DrawingArea;

    fn as_widget(&self) -> &Self::T {
        &self.canvas
    }
}

