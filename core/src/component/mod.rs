mod attribute;
pub use self::attribute::*;
mod property;
pub use self::property::*;
mod effect;
pub use self::effect::*;
mod interface;
pub use self::interface::*;

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
mod component;
pub use self::component::*;

impl Component {
    pub fn new_from_json(json: serde_json::Value) -> Component {
        use Component::*;

        match json.as_object().unwrap()["component_type"].as_str().unwrap() {
            "Video" => Video(VideoFileComponent::new_from_json(json)),
            "Image" => Image(ImageComponent::new_from_json(json)),
            "Text" => Text(TextComponent::new_from_json(json)),
            "Sound" => Sound(SoundComponent::new_from_json(json)),
            _ => unimplemented!(),
        }
    }
}

