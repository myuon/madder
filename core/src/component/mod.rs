mod effect;
pub use self::effect::*;
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
    pub fn new_from_structure(component: &Component) -> Box<ComponentLike> {
        match component.component_type {
            ComponentType::Video => Box::new(VideoFileComponent::new_from_structure(component)),
            ComponentType::Image => Box::new(ImageComponent::new_from_structure(component)),
            ComponentType::Text => Box::new(TextComponent::new_from_structure(component)),
            _ => unimplemented!(),
        }
    }
}

