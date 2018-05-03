use std::marker::PhantomData;

extern crate gdk_pixbuf;
extern crate glib;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate serde;
extern crate serde_json;

use serde::*;
use component::property::*;
use component::attribute::*;
use component::interface::*;

impl Peekable for gst::Element {
    fn get_duration(&self) -> gst::ClockTime {
        100 * 1000 * gst::MSECOND
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.seek_simple(gst::SeekFlags::FLUSH, time).ok().and_then(|_| {
            self.get_property("last-pixbuf").ok().and_then(|x| x.get::<gdk_pixbuf::Pixbuf>())
        })
    }
}

pub struct VideoTestComponent {
    component: ComponentProperty,
    data: gst::Element,
}

impl VideoTestComponent {
    pub fn new_from_structure(component: &ComponentProperty) -> VideoTestComponent {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        pipeline.add_many(&[&src, &pixbufsink]).unwrap();
        src.link(&pixbufsink).unwrap();

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        VideoTestComponent {
            component: component.clone(),
            data: pixbufsink,
        }
    }
}

impl Peekable for VideoTestComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

pub struct VideoFileComponent {
    component: ComponentProperty,
    geometry: GeometryProperty,
    entity: String,
    data: gst::Element,
}

impl Serialize for VideoFileComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serde_json::Map::new();
        map.extend(serde_json::to_value(self.component.clone()).unwrap().as_object().unwrap().clone());
        map.extend(serde_json::to_value(self.geometry.clone()).unwrap().as_object().unwrap().clone());
        map.extend(vec![("entity".to_string(), json!(self.entity))]);

        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VideoFileComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<VideoFileComponent, D::Error> {
        let json: serde_json::Value = Deserialize::deserialize(deserializer)?;

        Ok(VideoFileComponent::new_from_json(json))
    }
}

impl VideoFileComponent {
    pub fn new_from_json(json: serde_json::Value) -> VideoFileComponent {
        let entity = json.as_object().unwrap()["entity"].as_str().unwrap();

        VideoFileComponent {
            component: serde_json::from_value(json.clone()).unwrap(),
            geometry: serde_json::from_value(json.clone()).unwrap(),
            entity: entity.to_string(),
            data: VideoFileComponent::create_data(&entity),
        }
    }

    fn create_data(uri: &str) -> gst::Element {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let queue = gst::ElementFactory::make("queue", None).unwrap();
        let convert = gst::ElementFactory::make("videoconvert", None).unwrap();
        let pixbufsink = gst::ElementFactory::make("gdkpixbufsink", None).unwrap();

        src.set_property("location", &glib::Value::from(uri)).unwrap();

        pipeline.add_many(&[&src, &decodebin, &queue, &convert, &pixbufsink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();
        gst::Element::link_many(&[&queue, &convert, &pixbufsink]).unwrap();

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = queue.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        pipeline.set_state(gst::State::Paused).into_result().unwrap();

        pixbufsink
    }

    pub fn reload(&mut self, uri: &str) {
        self.data = VideoFileComponent::create_data(uri);
    }
}

impl Peekable for VideoFileComponent {
    fn get_duration(&self) -> gst::ClockTime {
        self.data.get_duration()
    }

    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf> {
        self.data.peek(time)
    }
}

impl AsProperty for VideoFileComponent {
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

/*
impl AsProperty for VideoFileComponent {
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
        format!("video\ndata_uri:\t{}\nclock:\t{:?}\n",
                self.prop.entity,
                self.data.get_clock()
        )
    }
}
*/

impl HasPropertyBuilder for VideoFileComponent {
    fn keys(_: PhantomData<Self>) -> Vec<&'static str> {
        vec_add!(ComponentProperty::keys(PhantomData), vec!["entity"])
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

