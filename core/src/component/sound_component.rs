extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate serde_json;
extern crate glib;
use gst::prelude::*;

use component::property::*;
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

impl SoundComponent {
    fn keys() -> Vec<String> {
        strings!["entity"]
    }
}

impl HasProperty for SoundComponent {
    fn get_props(&self) -> Properties {
        self._make_get_props(Self::keys())
    }

    fn get_prop(&self, name: &str) -> Property {
        use Property::*;

        match name {
            "entity" => Document(self.prop.entity.clone()),
            _ => unimplemented!(),
        }
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("entity", FilePath(doc)) => {
                self.reload(&doc);
                self.prop.entity = doc;
            },
            _ => unimplemented!(),
        }
    }
}

