use std::collections::HashMap;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;

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

    pub coordinate: (i32,i32),

    pub layer_index: usize,

    pub entity: String,
}

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

#[derive(Debug, Clone)]
pub enum Property {
    I32(i32),
    Usize(usize),
    Time(gst::ClockTime),
    Pair(Box<Property>, Box<Property>),
    FilePath(String),
    Document(String),
    Font(String),
    Color(gdk::RGBA),
    ReadOnly(String),
}

impl Property {
    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Property::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }
}

pub type Properties = HashMap<String, Property>;

pub trait ComponentWrapper {
    fn get_component(&self) -> Component;
    fn get_properties(&self) -> Properties;
    fn set_property(&mut self, name: String, prop: Property);
}

impl ComponentWrapper for Component {
    fn get_component(&self) -> Component {
        self.clone()
    }

    fn get_properties(&self) -> Properties {
        use Property::*;

        [
            ("component_type".to_string(), ReadOnly(format!("{:?}", self.component_type))),
            ("start_time".to_string(), Time(self.start_time)),
            ("length".to_string(), Time(self.length)),
            ("coordinate".to_string(), Pair(box I32(self.coordinate.0), box I32(self.coordinate.1))),
            ("entity".to_string(), ReadOnly(self.entity.clone())),
            ("layer_index".to_string(), Usize(self.layer_index)),
        ].iter().cloned().collect()
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("start_time", Time(v)) => self.start_time = v,
            ("length", Time(v)) => self.length = v,
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("layer_index", Usize(v)) => self.layer_index = v,
            _ => unimplemented!(),
        }
    }
}

impl<T: ComponentWrapper> ComponentWrapper for Box<T> {
    fn get_component(&self) -> Component {
        self.as_ref().get_component()
    }

    fn get_properties(&self) -> Properties {
        self.as_ref().get_properties()
    }

    fn set_property(&mut self, name: String, prop: Property) {
        self.as_mut().set_property(name, prop);
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable {}
impl<T: ComponentWrapper + Peekable> ComponentLike for T {}

