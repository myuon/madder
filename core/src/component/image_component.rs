use std::marker::PhantomData;

extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate serde;
extern crate serde_json;

use serde::*;
use component::property::*;
use component::attribute::*;
use component::interface::*;

pub struct ImageComponent {
    component: ComponentProperty,
    geometry: GeometryProperty,
    entity: String,
    data: gdk_pixbuf::Pixbuf,
}

impl Serialize for ImageComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        map.extend(serde_json::to_value(self.component.clone()).unwrap().as_object().unwrap().clone());
        map.extend(serde_json::to_value(self.geometry.clone()).unwrap().as_object().unwrap().clone());
        map.extend(vec![("entity".to_string(), json!(self.entity))]);

        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ImageComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<ImageComponent, D::Error> {
        let json: serde_json::Value = Deserialize::deserialize(deserializer)?;

        Ok(ImageComponent::new_from_json(json))
    }
}

impl ImageComponent {
    pub fn new_from_json(json: serde_json::Value) -> ImageComponent {
        let entity = json.as_object().unwrap()["entity"].as_str().unwrap();

        ImageComponent {
            component: serde_json::from_value(json.clone()).unwrap(),
            geometry: serde_json::from_value(json.clone()).unwrap(),
            entity: entity.to_string(),
            data: ImageComponent::create_data(entity),
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

impl AsProperty for ImageComponent {
    fn as_component(&self) -> &ComponentProperty {
        &self.component
    }

    fn as_component_mut(&mut self) -> &mut ComponentProperty {
        &mut self.component
    }

    fn as_geometry(&self) -> Option<&GeometryProperty> {
        Some(&self.geometry)
    }

    fn as_geometry_mut(&mut self) -> Option<&mut GeometryProperty> {
        Some(&mut self.geometry)
    }
}

impl HasPropertyBuilder for ImageComponent {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec_add!(ComponentProperty::keys(PhantomData), vec_add!(GeometryProperty::keys(PhantomData), vec!["entity"]))
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "entity" => AsAttribute::from_filepath(self.entity.clone()),
            k if ComponentProperty::keys(PhantomData).contains(&k) => self.component.getter(k),
            k if GeometryProperty::keys(PhantomData).contains(&k) => self.geometry.getter(k),
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
            k if GeometryProperty::keys(PhantomData).contains(&k) => self.geometry.setter(k, prop),
            _ => unimplemented!(),
        }
    }
}

