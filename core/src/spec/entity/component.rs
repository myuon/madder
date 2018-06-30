extern crate gstreamer as gst;
use spec::*;

// Component domain requires the following specifications:
// - represents an object in timeline (start_time, end_time, length)
// - play video and/or sound
// - has effects

#[derive(Clone)]
pub struct Component {
    start_time: gst::ClockTime,
    length: gst::ClockTime,
    effect: Vec<Effect>,
}

impl Component {
    pub fn end_time(&self) -> gst::ClockTime {
        self.start_time + self.length
    }
}

pub trait HaveComponent {
    fn component(&self) -> &Component;
    fn component_mut(&mut self) -> &mut Component;
}

