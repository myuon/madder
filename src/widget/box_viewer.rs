use std::rc::Rc;

extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate relm;
use relm::*;

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

pub struct Model<Renderer: AsRef<BoxObject> + 'static> {
    offset: i32,
    selecting_box_index: Option<usize>,
    flag_resize: bool,
    scale: Rc<gtk::Scale>,
    height: i32,
    on_get_object: Rc<Box<Fn() -> Vec<Renderer>>>,
    on_render: Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>,
    relm: Relm<BoxViewerWidget<Renderer>>,
}

#[derive(Msg)]
pub enum BoxViewerMsg {
    Motion(gdk::EventMotion),
    Select(gdk::EventButton),
    OnSelect(usize, gdk::EventButton),
    OnSelectNoBox(gdk::EventButton),
    OnResize(usize, i32),
    OnDrag(usize, i32, usize),
}

pub struct BoxViewerWidget<Renderer: AsRef<BoxObject> + 'static> {
    model: Model<Renderer>,
    canvas: gtk::DrawingArea,
}

impl<Renderer> Update for BoxViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Model = Model<Renderer>;
    type ModelParam = (i32, Rc<gtk::Scale>, Rc<Box<Fn() -> Vec<Renderer>>>, Rc<Box<Fn(&Renderer, f64, &cairo::Context)>>);
    type Msg = BoxViewerMsg;

    fn model(relm: &Relm<Self>, (height, scale, on_get_object, on_render): Self::ModelParam) -> Model<Renderer> {
        Model {
            offset: 0,
            selecting_box_index: None,
            flag_resize: false,
            scale: scale,
            height: height,
            on_get_object: on_get_object,
            on_render: on_render,
            relm: relm.clone(),
        }
    }

    fn update(&mut self, msg: BoxViewerMsg) {
        use self::BoxViewerMsg::*;

        match msg {
            Select(event) => {
                let event = &event;
                let (x,y) = event.get_position();
                let x = x as i32;
                let y = y as i32;
                let scale = self.model.scale.get_value();

                if let Some(object) = (self.model.on_get_object)().iter().find(|object| object.as_ref().clone().hscaled(scale).contains(x,y)) {
                    self.model.offset = x;
                    self.model.selecting_box_index = Some(object.as_ref().index);
                    self.model.relm.stream().emit(BoxViewerMsg::OnSelect(object.as_ref().index, event.clone()));
                } else {
                    self.model.relm.stream().emit(BoxViewerMsg::OnSelectNoBox(event.clone()));
                }
            },
            Motion(event) => {
                let (x,y) = event.get_position();
                let x = x as i32;
                let y = y as i32;

                let window = self.canvas.get_window().unwrap();
                let scale = self.model.scale.get_value();

                if event.get_state().contains(gdk::ModifierType::BUTTON1_MASK) && self.model.selecting_box_index.is_some() {
                    let distance = ((x - self.model.offset) as f64 * scale) as i32;
                    let layer_index = y / BoxObject::HEIGHT;

                    let index = self.model.selecting_box_index.unwrap();
                    if self.model.flag_resize {
                        self.model.relm.stream().emit(BoxViewerMsg::OnResize(index, distance));
                    } else {
                        self.model.relm.stream().emit(BoxViewerMsg::OnDrag(index, distance, layer_index as usize));
                    }
                    self.model.offset = event.get_position().0 as i32;
                } else {
                    match (self.model.on_get_object)().iter().find(|object| object.as_ref().contains(x,y)).map(|x| x.as_ref().clone()) {
                        Some(ref object) if object.coordinate().0 + (object.size().0 as f64 / scale) as i32 - BoxObject::EDGE_WIDTH <= x && x <= object.coordinate().0 + (object.size().0 as f64 / scale) as i32 => {
                            window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "e-resize"));
                            self.model.flag_resize = true;
                        },
                        _ => {
                            window.set_cursor(&gdk::Cursor::new_from_name(&window.get_display(), "default"));
                            self.model.flag_resize = false;
                        },
                    }
                }
            },
            _ => (),
        }
    }
}

impl<Renderer> Widget for BoxViewerWidget<Renderer> where Renderer: AsRef<BoxObject> + 'static {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.canvas.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let canvas = gtk::DrawingArea::new();
        canvas.set_size_request(-1, model.height);
        canvas.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        canvas.add_events(gdk::EventMask::POINTER_MOTION_MASK.bits() as i32);

        let on_get_object = model.on_get_object.clone();
        let on_render = model.on_render.clone();
        let scale = model.scale.clone();
        connect!(relm, canvas, connect_draw(_,cr), return {
            for object in on_get_object().iter() {
                on_render(object, scale.get_value(), cr);
            }

            Inhibit(false)
        });

        connect!(relm, canvas, connect_button_press_event(_, event), return (Some(BoxViewerMsg::Select(event.clone())), Inhibit(false)));
        connect!(relm, canvas, connect_motion_notify_event(_, event), return (Some(BoxViewerMsg::Motion(event.clone())), Inhibit(false)));

        BoxViewerWidget {
            model: model,
            canvas: canvas,
        }
    }
}

