use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate glib;
extern crate cairo;
use gtk::prelude::*;

use widget::{AsWidget, BoxObject, BoxViewerWidget};

pub struct EffectViewer {
    viewer: Rc<RefCell<BoxViewerWidget>>,
    window: gtk::Window,
}

impl EffectViewer {
    pub fn new() -> EffectViewer {
        let widget = EffectViewer {
            viewer: BoxViewerWidget::new(200),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
        };

        widget.create_ui();
        widget
    }

    pub fn setup<T: 'static + AsRef<BoxObject>>(&self, requester: Box<Fn() -> Vec<T>>, renderer: Box<Fn(&T, f64, &cairo::Context)>) {
        BoxViewerWidget::setup(self.viewer.clone(), requester, renderer);
    }

    fn create_ui(&self) {
        self.window.set_size_request(500, 200);
        self.window.add(self.viewer.borrow().as_widget());
        self.window.connect_delete_event(move |window,_| {
            window.hide();
            gtk::Inhibit(true)
        });
    }

    pub fn connect_new_point(&self, cont: Box<Fn(usize, f64)>) {
        let viewer_ = self.viewer.clone();
        BoxViewerWidget::connect_select_box(self.viewer.clone(), Box::new(move |index, event| {
            if event.get_button() == 3 {
                cont(index, event.get_position().0 / viewer_.borrow().get_selected_object().unwrap().size().0 as f64);
            }
        }));
    }

    pub fn popup(&self) {
        self.window.show_all();
    }

    pub fn queue_draw(&self) {
        self.as_widget().queue_draw();
    }
}

impl AsWidget for EffectViewer {
    type T = gtk::Window;

    fn as_widget(&self) -> &Self::T {
        &self.window
    }
}

