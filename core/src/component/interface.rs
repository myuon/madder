extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use component::effect::*;
use component::property::*;

pub trait EffectOn {
    fn effect_on_component(&self, component: GeometryProperty, current: f64) -> GeometryProperty;
}

impl EffectOn for Effect {
    fn effect_on_component(&self, component: GeometryProperty, current: f64) -> GeometryProperty {
        use EffectType::*;

        match self.effect_type {
            CoordinateX => {
                let mut comp = component;
                comp.coordinate.0 += self.value(current) as i32;
                comp
            },
            CoordinateY => {
                let mut comp = component;
                comp.coordinate.1 += self.value(current) as i32;
                comp
            },
            ScaleX => {
                let mut comp = component;
                comp.scale.0 *= self.value(current);
                comp
            },
            ScaleY => {
                let mut comp = component;
                comp.scale.1 *= self.value(current);
                comp
            },
            Alpha => {
                let mut comp = component;
                comp.alpha = (comp.alpha as f64 * self.value(current) / 255.0) as i32;
                comp
            },
            _ => component,
        }
    }
}

pub trait Peekable {
    fn get_duration(&self) -> gst::ClockTime;
    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>;
}

pub trait AsProperty {
    fn as_component(&self) -> &ComponentProperty;
    fn as_component_mut(&mut self) -> &mut ComponentProperty;

    fn as_geometry(&self) -> Option<&GeometryProperty>;
    fn as_geometry_mut(&mut self) -> Option<&mut GeometryProperty>;
}

/*
pub trait ComponentWrapper {
    fn as_value(&self) -> serde_json::Value;

    fn as_effect_value(&self) -> Vec<serde_json::Value> {
        self.as_effect().iter().map(|vec| {
            serde_json::Value::Object(vec.iter().cloned().collect())
        }).collect()
    }

    fn as_effect(&self) -> Vec<Vec<(String, serde_json::Value)>> {
        self.as_component().effect.iter().map(|eff| eff.get_props()).collect()
    }

    fn as_effect_attrs(&self) -> Vec<Vec<(String, Attribute)>> {
        self.as_component().effect.iter().map(|eff| eff.get_attrs()).collect()
    }

    fn as_effect_mut(&mut self) -> &mut Vec<serde_json::Value> {
        unimplemented!()
    }

    fn get_info(&self) -> String;

    fn get_audio_pipeline(&self) -> Option<&gst::Pipeline> {
        None
    }
}
*/

