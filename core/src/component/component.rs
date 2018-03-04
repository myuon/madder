use std::ops::{Deref, DerefMut};
use std::collections::HashMap;

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
use self::serde::ser::SerializeTuple;

use component::effect::*;

pub trait EffectOn {
    fn effect_on_component(&self, component: Component, current: f64) -> Component;
}

impl EffectOn for Effect {
    fn effect_on_component(&self, component: Component, current: f64) -> Component {
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    #[serde(default = "coordinate_default")]
    pub coordinate: (i32,i32),

    pub layer_index: usize,

    #[serde(default = "rotate_default")]
    pub rotate: f64,

    #[serde(default = "alpha_default")]
    pub alpha: i32,

    pub entity: String,

    #[serde(default = "scale_default")]
    pub scale: (f64, f64),

    #[serde(default = "Vec::new")]
    pub effect: Vec<Effect>,
}

fn coordinate_default() -> (i32, i32) { (0,0) }
fn rotate_default() -> f64 { 0.0 }
fn alpha_default() -> i32 { 255 }
fn scale_default() -> (f64, f64) { (1.0,1.0) }

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Property {
    I32(i32),
    F64(f64),
    Usize(usize),

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    Time(gst::ClockTime),

    Pair(Box<Property>, Box<Property>),
    FilePath(String),
    Document(String),
    Font(String),

    #[serde(serialize_with = "gdk_rgba_serialize")]
    #[serde(deserialize_with = "gdk_rgba_deserialize")]
    Color(gdk::RGBA),

    ReadOnly(String),
    Choose(String,i32),
    EffectInfo(EffectType, Transition, f64, f64),
}

fn gdk_rgba_serialize<S: serde::Serializer>(self_: &gdk::RGBA, serializer: S) -> Result<S::Ok, S::Error> {
    let mut tuple = serializer.serialize_tuple(4)?;
    tuple.serialize_element(&self_.red)?;
    tuple.serialize_element(&self_.green)?;
    tuple.serialize_element(&self_.blue)?;
    tuple.serialize_element(&self_.alpha)?;
    tuple.end()
}

fn gdk_rgba_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gdk::RGBA, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(|(x,y,z,w)| gdk::RGBA {
        red: x,
        green: y,
        blue: z,
        alpha: w,
    })
}

impl Property {
    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Property::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        use Property::*;

        match self {
            &I32(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        use Property::*;

        match self {
            &F64(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_choose(&self) -> Option<i32> {
        use Property::*;

        match self {
            &Choose(_,t) => Some(t),
            _ => None,
        }
    }

    pub fn as_effect(&self) -> Option<Effect> {
        use Property::*;

        match self {
            &EffectInfo(ref typ, ref trans, ref start, ref end) => Some(Effect {
                effect_type: typ.clone(),
                transition: trans.clone(),
                start_value: *start,
                end_value: *end,
            }),
            _ => None,
        }
    }
}

pub type Properties = HashMap<String, Property>;

pub trait ComponentWrapper : AsRef<Component> + AsMut<Component> {
    fn get_properties(&self) -> Properties {
        self.as_ref().get_properties()
    }

    fn set_property(&mut self, name: &str, prop: Property) {
        self.as_mut().set_property(name, prop);
    }

    fn get_effect_properties(&self) -> Vec<Property> {
        self.as_ref().get_effect_properties()
    }

    fn set_effect_property(&mut self, i: usize, prop: Property) {
        self.as_mut().set_effect_property(i, prop);
    }

    fn get_info(&self) -> String;
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
    fn get_properties(&self) -> Properties {
        use Property::*;

        [
            ("component_type".to_string(), ReadOnly(format!("{:?}", self.component_type))),
            ("start_time".to_string(), Time(self.start_time)),
            ("length".to_string(), Time(self.length)),
            ("coordinate".to_string(), Pair(box I32(self.coordinate.0), box I32(self.coordinate.1))),
            ("entity".to_string(), ReadOnly(self.entity.clone())),
            ("layer_index".to_string(), Usize(self.layer_index)),
            ("rotate".to_string(), F64(self.rotate)),
            ("alpha".to_string(), I32(self.alpha)),
            ("scale".to_string(), Pair(box F64(self.scale.0), box F64(self.scale.1))),
        ].iter().cloned().collect()
    }

    fn set_property(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("start_time", Time(v)) => self.start_time = v,
            ("length", Time(v)) => self.length = v,
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("layer_index", Usize(v)) => self.layer_index = v,
            ("rotate", F64(v)) => self.rotate = v,
            ("alpha", I32(v)) => self.alpha = v,
            ("scale", Pair(box F64(x), box F64(y))) => self.scale = (x,y),
            _ => unimplemented!(),
        }
    }

    fn get_effect_properties(&self) -> Vec<Property> {
        use Property::*;

        self.effect.iter().map(|eff| {
            EffectInfo(eff.effect_type.clone(), eff.transition.clone(), eff.start_value, eff.end_value)
        }).collect()
    }

    fn set_effect_property(&mut self, i: usize, prop: Property) {
        self.effect[i] = prop.as_effect().unwrap();
    }

    fn get_info(&self) -> String {
        format!("Component")
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

