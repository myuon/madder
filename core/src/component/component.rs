use std::ops::{Deref, DerefMut};

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use component::effect::*;
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

impl HasProperty for Effect {
    fn get_props(&self) -> Properties {
        unimplemented!()
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        unimplemented!()
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

    fn as_effect(&self) -> Vec<Properties> {
        self.as_ref().effect.iter().map(|eff| eff.get_props()).collect()
    }

    fn as_effect_mut(&mut self) -> &mut Vec<Properties> {
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

impl HasProperty for Component {
    fn get_props(&self) -> Properties {
        use Property::*;

        vec![
            ("component_type".to_string(), ReadOnly(format!("{:?}", self.component_type))),
            ("start_time".to_string(), Time(self.start_time)),
            ("length".to_string(), Time(self.length)),
            ("layer_index".to_string(), Usize(self.layer_index)),
        ]
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("start_time", Time(v)) => self.start_time = v,
            ("length", Time(v)) => self.length = v,
            ("layer_index", Usize(v)) => self.layer_index = v,
            _ => unimplemented!(),
        }
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

impl Effect {
    pub fn get_property(&self) -> Properties {
        use Property::*;

        [
            ("effect_type".to_string(), Choose(format!("EffectType"), EffectType::types().iter().position(|p| p == &self.effect_type).unwrap() as i32)),
            ("transition".to_string(), Choose(format!("Transition"), Transition::transitions().iter().position(|p| p == &self.transition).unwrap() as i32)),
            ("start_value".to_string(), F64(self.start_value)),
            ("end_value".to_string(), F64(self.end_value)),
        ].iter().cloned().collect()
    }

    pub fn set_property(&mut self, name: &str, prop: Property) {
        match name {
            "effect_type" => {
                self.effect_type = EffectType::types()[prop.as_choose().unwrap() as usize].clone();
            },
            "transition" => {
                self.transition = Transition::transitions()[prop.as_choose().unwrap() as usize].clone();
            },
            "start_value" => {
                self.start_value = prop.as_f64().unwrap();
            },
            "end_value" => {
                self.end_value = prop.as_f64().unwrap();
            },
            _ => unimplemented!(),
        }
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable {}
impl<T: ComponentWrapper + Peekable> ComponentLike for T {}

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

pub enum GdkInterpType {
    Nearest,
    Tiles,
    Bilinear,
    Hyper
}

impl GdkInterpType {
    pub fn to_i32(&self) -> i32 {
        use GdkInterpType::*;

        match self {
            &Nearest => 0,
            &Tiles => 1,
            &Bilinear => 2,
            &Hyper => 3,
        }
    }
}

