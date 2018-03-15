extern crate gstreamer as gst;
extern crate gdk;
extern crate serde;
extern crate madder_util as util;

use util::serde_impl::*;

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
}

pub trait AsAttribute {
    fn as_i32(&self) -> Option<i32>;
    fn as_f64(&self) -> Option<f64>;
    fn as_usize(&self) -> Option<usize>;
//    fn as_time(&self) ->
    fn as_readonly(&self) -> Option<String>;
}

impl Attribute {
    pub fn as_readonly(&self) -> Option<String> {
        use Attribute::*;

        match self {
            &ReadOnly(ref t) => Some(t.to_string()),
            _ => None,
        }
    }

    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Attribute::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        use Attribute::*;

        match self {
            &Usize(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        use Attribute::*;

        match self {
            &I32(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        use Attribute::*;

        match self {
            &F64(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_choose(&self) -> Option<usize> {
        use Attribute::*;

        match self {
            &Choose(_,Some(t)) => Some(t),
            _ => None,
        }
    }
}

