extern crate serde;
extern crate gdk;
use self::serde::ser::*;
use self::serde::de::*;

pub struct SerRGBA(pub gdk::RGBA);

impl Serialize for SerRGBA {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&self.0.red)?;
        tuple.serialize_element(&self.0.green)?;
        tuple.serialize_element(&self.0.blue)?;
        tuple.serialize_element(&self.0.alpha)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for SerRGBA {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SerRGBA, D::Error> {
        Deserialize::deserialize(deserializer).map(|(x,y,z,w)| SerRGBA(gdk::RGBA {
            red: x,
            green: y,
            blue: z,
            alpha: w,
        }))
    }
}

impl SerRGBA {
    pub fn serialize_rgba<S: Serializer>(rgba: &gdk::RGBA, serializer: S) -> Result<S::Ok, S::Error> {
        SerRGBA(*rgba).serialize(serializer)
    }

    pub fn deserialize_rgba<'de, D: Deserializer<'de>>(deserializer: D) -> Result<gdk::RGBA, D::Error> {
        Deserialize::deserialize(deserializer).map(|t: SerRGBA| t.0)
    }
}

