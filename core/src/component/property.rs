extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;
extern crate madder_util as util;

use util::serde_impl::*;
use std::marker::PhantomData;
use component::*;

pub trait HasProperty {
    fn get_attrs(&self) -> Vec<(String, Attribute)>;
    fn get_attr(&self, name: &str) -> Attribute;
    fn set_attr(&mut self, name: &str, attr: Attribute);

    fn get_props(&self) -> Vec<(String, serde_json::Value)>;
    fn get_prop(&self, name: &str) -> serde_json::Value;
    fn set_prop(&mut self, name: &str, prop: serde_json::Value);
}

pub trait HasPropertyBuilder {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str>;
    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T);
    fn getter<T: AsAttribute>(&self, name: &str) -> T;
}

impl<P: HasPropertyBuilder> HasProperty for P {
    fn get_attrs(&self) -> Vec<(String, Attribute)> {
        <P as HasPropertyBuilder>::keys(PhantomData).into_iter().map(|key| (key.to_string(), self.getter(key))).collect()
    }

    fn get_attr(&self, name: &str) -> Attribute {
        self.getter(name)
    }

    fn set_attr(&mut self, name: &str, attr: Attribute) {
        self.setter(name, attr)
    }

    fn get_props(&self) -> Vec<(String, serde_json::Value)> {
        <P as HasPropertyBuilder>::keys(PhantomData).into_iter().map(|key| (key.to_string(), self.getter(key))).collect()
    }

    fn get_prop(&self, name: &str) -> serde_json::Value {
        self.getter(name)
    }

    fn set_prop(&mut self, name: &str, prop: serde_json::Value) {
        self.setter(name, prop)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentProperty {
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

impl HasPropertyBuilder for ComponentProperty {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec!["start_time", "length", "layer_index"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "start_time" => AsAttribute::from_time(self.start_time),
            "length" => AsAttribute::from_time(self.length),
            "layer_index" => AsAttribute::from_usize(self.layer_index),
            z => panic!("Has no such prop name: {}", z),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "start_time" => self.start_time = prop.as_time().unwrap(),
            "length" => self.length = prop.as_time().unwrap(),
            "layer_index" => self.layer_index = prop.as_usize().unwrap(),
            z => panic!("Has no such prop name: {}", z),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeometryProperty {
    #[serde(default = "coordinate_default")]
    pub coordinate: (i32,i32),

    #[serde(default = "rotate_default")]
    pub rotate: f64,

    #[serde(default = "alpha_default")]
    pub alpha: i32,

    #[serde(default = "scale_default")]
    pub scale: (f64, f64),
}

impl HasPropertyBuilder for GeometryProperty {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec!["coordinate", "rotate", "alpha", "scale"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "coordinate" => {
                AsAttribute::from_pair(
                    box AsAttribute::from_i32(self.coordinate.0),
                    box AsAttribute::from_i32(self.coordinate.1)
                )
            },
            "rotate" => AsAttribute::from_f64(self.rotate),
            "alpha" => AsAttribute::from_i32(self.alpha),
            "scale" => {
                AsAttribute::from_pair(
                    box AsAttribute::from_f64(self.scale.0),
                    box AsAttribute::from_f64(self.scale.1)
                )
            },
            z => panic!("Call getter `{}` but not found", z),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "coordinate" => {
                let (x,y) = prop.as_pair().unwrap();
                self.coordinate = (x.as_i32().unwrap(), y.as_i32().unwrap());
            },
            "rotate" => self.rotate = prop.as_f64().unwrap(),
            "alpha" => self.alpha = prop.as_i32().unwrap(),
            "scale" => {
                let (x,y) = prop.as_pair().unwrap();
                self.scale = (x.as_f64().unwrap(), y.as_f64().unwrap());
            },
            _ => unimplemented!(),
        }
    }
}

use std::default::Default;
impl Default for GeometryProperty {
    fn default() -> GeometryProperty {
        GeometryProperty {
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

