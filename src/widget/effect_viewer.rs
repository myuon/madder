extern crate gtk;
extern crate glib;
use gtk::prelude::*;

use widget::AsWidget;

pub struct EffectViewer {
    container: gtk::Box,
    window: gtk::Window,
}

impl EffectViewer {
    pub fn new() -> EffectViewer {
        let widget = EffectViewer {
            container: gtk::Box::new(gtk::Orientation::Vertical, 0),
            window: gtk::Window::new(gtk::WindowType::Toplevel),
        };

        widget.create_ui();
        widget
    }

    pub fn pack_start<W: glib::IsA<gtk::Widget>>(&self, w: &W) {
        self.container.pack_start(w, true, true, 0);
    }

    pub fn clear(&mut self) {
        for w in self.container.get_children() {
            self.container.remove(&w);
        }
    }

    fn create_ui(&self) {
        self.window.add(&self.container);
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

