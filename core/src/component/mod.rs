mod component;
pub use self::component::*;

extern crate gstreamer as gst;

use ::serializer::*;

mod video_component;
pub use self::video_component::*;
mod image_component;
pub use self::image_component::*;
mod text_component;
pub use self::text_component::*;

impl Component {
    pub fn new_from_structure(structure: &ComponentStructure) -> Component {
        match structure.component_type {
            ComponentType::Video => {
                let mut c = VideoFileComponent::new(structure.entity.as_str(), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            },
            ComponentType::Image => {
                let mut c = ImageComponent::new(structure.entity.as_str(), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            },
            ComponentType::Text => {
                let mut c = TextComponent::new(structure.entity.as_str(), (640,480), structure.start_time * gst::MSECOND, structure.coordinate).get_component();
                c.end_time = structure.end_time * gst::MSECOND;
                c
            }
            _ => unimplemented!(),
        }
    }
}

