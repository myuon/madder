extern crate gstreamer as gst;

pub struct TimelineStructure {
    pub size: (i32, i32),
    pub components: Box<[ComponentStructure]>,
}

pub struct ComponentStructure {
    pub component_type: ComponentType,
    pub start_time: gst::ClockTime,
    pub end_time: gst::ClockTime,
    pub entity: String,
    pub coordinate: (i32, i32),
}

pub enum ComponentType {
    Video,
    Image,
    Text,
    Sound,
}

