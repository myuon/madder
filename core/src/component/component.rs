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

#[derive(Debug, Clone)]
pub struct Component {
    pub structure: ComponentStructure,
    pub name: String,
}

pub trait ComponentWrapper {
    fn get_component(&self) -> Component;
    fn get_properties(&self) -> Vec<Property>;
    fn set_property(&mut self, prop: Property);
}

impl ComponentWrapper for Component {
    fn get_component(&self) -> Component {
        self.clone()
    }

    fn get_properties(&self) -> Vec<Property> {
        use EditType::*;

        vec![
            Property { name: "component_type".to_string(), edit_type: ReadOnly(format!("{:?}", self.structure.component_type)) },
            Property { name: "start_time".to_string(), edit_type: U64(self.structure.start_time.mseconds().unwrap()) },
            Property { name: "length".to_string(), edit_type: U64(self.structure.start_time.mseconds().unwrap()) },
            Property { name: "coordinate".to_string(), edit_type: Pair(box I32(self.structure.coordinate.0), box I32(self.structure.coordinate.1)) },
            Property { name: "entity".to_string(), edit_type: ReadOnly(self.structure.entity.clone()) },
        ]
    }

    fn set_property(&mut self, prop: Property) {
        use EditType::*;

        match (prop.name.as_str(), prop.edit_type) {
            ("start_time", U64(v)) => self.structure.start_time = gst::ClockTime::from_mseconds(v),
            ("length", U64(v)) => self.structure.length = gst::ClockTime::from_mseconds(v),
            ("coordinate", Pair(box I32(x), box I32(y))) => self.structure.coordinate = (x,y),
            _ => unimplemented!(),
        }
    }
}

impl<T: ComponentWrapper> ComponentWrapper for Box<T> {
    fn get_component(&self) -> Component {
        self.as_ref().get_component()
    }

    fn get_properties(&self) -> Vec<Property> {
        self.as_ref().get_properties()
    }

    fn set_property(&mut self, prop: Property) {
        self.as_mut().set_property(prop);
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable {}
impl<T: ComponentWrapper + Peekable> ComponentLike for T {}

