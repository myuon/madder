mod component;
pub use self::component::*;

extern crate gstreamer as gst;

mod video_component;
pub use self::video_component::*;
mod image_component;
pub use self::image_component::*;
mod text_component;
pub use self::text_component::*;

impl Component {
    pub fn new_from_structure(structure: &ComponentStructure) -> Component {
        match structure.component_type {
            ComponentType::Video => VideoFileComponent::new_from_structure(structure).get_component(),
            ComponentType::Image => ImageComponent::new_from_structure(structure).get_component(),
            ComponentType::Text => TextComponent::new_from_structure(structure).get_component(),
            _ => unimplemented!(),
        }
    }
}

