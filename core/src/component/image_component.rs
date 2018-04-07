use std::marker::PhantomData;

extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate serde_json;

use component::property::*;
use component::attribute::*;
use component::component::*;

#[derive(Deserialize, Debug, Clone)]
struct ImageProperty {
    #[serde(default)]
    common: CommonProperty,

    entity: String,
}

impl ImageProperty {
    fn from_value(mut json: serde_json::Value) -> ImageProperty {
        let json_ = json.clone();
        json.as_object_mut().unwrap().insert("common".to_string(), json_);
        serde_json::from_value(json).unwrap()
    }
}

pub struct ImageComponent {
    component: Component,
    data: gdk_pixbuf::Pixbuf,
    prop: ImageProperty,
}

impl ImageComponent {
    pub fn new_from_json(json: serde_json::Value) -> ImageComponent {
        let prop = ImageProperty::from_value(json.as_object().unwrap()["prop"].clone());

        ImageComponent {
            component: serde_json::from_value(json).unwrap(),
            data: ImageComponent::create_data(&prop.entity),
            prop: prop,
        }
    }

    fn create_data(uri: &str) -> gdk_pixbuf::Pixbuf {
        gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap()
    }

    pub fn reload(&mut self, uri: &str) {
        self.data = gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap();
    }
}

impl Peekable for gdk_pixbuf::Pixbuf {
    fn get_duration(&self) -> gst::ClockTime {
        100 * gst::MSECOND
    }

    fn peek(&self, _: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        Some(self.clone())
    }
}

impl Peekable for ImageComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

impl ComponentWrapper for ImageComponent {
    fn as_component(&self) -> &Component {
        &self.component
    }

    fn as_component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn as_value(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self.as_component()).unwrap();
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
        let mut w = 0;
        let mut h = 0;
        format!("image {:?}", gdk_pixbuf::Pixbuf::get_file_info(&self.prop.entity, &mut w, &mut h).unwrap().get_description())
    }
}

impl HasPropertyBuilder for ImageComponent {
    fn keys(_: PhantomData<Self>) -> Vec<String> {
        vec_add!(CommonProperty::keys(PhantomData), strings!["entity"])
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "entity" => AsAttribute::from_filepath(self.prop.entity.clone()),
            _ => self.prop.common.getter(name),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "entity" => {
                let uri = prop.as_filepath().unwrap();
                self.reload(&uri);
                self.prop.entity = uri;
            },
            _ => self.prop.common.setter(name, prop),
        }
    }
}

