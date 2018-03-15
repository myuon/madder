extern crate gtk;
extern crate glib;
use glib::IsA;

pub trait AsWidget {
    type T: IsA<gtk::Widget>;
    fn as_widget(&self) -> &Self::T;
}

