extern crate serde;
extern crate gstreamer as gst;
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

pub struct SerTime(pub gst::ClockTime);

impl Serialize for SerTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.0.mseconds().unwrap())
    }
}

impl<'de> Deserialize<'de> for SerTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SerTime, D::Error> {
        Deserialize::deserialize(deserializer).map(|v: f64| SerTime(gst::ClockTime::from_mseconds(v as u64)))
    }
}

impl SerTime {
    pub fn serialize_time<S: Serializer>(time: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
        SerTime(*time).serialize(serializer)
    }

    pub fn deserialize_time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
        Deserialize::deserialize(deserializer).map(|t: SerTime| t.0)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerIntPair(f32,f32);

impl SerIntPair {
    pub fn serialize_pair<S: Serializer>(pair: &(i32,i32), serializer: S) -> Result<S::Ok, S::Error> {
        SerIntPair(pair.0 as f32, pair.1 as f32).serialize(serializer)
    }

    pub fn deserialize_pair<'de, D: Deserializer<'de>>(deserializer: D) -> Result<(i32,i32), D::Error> {
        Deserialize::deserialize(deserializer).map(|p: SerIntPair| (p.0 as i32, p.1 as i32))
    }
}


