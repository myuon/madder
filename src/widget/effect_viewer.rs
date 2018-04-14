use std::rc::Rc;

extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate relm;
extern crate relm_attributes;
extern crate relm_derive;
use relm_attributes::widget;
use relm::*;

extern crate madder_core;
use madder_core::*;
use widget::{BoxObject, BoxViewerWidget};

pub struct Model {
    tracking_position: (f64, usize),
    name_list: gtk::Box,
    connect_get_effect: Box<Fn(usize) -> component::Effect>,
    connect_new_point: Box<Fn(usize, f64)>,
}

#[derive(Msg)]
pub enum EffectMsg {
    Hide(Rc<gtk::Window>),
}

#[widget]
impl<Renderer: 'static> Widget for EffectViewer<Renderer> where Renderer: AsRef<BoxObject> {
    fn model(_: &Relm<Self>, parameter: (i32, i32, i32)) -> Model {
        Model {
            tracking_position: (0.0, 0),
            name_list: gtk::Box::new(gtk::Orientation::Vertical, 0),
            connect_get_effect: Box::new(|_| unreachable!()),
            connect_new_point: Box::new(|_,_| unreachable!()),
        }
    }

    fn update(&mut self, event: EffectMsg) {
        use self::EffectMsg::*;

        match event {
            Hide(window) => {
                let window = &window;
                window.hide();
            },
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: gtk::Orientation::Horizontal,
                gtk::Overlay {
                    BoxViewerWidget<Renderer>(200, Rc::new(gtk::Scale::new(gtk::Orientation::Vertical, None))) {
                    }
                },
            },
            delete_event(window,_) => (EffectMsg::Hide(unsafe { Rc::from_raw(window) }), Inhibit(true)),
        },
    }
}

/*
impl<Renderer: 'static + AsRef<BoxObject>> EffectViewer<Renderer> {
    pub fn new() -> EffectViewer<Renderer> {
        let mut viewer = EffectViewer {
            viewer: BoxViewerWidget::new(200),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
            overlay: gtk::Overlay::new(),
            tracker: gtk::DrawingArea::new(),
            tracking_position: (0.0, 0),
            name_list: gtk::Box::new(gtk::Orientation::Vertical, 0),
            connect_get_effect: Box::new(|_| unreachable!()),
            connect_new_point: Box::new(|_,_| unreachable!()),
        };

        viewer.create_ui();
        viewer
    }

    pub fn connect_get_objects(&mut self, cont: Box<Fn() -> Vec<Renderer>>) {
        self.viewer.connect_get_objects = cont;
    }

    pub fn connect_render_object(&mut self, cont: Box<Fn(Renderer, f64, &cairo::Context)>) {
        self.viewer.connect_render_object = cont;
    }

    pub fn get_objects(&self) -> Vec<Renderer> {
        (self.viewer.connect_get_objects)()
    }

    pub fn get_effect(&self, index: usize) -> component::Effect {
        (self.connect_get_effect)(index)
    }

    pub fn setup(&mut self) {
        for child in &self.name_list.get_children() {
            self.name_list.remove(child);
        }

        for obj in self.get_objects() {
            let label = gtk::Label::new(format!("{}: {}", obj.as_ref().index, self.get_effect(obj.as_ref().index).value(0.75)).as_str());
            label.set_size_request(-1, BoxObject::HEIGHT);
            self.name_list.pack_start(&label, false, false, 0);
        }

        self.viewer.setup();
    }

    fn create_ui(&mut self) {
        let self_ = self as *mut Self;
        self.viewer.connect_select_box = Box::new(move |viewer, index, event| {
            let self_ = unsafe { self_.as_mut().unwrap() };
            self_.tracking_position = (event.get_position().0, index);
            viewer.as_widget().queue_draw();

            if event.get_button() == 3 {
                let ratio = event.get_position().0 / viewer.get_selected_object().unwrap().size().0 as f64;
                // SEGV
                (self_.connect_new_point)(index, ratio);
            }
        });

        self.name_list.set_size_request(30,-1);

        self.overlay.add(self.viewer.as_widget());
        self.overlay.add_overlay(&self.tracker);
        self.overlay.set_overlay_pass_through(&self.tracker, true);

        self.tracker.set_size_request(-1, -1);
        self.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&self.name_list, false, false, 0);
        hbox.pack_start(&self.overlay, true, true, 0);

        self.window.set_size_request(500, 200);
        self.window.add(&hbox);
        self.window.connect_delete_event(move |window,_| {
            window.hide();
            gtk::Inhibit(true)
        });

        let self_ = self as *mut Self;
        self.tracker.connect_draw(move |tracker,cr| {
            let self_ = unsafe { self_.as_ref().unwrap() };

            cr.set_source_rgb(200f64, 0f64, 0f64);

            cr.move_to(self_.tracking_position.0, 0.0);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });
    }

    pub fn popup(&self) {
        self.window.show_all();
    }

    pub fn queue_draw(&self) {
        self.viewer.as_widget().queue_draw();
    }
}

impl<M: AsRef<BoxObject>> AsWidget for EffectViewer<M> {
    type T = gtk::Window;

    fn as_widget(&self) -> &Self::T {
        &self.window
    }
}
*/
