extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate glib;
extern crate serde;
extern crate serde_json;

use std::marker::PhantomData;
use gst::prelude::*;
use serde::*;
use component::property::*;
use component::attribute::*;
use component::interface::*;

pub struct SoundComponent {
    component: ComponentProperty,
    entity: String,
    data: gst::Pipeline,
}

impl Serialize for SoundComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        map.extend(serde_json::to_value(self.component.clone()).unwrap().as_object().unwrap().clone());
        map.extend(vec![("entity".to_string(), json!(self.entity))]);

        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SoundComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<SoundComponent, D::Error> {
        let json: serde_json::Value = Deserialize::deserialize(deserializer)?;

        Ok(SoundComponent::new_from_json(json))
    }
}

impl SoundComponent {
    pub fn new_from_json(json: serde_json::Value) -> SoundComponent {
        let entity = json.as_object().unwrap()["entity"].as_str().unwrap();

        SoundComponent {
            component: serde_json::from_value(json.clone()).unwrap(),
            entity: entity.to_string(),
            data: SoundComponent::create_data(entity),
        }
    }

    fn create_data(uri: &str) -> gst::Pipeline {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let convert = gst::ElementFactory::make("audioconvert", None).unwrap();
        let audiosink = gst::ElementFactory::make("autoaudiosink", None).unwrap();

        src.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &convert, &audiosink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&convert, &audiosink]).unwrap();

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = convert.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        pipeline
    }

    pub fn reload(&mut self, uri: &str) {
        self.data = SoundComponent::create_data(uri);
    }

    pub fn get_audio_pipeline(&self) -> Option<&gst::Pipeline> {
        Some(&self.data)
    }
}

impl Peekable for SoundComponent {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        None
    }
}

impl AsProperty for SoundComponent {
    fn as_component(&self) -> &ComponentProperty {
        &self.component
    }

    fn as_component_mut(&mut self) -> &mut ComponentProperty {
        &mut self.component
    }

    fn as_geometry(&self) -> Option<&GeometryProperty> {
        None
    }

    fn as_geometry_mut(&mut self) -> Option<&mut GeometryProperty> {
        None
    }
}

impl HasPropertyBuilder for SoundComponent {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec!["entity"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "entity" => AsAttribute::from_filepath(self.entity.clone()),
            k if ComponentProperty::keys(PhantomData).contains(&k) => self.component.getter(k),
            _ => unimplemented!(),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "entity" => {
                let uri = prop.as_filepath().unwrap();
                self.reload(&uri);
                self.entity = uri;
            },
            k if ComponentProperty::keys(PhantomData).contains(&k) => self.component.setter(k, prop),
            _ => unimplemented!(),
        }
    }
}

