use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate gdk;
extern crate glib;
extern crate cairo;
extern crate gstreamer as gst;
use gtk::prelude::*;
use gdk::prelude::*;

extern crate madder_core;
use madder_core::*;
use widget::{AsWidget, BoxObject, BoxViewerWidget, BoxViewerWidgetI};

pub trait EffectViewerI {
    type Renderer : AsRef<BoxObject>;

    fn get_effect(&self, usize) -> component::Effect;
    fn get_effects(&self) -> Vec<Self::Renderer>;
    fn do_render(&self, Self::Renderer, f64, &cairo::Context);
    fn connect_new_point(&mut self, usize, f64) {}
}

impl<M: 'static + EffectViewerI> BoxViewerWidgetI for EffectViewer<M> {
    type Renderer = <M as EffectViewerI>::Renderer;

    fn get_objects(&self) -> Vec<Self::Renderer> {
        let inst = self.model.as_ref().unwrap().borrow();
        inst.get_effects()
    }

    fn do_render(&self, renderer: Self::Renderer, scaler: f64, cr: &cairo::Context) {
        let inst = self.model.as_ref().unwrap();
        let inst = inst.borrow();
        inst.do_render(renderer, scaler, cr);
    }

    fn connect_select_box(&mut self, index: usize, event: &gdk::EventButton) {
        self.tracking_position = (event.get_position().0, index);
        self.queue_draw();

        let inst = self.model.as_ref().unwrap();
        if event.get_button() == 3 {
            inst.borrow_mut().connect_new_point(index, event.get_position().0 / self.viewer.borrow().get_selected_object().unwrap().size().0 as f64);
        }
    }
}

pub struct EffectViewer<M: 'static + EffectViewerI> {
    viewer: Rc<RefCell<BoxViewerWidget<EffectViewer<M>>>>,
    window: gtk::Window,
    overlay: gtk::Overlay,
    tracker: gtk::DrawingArea,
    tracking_position: (f64, usize),
    name_list: gtk::Box,
    model: Option<Rc<RefCell<M>>>,
}

impl<M: 'static + EffectViewerI> EffectViewer<M> {
    pub fn new() -> Rc<RefCell<EffectViewer<M>>> {
        let viewer = Rc::new(RefCell::new(EffectViewer {
            viewer: BoxViewerWidget::new(200),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
            overlay: gtk::Overlay::new(),
            tracker: gtk::DrawingArea::new(),
            tracking_position: (0.0, 0),
            name_list: gtk::Box::new(gtk::Orientation::Vertical, 0),
            model: None,
        }));

        EffectViewer::create_ui(viewer.clone());
        viewer
    }

    pub fn set_model(&mut self, model: Rc<RefCell<M>>) {
        self.model = Some(model);
    }

    pub fn setup(self_: Rc<RefCell<EffectViewer<M>>>) {
        let this = self_.borrow();
        for child in &this.name_list.get_children() {
            this.name_list.remove(child);
        }

        let inst = this.model.as_ref().unwrap();
        for obj in inst.borrow().get_effects() {
            let inst = inst.borrow();
            let label = gtk::Label::new(format!("{}: {}", obj.as_ref().index, inst.get_effect(obj.as_ref().index).value(0.75)).as_str());
            label.set_size_request(-1, BoxObject::HEIGHT);
            this.name_list.pack_start(&label, false, false, 0);
        }

        this.viewer.borrow_mut().setup();
    }

    fn create_ui(self_: Rc<RefCell<EffectViewer<M>>>) {
        let this = self_.borrow();

        let self__ = self_.clone();
        this.viewer.borrow_mut().set_model(self__);

        this.name_list.set_size_request(30,-1);

        this.overlay.add(this.viewer.borrow().as_widget());
        this.overlay.add_overlay(&this.tracker);
        this.overlay.set_overlay_pass_through(&this.tracker, true);

        this.tracker.set_size_request(-1, -1);
        this.tracker.connect_realize(move |tracker| {
            let window = tracker.get_window().unwrap();
            window.set_pass_through(true);
        });

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&this.name_list, false, false, 0);
        hbox.pack_start(&this.overlay, true, true, 0);

        this.window.set_size_request(500, 200);
        this.window.add(&hbox);
        this.window.connect_delete_event(move |window,_| {
            window.hide();
            gtk::Inhibit(true)
        });

        let self__ = self_.clone();
        this.tracker.connect_draw(move |tracker,cr| {
            cr.set_source_rgb(200f64, 0f64, 0f64);

            cr.move_to(self__.borrow().tracking_position.0, 0.0);
            cr.rel_line_to(0.0, tracker.get_allocation().height as f64);
            cr.stroke();

            Inhibit(false)
        });
    }

    pub fn popup(&self) {
        self.window.show_all();
    }

    pub fn queue_draw(&self) {
        self.as_widget().queue_draw();
    }
}

impl<M: EffectViewerI> AsWidget for EffectViewer<M> {
    type T = gtk::Window;

    fn as_widget(&self) -> &Self::T {
        &self.window
    }
}

