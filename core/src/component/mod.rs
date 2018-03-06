mod effect;
pub use self::effect::*;
mod component;
pub use self::component::*;

extern crate gstreamer as gst;
extern crate serde_json;

mod video_component;
pub use self::video_component::*;
mod image_component;
pub use self::image_component::*;
mod text_component;
pub use self::text_component::*;
mod sound_component;
pub use self::sound_component::*;

impl Component {
    pub fn new_from_json(json: serde_json::Value) -> Box<ComponentLike> {
        match json.as_object().unwrap()["component_type"].as_str().unwrap() {
            "Video" => Box::new(VideoFileComponent::new_from_json(json)),
            "Image" => Box::new(ImageComponent::new_from_json(json)),
            "Text" => Box::new(TextComponent::new_from_json(json)),
            "Sound" => Box::new(SoundComponent::new_from_json(json)),
            _ => unimplemented!(),
        }
    }
}

