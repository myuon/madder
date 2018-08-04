extern crate serde_json;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate madder_util as util;
use util::serde_impl::*;
use std::collections::HashMap;

// Component domain requires the following specifications:
// - represents an object in timeline (start_time, end_time, length)
// - play video and/or sound
// - has effects

#[derive(Clone, Serialize, Deserialize)]
pub struct Component {
    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub length: gst::ClockTime,

    pub attributes: HashMap<String, serde_json::Value>,

    // key of effects
    // fix type as String might be a bad idea...
    pub effect: Vec<String>,
}

impl Component {
    pub fn end_time(&self) -> gst::ClockTime {
        self.start_time + self.length
    }
}

pub trait HaveComponent {
    fn component(&self) -> &Component;
    fn component_mut(&mut self) -> &mut Component;

    fn get_pixbuf(&self, gst::ClockTime) -> gdk_pixbuf::Pixbuf;
}
