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
        self.as_array().and_then(|ref v| from_value(v[1].clone()).ok())
    }
}
