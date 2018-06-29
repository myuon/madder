extern crate gdk_pixbuf;

pub trait PixbufService {
    fn get_pixbuf(&self) -> gdk_pixbuf::Pixbuf;
}

pub trait HavePixbufService {
    type SERVICE;

    fn pixbuf_service(&self) -> Self::SERVICE;
}

