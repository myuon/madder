use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;
extern crate madder_util as util;

use util::serde_impl::*;
use component::effect::*;
use component::property::*;
use component::attribute::*;

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

impl ComponentType {
    fn types() -> Vec<ComponentType> {
        use ComponentType::*;

        vec![Video, Image, Text, Sound]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Component {
    pub component_type: ComponentType,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub length: gst::ClockTime,

    pub layer_index: usize,

    #[serde(default = "Vec::new")]
    pub effect: Vec<Effect>,
}

pub trait ComponentWrapper {
    fn as_value(&self) -> serde_json::Value;

    fn as_component(&self) -> &Component;
    fn as_component_mut(&mut self) -> &mut Component;

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

impl ComponentWrapper for Component {
    fn as_component(&self) -> &Component {
        self
    }

    fn as_component_mut(&mut self) -> &mut Component {
        self
    }

    fn as_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }

    fn get_info(&self) -> String {
        format!("Component")
    }
}

impl HasPropertyBuilder for Component {
    fn keys(_: PhantomData<Self>) -> Vec<String> {
        strings!["component_type", "start_time", "length", "layer_index"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "component_type" => AsAttribute::from_choose(ComponentType::types().iter().map(|v| format!("{:?}", v)).collect(), ComponentType::types().iter().position(|v| v == &self.component_type)),
            "start_time" => AsAttribute::from_time(self.start_time),
            "length" => AsAttribute::from_time(self.length),
            "layer_index" => AsAttribute::from_usize(self.layer_index),
            _ => unimplemented!(),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "component_type" => self.component_type = ComponentType::types()[prop.as_choose().unwrap()].clone(),
            "start_time" => self.start_time = prop.as_time().unwrap(),
            "length" => self.length = prop.as_time().unwrap(),
            "layer_index" => self.layer_index = prop.as_usize().unwrap(),
            _ => unimplemented!(),
        }
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable + HasProperty {}
impl<T: ComponentWrapper + Peekable + HasProperty> ComponentLike for T {}

impl Deref for ComponentLike {
    type Target = Component;

    fn deref(&self) -> &Component {
        self.as_component()
    }
}

impl DerefMut for ComponentLike {
    fn deref_mut(&mut self) -> &mut Component {
        self.as_component_mut()
    }
}

