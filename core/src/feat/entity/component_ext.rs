extern crate gstreamer as gst;
use spec::{Component, HaveComponent};

#[derive(Clone)]
pub struct ComponentImpl {
    start_time: gst::ClockTime,
    length: gst::ClockTime,
    layer_index: usize,
}

impl Component for ComponentImpl {
    fn start_time(&self) -> gst::ClockTime {
        self.start_time
    }

    fn length(&self) -> gst::ClockTime {
        self.length
    }
}

#[derive(Clone)]
pub enum ComponentExt {
    Video(ComponentImpl),
}

impl HaveComponent for ComponentExt {
    type COMPONENT = ComponentImpl;

    fn component(&self) -> &Self::COMPONENT {
        use ComponentExt::*;

        match self {
            Video(c) => c
        }
    }

    fn component_mut(&mut self) -> &mut Self::COMPONENT {
        use ComponentExt::*;

        match self {
            Video(c) => c
        }
    }
}

impl Component for ComponentExt {
    fn start_time(&self) -> gst::ClockTime {
        self.component().start_time
    }

    fn length(&self) -> gst::ClockTime {
        self.component().length
    }
}

