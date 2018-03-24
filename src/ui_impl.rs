extern crate cairo;
extern crate serde_json;

use madder_core::*;
use widget::*;

pub struct TimelineComponentRenderer {
    pub object: BoxObject,
    pub object_type: ComponentType,
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
    const WIDTH: i32 = 150;

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
        cr.set_source_rgb(0.0, 0.0, 0.0);
        for intermed in &self.effect.intermeds {
            cr.move_to(intermed.position * Self::WIDTH as f64, (self.index as f64 + 0.5) * BoxObject::HEIGHT as f64 - font_extent.descent + font_extent.height / 2.0);
            cr.show_text("◆");
            cr.stroke();
        }
    }
}

impl AsRef<BoxObject> for EffectComponentRenderer {
    fn as_ref(&self) -> &BoxObject {
        &self.object
    }
}

