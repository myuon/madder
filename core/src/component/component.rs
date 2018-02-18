extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate gdk_pixbuf;

extern crate serde;

pub trait Peekable {
    fn get_duration(&self) -> gst::ClockTime;
    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>;

    // trick for Clone instance for Box<Peekable> object
    fn box_clone(&self) -> Box<Peekable>;
}

impl Clone for Box<Peekable> {
    fn clone(&self) -> Box<Peekable> {
        self.box_clone()
    }
}

impl Peekable for gst::Element {
    fn get_duration(&self) -> gst::ClockTime {
        100 * 1000 * gst::MSECOND
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.seek_simple(gst::SeekFlags::FLUSH, time).ok().and_then(|_| {
            self.get_property("last-pixbuf").ok().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
        })
    }

    fn box_clone(&self) -> Box<Peekable> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ComponentType {
    Video,
    Image,
    Text,
    Sound,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ComponentStructure {
    pub component_type: ComponentType,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub length: gst::ClockTime,

    pub coordinate: (i32,i32),

    pub entity: String,
}

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

#[derive(Debug, Clone)]
pub enum EditType {
    I32(i32),
    U64(u64),
    Pair(Box<EditType>, Box<EditType>),
    ReadOnly(String),
}

#[derive(Debug)]
pub struct Property {
    pub name: String,
    pub edit_type: EditType,
}

impl ComponentStructure {
    pub fn get_properties(&self) -> Vec<Property> {
        use EditType::*;

        vec![
            Property { name: "component_type".to_string(), edit_type: ReadOnly(format!("{:?}", self.component_type)) },
            Property { name: "start_time".to_string(), edit_type: U64(self.start_time.mseconds().unwrap()) },
            Property { name: "length".to_string(), edit_type: U64(self.start_time.mseconds().unwrap()) },
            Property { name: "coordinate".to_string(), edit_type: Pair(box I32(self.coordinate.0), box I32(self.coordinate.1)) },
            Property { name: "entity".to_string(), edit_type: ReadOnly(self.entity.clone()) },
        ]
    }

    pub fn set_property(&mut self, prop: Property) {
        use EditType::*;

        match (prop.name.as_str(), prop.edit_type) {
            ("start_time", U64(v)) => self.start_time = gst::ClockTime::from_mseconds(v),
            ("length", U64(v)) => self.length = gst::ClockTime::from_mseconds(v),
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub struct Component {
    pub structure: ComponentStructure,
    pub name: String,
    pub data: Box<Peekable>,
}

