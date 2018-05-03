extern crate serde;
extern crate serde_json;
extern crate gstreamer as gst;
extern crate gdk_pixbuf;

use serde::*;
use component::attribute::*;
use component::interface::*;
use component::property::*;
use component::video_component::*;
use component::image_component::*;
use component::text_component::*;
use component::sound_component::*;

pub enum Component {
    Video(VideoFileComponent),
    Image(ImageComponent),
    Text(TextComponent),
    Sound(SoundComponent),
}

macro_rules! component_repeat {
    ($e:expr, ($i:ident) => $b:block) => {{
        use Component::*;

        match $e {
            Video($i) => $b,
            Image($i) => $b,
            Text($i) => $b,
            Sound($i) => $b,
        }
    }};
}

impl Serialize for Component {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use Component::*;

        let mut map = serde_json::Map::new();
        map.insert("component_type".to_string(), json!(match self {
            Video(_) => "Video",
            Image(_) => "Image",
            Text(_) => "Text",
            Sound(_) => "Sound",
        }));

        component_repeat!(self, (c) => {
            map.extend(serde_json::to_value(c).unwrap().as_object().unwrap().clone())
        });

        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Component, D::Error> {
        let json: serde_json::Value = Deserialize::deserialize(deserializer)?;

        Ok(Component::new_from_json(json))
    }
}

impl Component {
    pub fn get_info(&self) -> String {
        "info".to_string()
    }
}

impl AsProperty for Component {
    fn as_component(&self) -> &ComponentProperty {
        component_repeat!(self, (c) => {
            c.as_component()
        })
    }

    fn as_component_mut(&mut self) -> &mut ComponentProperty {
        component_repeat!(self, (c) => {
            c.as_component_mut()
        })
    }

    fn as_geometry(&self) -> Option<&GeometryProperty> {
        component_repeat!(self, (c) => {
            c.as_geometry()
        })
    }

    fn as_geometry_mut(&mut self) -> Option<&mut GeometryProperty> {
        component_repeat!(self, (c) => {
            c.as_geometry_mut()
        })
    }
}

impl Peekable for Component {
    fn get_duration(&self) -> gst::ClockTime {
        unimplemented!()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        use Component::*;

        match self {
            Video(c) => c.peek(time),
            Image(c) => c.peek(time),
            Text(c) => c.peek(time),
            Sound(c) => c.peek(time),
        }
    }
}

impl HasProperty for Component {
    fn get_attrs(&self) -> Vec<(String, Attribute)> {
        component_repeat!(self, (c) => {
            c.get_attrs()
        })
    }

    fn get_attr(&self, name: &str) -> Attribute {
        component_repeat!(self, (c) => {
            c.get_attr(name)
        })
    }

    fn set_attr(&mut self, name: &str, attr: Attribute) {
        component_repeat!(self, (c) => {
            c.set_attr(name, attr)
        })
    }

    fn get_props(&self) -> Vec<(String, serde_json::Value)> {
        component_repeat!(self, (c) => {
            c.get_props()
        })
    }

    fn get_prop(&self, name: &str) -> serde_json::Value {
        component_repeat!(self, (c) => {
            c.get_prop(name)
        })
    }

    fn set_prop(&mut self, name: &str, prop: serde_json::Value) {
        component_repeat!(self, (c) => {
            c.set_prop(name, prop)
        })
    }
}

