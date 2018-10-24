extern crate serde_json;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
use util::*;
use std::collections::HashMap;

// Component domain requires the following specifications:
// - represents an object in timeline (start_time, end_time, length)
// - play video and/or sound
// - has effects

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub length: gst::ClockTime,

    #[serde(default = "HashMap::new")]
    pub attributes: HashMap<String, serde_json::Value>,

    // key of effects
    // fix type as String might be a bad idea...
    #[serde(default = "Vec::new")]
    pub effect: Vec<String>,
}

impl Component {
    pub fn end_time(&self) -> gst::ClockTime {
        self.start_time + self.length
    }

    pub fn partial_update(&mut self, value: &serde_json::Map<String, serde_json::Value>) {
        for (k,v) in value {
            match k.as_str() {
                "start_time" => self.start_time = serde_json::from_value::<SerTime>(v.clone()).unwrap().0,
                "length" => self.length = serde_json::from_value::<SerTime>(v.clone()).unwrap().0,
                _ => unreachable!(),
            }
        }
    }
}

pub trait HaveComponent {
    fn component(&self) -> &Component;
    fn component_mut(&mut self) -> &mut Component;

    fn get_pixbuf(&self, gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        None
    }

    fn get_audio_elements(&self) -> Vec<gst::Element> {
        vec![]
    }

    fn tick(&self) -> Option<gdk_pixbuf::Pixbuf> {
        None
    }
}
