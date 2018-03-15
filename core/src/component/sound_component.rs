extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate serde_json;
extern crate glib;
use gst::prelude::*;
use std::marker::PhantomData;

use component::attribute::*;
use component::component::*;

#[derive(Deserialize, Debug, Clone)]
struct SoundProperty {
    entity: String,
}

pub struct SoundComponent {
    component: Component,
    data: gst::Pipeline,
    prop: SoundProperty,
}

impl SoundComponent {
    pub fn new_from_json(json: serde_json::Value) -> SoundComponent {
        let prop = serde_json::from_value::<SoundProperty>(json.as_object().unwrap()["prop"].clone()).unwrap();

        SoundComponent {
            component: serde_json::from_value(json).unwrap(),
            data: SoundComponent::create_data(&prop.entity),
            prop: prop,
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
}

impl Peekable for SoundComponent {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        None
    }
}

impl AsRef<Component> for SoundComponent {
    fn as_ref(&self) -> &Component {
        &self.component
    }
}

impl AsMut<Component> for SoundComponent {
    fn as_mut(&mut self) -> &mut Component {
        &mut self.component
    }
}

impl ComponentWrapper for SoundComponent {
    fn as_value(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self.as_ref()).unwrap();
        let props = {
            let mut props = serde_json::Map::new();
            for (k,v) in self.get_props() {
                props.insert(k, serde_json::to_value(v).unwrap());
            }

            props
        };

        json.as_object_mut().unwrap().insert("prop".to_string(), json!(props));
        json
    }

    fn get_info(&self) -> String {
        format!("sound")
    }

    fn get_audio_pipeline(&self) -> Option<&gst::Pipeline> {
        Some(&self.data)
    }
}

impl HasPropertyBuilder for SoundComponent {
    fn keys(_: PhantomData<Self>) -> Vec<String> {
        strings!["entity"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "entity" => AsAttribute::from_filepath(self.prop.entity.clone()),
            _ => unimplemented!(),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "entity" => {
                let uri = prop.as_filepath().unwrap();
                self.reload(&uri);
                self.prop.entity = uri;
            },
            _ => unimplemented!(),
        }
    }
}

