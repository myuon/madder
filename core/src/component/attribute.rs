use std::collections::HashMap;

extern crate gstreamer as gst;
extern crate gdk;
extern crate serde;
extern crate serde_json;
extern crate madder_util as util;

use util::serde_impl::*;
use serde_json::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "attribute", content = "value")]
pub enum Attribute {
    I32(i32),
    F64(f64),
    Usize(usize),

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    Time(gst::ClockTime),

    Pair(Box<Attribute>, Box<Attribute>),
    FilePath(String),
    Document(String),
    Font(String),

    #[serde(serialize_with = "SerRGBA::serialize_rgba")]
    #[serde(deserialize_with = "SerRGBA::deserialize_rgba")]
    Color(gdk::RGBA),

    ReadOnly(String),
    Choose(Vec<String>,Option<usize>),
    Sequence(Vec<Attribute>),
    HashMap(HashMap<String, Attribute>),
}

pub trait AsAttribute {
    fn as_i32(self) -> Option<i32>;
    fn as_f64(self) -> Option<f64>;
    fn as_usize(self) -> Option<usize>;
    fn as_time(self) -> Option<gst::ClockTime>;
    fn as_pair(self) -> Option<(Box<Self>, Box<Self>)>;
    fn as_filepath(self) -> Option<String>;
    fn as_document(self) -> Option<String>;
    fn as_font(self) -> Option<String>;
    fn as_color(self) -> Option<gdk::RGBA>;
    fn as_readonly(self) -> Option<String>;
    fn as_choose(self) -> Option<usize>;
    fn as_sequence(self) -> Option<Vec<Self>> where Self: Sized;
    fn as_map(self) -> Option<HashMap<String, Self>> where Self: Sized;
    fn as_attribute(self) -> Option<Attribute>;

    fn from_i32(arg: i32) -> Self;
    fn from_f64(arg: f64) -> Self;
    fn from_usize(arg: usize) -> Self;
    fn from_time(arg: gst::ClockTime) -> Self;
    fn from_pair(arg: Box<Self>, arg2: Box<Self>) -> Self where Self: Sized;
    fn from_filepath(arg: String) -> Self;
    fn from_document(arg: String) -> Self;
    fn from_font(arg: String) -> Self;
    fn from_color(arg: gdk::RGBA) -> Self;
    fn from_readonly(arg: String) -> Self;
    fn from_choose(arg: Vec<String>, arg2: Option<usize>) -> Self;
    fn from_sequence(arg: Vec<Self>) -> Self where Self: Sized;
    fn from_map(arg: HashMap<String, Self>) -> Self where Self: Sized;
    fn from_attribute(attr: Attribute) -> Self;
}

use Attribute::*;

impl AsAttribute for Attribute {
    fn as_i32(self) -> Option<i32> {
        match self {
            I32(t) => Some(t),
            _ => None,
        }
    }

    fn as_f64(self) -> Option<f64> {
        match self {
            F64(t) => Some(t),
            _ => None,
        }
    }

    fn as_usize(self) -> Option<usize> {
        match self {
            Usize(t) => Some(t),
            _ => None,
        }
    }

    fn as_time(self) -> Option<gst::ClockTime> {
        match self {
            Time(t) => Some(t),
            _ => None,
        }
    }

    fn as_pair(self) -> Option<(Box<Attribute>, Box<Attribute>)> {
        match self {
            Pair(x,y) => Some((x,y)),
            _ => None,
        }
    }

    fn as_filepath(self) -> Option<String> {
        match self {
            FilePath(s) => Some(s),
            _ => None,
        }
    }

    fn as_document(self) -> Option<String> {
        match self {
            Document(d) => Some(d),
            _ => None,
        }
    }

    fn as_font(self) -> Option<String> {
        match self {
            Font(s) => Some(s),
            _ => None,
        }
    }

    fn as_color(self) -> Option<gdk::RGBA> {
        match self {
            Color(c) => Some(c),
            _ => None,
        }
    }

    fn as_readonly(self) -> Option<String> {
        match self {
            ReadOnly(t) => Some(t),
            _ => None,
        }
    }

    fn as_choose(self) -> Option<usize> {
        match self {
            Choose(_,Some(t)) => Some(t),
            _ => None,
        }
    }

    fn as_sequence(self) -> Option<Vec<Attribute>> {
        match self {
            Sequence(t) => Some(t),
            _ => None,
        }
    }

    fn as_map(self) -> Option<HashMap<String, Attribute>> {
        match self {
            HashMap(kv) => Some(kv),
            _ => None,
        }
    }

    fn as_attribute(self) -> Option<Attribute> { Some(self) }

    fn from_i32(arg: i32) -> Self { I32(arg) }
    fn from_f64(arg: f64) -> Self { F64(arg) }
    fn from_usize(arg: usize) -> Self { Usize(arg) }
    fn from_time(arg: gst::ClockTime) -> Self { Time(arg) }
    fn from_pair(arg: Box<Self>, arg2: Box<Self>) -> Self { Pair(arg, arg2) }
    fn from_filepath(arg: String) -> Self { FilePath(arg) }
    fn from_document(arg: String) -> Self { Document(arg) }
    fn from_font(arg: String) -> Self { Font(arg) }
    fn from_color(arg: gdk::RGBA) -> Self { Color(arg) }
    fn from_readonly(arg: String) -> Self { ReadOnly(arg) }
    fn from_choose(arg: Vec<String>, arg2: Option<usize>) -> Self { Choose(arg, arg2) }
    fn from_sequence(arg: Vec<Attribute>) -> Self { Sequence(arg) }
    fn from_map(arg: HashMap<String, Self>) -> Self { HashMap(arg) }
    fn from_attribute(attr: Attribute) -> Self { attr }
}


impl AsAttribute for Value {
    fn as_i32(self) -> Option<i32> {
        self.as_i64().map(|x| x as i32)
    }

    fn as_f64(self) -> Option<f64> {
        serde_json::Value::as_f64(&self)
    }

    fn as_usize(self) -> Option<usize> {
        self.as_u64().map(|x| x as usize)
    }

    fn as_time(self) -> Option<gst::ClockTime> {
        self.as_u64().map(|x| gst::ClockTime::from_mseconds(x))
    }

    fn as_pair(self) -> Option<(Box<Value>, Box<Value>)> {
        self.as_array().map(|ref v| (box v[0].clone(), box v[1].clone()))
    }

    fn as_filepath(self) -> Option<String> {
        self.as_str().map(|s| s.to_string())
    }

    fn as_document(self) -> Option<String> {
        self.as_str().map(|s| s.to_string())
    }

    fn as_font(self) -> Option<String> {
        self.as_str().map(|s| s.to_string())
    }

    fn as_color(self) -> Option<gdk::RGBA> {
        from_value::<SerRGBA>(self).ok().map(|x| x.0)
    }

    fn as_readonly(self) -> Option<String> {
        self.as_str().map(|s| s.to_string())
    }

    fn as_choose(self) -> Option<usize> {
        unimplemented!()
    }

    fn as_sequence(self) -> Option<Vec<Value>> {
        self.as_array().cloned()
    }

    fn as_map(self) -> Option<HashMap<String, serde_json::Value>> {
        self.as_object().cloned().map(|x| x.into_iter().collect())
    }

    fn as_attribute(self) -> Option<Attribute> { serde_json::from_value(self).ok() }

    fn from_i32(arg: i32) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_f64(arg: f64) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_usize(arg: usize) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_time(arg: gst::ClockTime) -> Self { serde_json::to_value(SerTime(arg)).unwrap() }
    fn from_pair(arg: Box<Self>, arg2: Box<Self>) -> Self { serde_json::to_value([arg, arg2]).unwrap() }
    fn from_filepath(arg: String) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_document(arg: String) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_font(arg: String) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_color(arg: gdk::RGBA) -> Self { serde_json::to_value(SerRGBA(arg)).unwrap() }
    fn from_readonly(arg: String) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_choose(arg: Vec<String>, i: Option<usize>) -> Self { serde_json::to_value(arg[i.unwrap()].clone()).unwrap() }
    fn from_sequence(arg: Vec<Value>) -> Self { serde_json::to_value(arg).unwrap() }
    fn from_map(arg: HashMap<String, serde_json::Value>) -> Self { json!(arg) }
    fn from_attribute(attr: Attribute) -> Self { json!(attr) }
}
