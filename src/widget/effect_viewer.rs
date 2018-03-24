use std::rc::Rc;
use std::cell::RefCell;

extern crate gtk;
extern crate glib;
use gtk::prelude::*;

use widget::{AsWidget, BoxObject, BoxViewerWidget};

pub struct EffectViewer {
    name_box: gtk::Box,
    viewer: Rc<RefCell<BoxViewerWidget>>,
    window: gtk::Window,
}

impl EffectViewer {
    pub fn new() -> EffectViewer {
        let widget = EffectViewer {
            name_box: gtk::Box::new(gtk::Orientation::Vertical, 0),
            viewer: BoxViewerWidget::new(200),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
        };

        widget.create_ui();
        widget
    }

    pub fn setup<T: 'static + AsRef<BoxObject>>(&self, requester: Box<Fn() -> Vec<T>>) {
        BoxViewerWidget::setup(self.viewer.clone(), requester, Box::new(|_,_,_| {}));
    }

    fn create_ui(&self) {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        hbox.pack_start(&self.name_box, true, true, 0);
        hbox.pack_start(self.viewer.borrow().as_widget(), true, true, 0);

        self.window.add(&hbox);
        self.window.connect_delete_event(move |window,_| {
            window.hide();
            gtk::Inhibit(true)
        });
    }

    pub fn popup(&self) {
        self.window.show_all();
    }
}

impl AsWidget for EffectViewer {
    type T = gtk::Window;

    fn as_widget(&self) -> &Self::T {
        &self.window
    }
}

