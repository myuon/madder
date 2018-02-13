extern crate gtk;
extern crate glib;
use glib::IsA;

pub trait WidgetWrapper {
    type T: IsA<gtk::Widget>;
    fn to_widget(&self) -> &Self::T;
}

