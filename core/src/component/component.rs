use std::ops::{Deref, DerefMut};

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use component::effect::*;
use component::attribute::*;
use component::property::*;

pub trait EffectOn {
    fn effect_on_component(&self, component: CommonProperty, current: f64) -> CommonProperty;
}

impl EffectOn for Effect {
    fn effect_on_component(&self, component: CommonProperty, current: f64) -> CommonProperty {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ComponentType {
    Video,
    Image,
    Text,
    Sound,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Component {
    pub component_type: ComponentType,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub length: gst::ClockTime,

    pub layer_index: usize,

    #[serde(default = "Vec::new")]
    pub effect: Vec<Effect>,
}

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

pub trait ComponentWrapper : AsRef<Component> + AsMut<Component> {
    fn as_value(&self) -> serde_json::Value;

    fn as_effect(&self) -> &Vec<serde_json::Value> {
        //        self.as_ref().effect.iter().map(|eff| eff.get_props()).collect()
        unimplemented!()
    }

    fn as_effect_mut(&mut self) -> &mut Vec<serde_json::Value> {
        unimplemented!()
    }

    fn get_info(&self) -> String;

    fn get_audio_pipeline(&self) -> Option<&gst::Pipeline> {
        None
    }
}

impl AsRef<Component> for Component {
    fn as_ref(&self) -> &Component {
        self
    }
}

impl AsMut<Component> for Component {
    fn as_mut(&mut self) -> &mut Component {
        self
    }
}

impl ComponentWrapper for Component {
    fn as_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    fn get_info(&self) -> String {
        format!("Component")
    }
}

pub trait HasProperty {
    fn get_attrs(&self) -> Vec<(String, Attribute)>;
    fn get_attr(&self, name: &str) -> Attribute;
    fn set_attr(&mut self, name: &str, attr: Attribute);

    fn get_props(&self) -> Vec<(String, serde_json::Value)> {
        self.get_attrs().iter().map(|&(ref k, ref attr)| (k.to_string(), serde_json::to_value(attr).unwrap().as_object().unwrap()["value"].clone())).collect()
    }

    fn get_prop(&self, name: &str) -> serde_json::Value {
        serde_json::to_value(self.get_attr(name)).unwrap().as_object().unwrap()["value"].clone()
    }

    fn set_prop(&mut self, name: &str, prop: serde_json::Value);
}

pub trait ComponentLike: ComponentWrapper + Peekable + HasProperty {}
impl<T: ComponentWrapper + Peekable + HasProperty> ComponentLike for T {}

impl Deref for ComponentLike {
    type Target = Component;

    fn deref(&self) -> &Component {
        self.as_ref()
    }
}

impl DerefMut for ComponentLike {
    fn deref_mut(&mut self) -> &mut Component {
        self.as_mut()
    }
}

