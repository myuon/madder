extern crate gdk;
extern crate cairo;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde_json;
use gdk::prelude::*;
use gdk_pixbuf::prelude::*;

use madder_core::*;
use widget::*;

pub struct TimelineComponentRenderer {
    pub object: BoxObject,
    pub object_type: String,
}

impl TimelineComponentRenderer {
    const HEIGHT: i32 = 50;
    const EDGE_WIDTH: i32 = 15;

    pub fn hscaled(&self, scaler: f64) -> Self {
        TimelineComponentRenderer {
            object: self.object.clone().hscaled(scaler),
            object_type: self.object_type.clone(),
        }
    }

    pub fn renderer(&self, cr: &cairo::Context, peek: &Fn(gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>) {
        if self.object.selected {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
            cr.rectangle(self.object.coordinate().0 as f64 - 2.0, self.object.coordinate().1 as f64 - 2.0, self.object.size().0 as f64 + 4.0, self.object.size().1 as f64 + 4.0);
            cr.stroke();
        }

        cr.set_source_rgba(0.0, 0.5, 1.0, 0.5);
        cr.rectangle(self.object.coordinate().0 as f64, self.object.coordinate().1.into(), self.object.size().0 as f64 - Self::EDGE_WIDTH as f64, self.object.size().1.into());
        cr.fill();
        cr.stroke();
        cr.set_source_rgba(0.5, 0.5, 0.5, 0.5);
        cr.rectangle(self.object.coordinate().0 as f64 + self.object.size().0 as f64 - Self::EDGE_WIDTH as f64, self.object.coordinate().1.into(), Self::EDGE_WIDTH as f64, self.object.size().1.into());
        cr.fill();

        match self.object_type.as_str() {
            "Video" => {
                for i in 0..(self.object.size().0 / BoxObject::HEIGHT) {
                    if let Some(pixbuf) = peek((i * Self::HEIGHT) as u64 * gst::MSECOND) {
                        cr.set_source_pixbuf(
                            &pixbuf.scale_simple(BoxObject::HEIGHT, BoxObject::HEIGHT, gdk_pixbuf::InterpType::Nearest).unwrap(),
                            (self.object.coordinate().0 + BoxObject::HEIGHT * i) as f64,
                            self.object.coordinate().1 as f64
                        );
                        cr.rectangle(
                            (self.object.coordinate().0 + BoxObject::HEIGHT * i) as f64,
                            self.object.coordinate().1 as f64,
                            self.object.size().0 as f64,
                            self.object.size().1 as f64
                        );
                        cr.fill();
                    }
                }
            },
            "Image" => {
                let pixbuf = peek(0 * gst::MSECOND).unwrap();
                cr.set_source_pixbuf(&pixbuf.scale_simple(50, 50, gdk_pixbuf::InterpType::Nearest).unwrap(), self.object.coordinate().0 as f64, self.object.coordinate().1 as f64);
                cr.rectangle(self.object.coordinate().0 as f64, self.object.coordinate().1 as f64, self.object.size().0 as f64, self.object.size().1 as f64);
                cr.fill();
            },
            _ => (),
        }

        cr.stroke();

        cr.save();
        cr.rectangle(self.object.coordinate().0.into(), self.object.coordinate().1.into(), self.object.size().0 as f64, self.object.size().1.into());
        cr.clip();

        let font_extent = cr.font_extents();
        cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(15.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.move_to(self.object.coordinate().0.into(), self.object.coordinate().1 as f64 - font_extent.descent + font_extent.height / 2.0 + self.object.size().1 as f64 / 2.0);
        cr.show_text(self.object.label.as_str());
        cr.stroke();
        cr.restore();
    }
}

impl AsRef<BoxObject> for TimelineComponentRenderer {
    fn as_ref(&self) -> &BoxObject {
        &self.object
    }
}

pub struct EffectComponentRenderer {
    effect: Effect,
    object: BoxObject,
    index: usize,
}

impl EffectComponentRenderer {
    const WIDTH: i32 = 300;
    const HEIGHT: i32 = 50;

    pub fn new(index: usize, effect: Effect) -> EffectComponentRenderer {
        EffectComponentRenderer {
            effect: effect,
            object: BoxObject::new(0, Self::WIDTH, index).layer_index(index),
            index: index,
        }
    }

    pub fn renderer(&self, _: f64, cr: &cairo::Context) {
        let font_extent = cr.font_extents();
        cr.select_font_face("Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
        cr.set_font_size(15.0);

        cr.set_source_rgba(1.0, 0.5, 0.0, 0.5);
        cr.rectangle(self.object.coordinate().0 as f64, self.object.coordinate().1.into(), self.object.size().0 as f64, self.object.size().1.into());
        cr.fill();
        cr.stroke();

        let render_point = move |x: f64, value: f64| {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.move_to(x, (self.index as f64 + 0.5) * Self::HEIGHT as f64 - font_extent.descent + font_extent.height / 2.0);
            cr.show_text("◆");
            cr.rel_move_to(0.0, font_extent.height);
            cr.show_text(&format!("{:.*}", 1, value));
            cr.stroke();
        };

        render_point(0.0, self.effect.start_value);
        render_point(Self::WIDTH as f64, self.effect.end_value);

        for intermed in &self.effect.intermeds {
            render_point(intermed.position * Self::WIDTH as f64, intermed.value);
        }
    }
}

impl AsRef<BoxObject> for EffectComponentRenderer {
    fn as_ref(&self) -> &BoxObject {
        &self.object
    }
}

impl HasEffect for EffectComponentRenderer {
    fn as_effect(&self) -> &Effect {
        &self.effect
    }
}

