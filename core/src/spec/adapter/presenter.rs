extern crate gdk_pixbuf;
extern crate gstreamer as gst;
use gdk_pixbuf::prelude::*;
use std::cmp;
use spec::*;

pub trait HavePresenter : HaveProject + HaveComponentRepository + HaveEffectRepository {
    fn get_pixbuf(&self, position: gst::ClockTime) -> gdk_pixbuf::Pixbuf {
        let pixbuf = gdk_pixbuf::Pixbuf::new(
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            self.project().size.0,
            self.project().size.1
        );

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        for layer in self.project().list_layers().iter().rev() {
            for component in layer.list().iter().map(|component_id| {
                self.component_repo().get(component_id)
            }).filter(|component| {
                component.component().start_time <= position &&
                    position <= component.component().end_time()
            }) {
                if let Some(dest) = component.get_pixbuf(position) {
                    let coordinate = (0,0);
                    let scale = (1.0,1.0);
                    let alpha = 255;

                    &dest.as_ref().composite(
                        &pixbuf, coordinate.0, coordinate.1,
                        cmp::min(dest.get_width(), self.project().size.0 - coordinate.0),
                        cmp::min(dest.get_height(), self.project().size.1 - coordinate.1),
                        coordinate.0.into(), coordinate.1.into(),
                        scale.0, scale.1,
                        gdk_pixbuf::InterpType::Nearest, alpha);
                }
            }
        }

        pixbuf
    }
}

