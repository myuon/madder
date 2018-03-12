extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;
use self::serde::ser::SerializeTuple;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
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
    Choose(Vec<String>,Option<usize>),
}

pub fn gdk_rgba_serialize<S: serde::Serializer>(self_: &gdk::RGBA, serializer: S) -> Result<S::Ok, S::Error> {
    let mut tuple = serializer.serialize_tuple(4)?;
    tuple.serialize_element(&self_.red)?;
    tuple.serialize_element(&self_.green)?;
    tuple.serialize_element(&self_.blue)?;
    tuple.serialize_element(&self_.alpha)?;
    tuple.end()
}

pub fn gdk_rgba_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gdk::RGBA, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(|(x,y,z,w)| gdk::RGBA {
        red: x,
        green: y,
        blue: z,
        alpha: w,
    })
}

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

impl Property {
    pub fn as_readonly(&self) -> Option<String> {
        use Property::*;

        match self {
            &ReadOnly(ref t) => Some(t.to_string()),
            _ => None,
        }
    }

    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Property::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        use Property::*;

        match self {
            &Usize(t) => Some(t),
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

    pub fn as_choose(&self) -> Option<usize> {
        use Property::*;

        match self {
            &Choose(_,Some(t)) => Some(t),
            _ => None,
        }
    }
}

pub type Properties = Vec<(String, Property)>;

pub trait HasProperty {
    fn get_prop(&self, name: &str) -> Property;
    fn get_props(&self) -> Properties;
    fn set_prop(&mut self, name: &str, prop: Property);

    fn _make_get_props(&self, keys: Vec<String>) -> Properties {
        keys.into_iter().map(|key| (key.to_string(), self.get_prop(&key))).collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonProperty {
    #[serde(default = "coordinate_default")]
    pub coordinate: (i32,i32),

    #[serde(default = "rotate_default")]
    pub rotate: f64,

    #[serde(default = "alpha_default")]
    pub alpha: i32,

    #[serde(default = "scale_default")]
    pub scale: (f64, f64),
}

impl CommonProperty {
    pub fn keys() -> Vec<String> {
        strings!["coordinate", "rotate", "alpha", "scale"]
    }
}

impl HasProperty for CommonProperty {
    fn get_prop(&self, name: &str) -> Property {
        use Property::*;

        match name {
            "coordinate" => Pair(box I32(self.coordinate.0), box I32(self.coordinate.1)),
            "rotate" => F64(self.rotate),
            "alpha" => I32(self.alpha),
            "scale" => Pair(box F64(self.scale.0), box F64(self.scale.1)),
            _ => unimplemented!(),
        }
    }

    fn get_props(&self) -> Properties {
        self._make_get_props(Self::keys())
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("rotate", F64(n)) => self.rotate = n,
            ("alpha", I32(n)) => self.alpha = n,
            ("scale", Pair(box F64(x), box F64(y))) => self.scale = (x,y),
            _ => unimplemented!(),
        }
    }
}

use std::default::Default;
impl Default for CommonProperty {
    fn default() -> CommonProperty {
        CommonProperty {
            coordinate: coordinate_default(),
            rotate: rotate_default(),
            alpha: alpha_default(),
            scale: scale_default(),
        }
    }
}

fn coordinate_default() -> (i32, i32) { (0,0) }
fn rotate_default() -> f64 { 0.0 }
fn alpha_default() -> i32 { 255 }
fn scale_default() -> (f64, f64) { (1.0,1.0) }



