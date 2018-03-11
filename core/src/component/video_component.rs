extern crate gdk_pixbuf;
extern crate glib;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
extern crate gstreamer_app as gsta;
use gstv::prelude::*;

extern crate serde_json;

use component::property::*;
use component::component::*;

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
    component: Component,
    data: gst::Element,
}

impl VideoTestComponent {
    pub fn new_from_structure(component: &Component) -> VideoTestComponent {
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

impl AsRef<Component> for VideoTestComponent {
    fn as_ref(&self) -> &Component {
        &self.component
    }
}

impl AsMut<Component> for VideoTestComponent {
    fn as_mut(&mut self) -> &mut Component {
        &mut self.component
    }
}

#[derive(Deserialize, Debug, Clone)]
struct VideoFileProperty {
    #[serde(default)]
    common: CommonProperty,

    entity: String,
}

impl HasProperty for VideoFileProperty {
    fn get_props(&self) -> Properties {
        use Property::*;

        let mut props = self.common.get_props();
        props.push(("entity".to_string(), FilePath(self.entity.clone())));
        props
    }

    fn set_prop(&mut self, name: &str, prop: Property) {
        use Property::*;

        match (name, prop) {
            ("entity", FilePath(uri)) => unimplemented!(),
            (x,y) => {
                self.common.set_prop(x,y.clone());
            },
        }
    }
}

pub struct VideoFileComponent {
    component: Component,
    data: gst::Element,
    prop: VideoFileProperty,
}

impl AsRef<Component> for VideoFileComponent {
    fn as_ref(&self) -> &Component {
        &self.component
    }
}

impl AsMut<Component> for VideoFileComponent {
    fn as_mut(&mut self) -> &mut Component {
        &mut self.component
    }
}

impl VideoFileComponent {
    pub fn new_from_json(json: serde_json::Value) -> VideoFileComponent {
        let common = serde_json::from_value::<CommonProperty>(json.clone()).unwrap();
        let mut prop = serde_json::from_value::<VideoFileProperty>(json.as_object().unwrap()["prop"].clone()).unwrap();
        prop.common = common;

        VideoFileComponent {
            component: serde_json::from_value(json).unwrap(),
            data: VideoFileComponent::create_data(&prop.entity),
            prop: prop,
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

impl ComponentWrapper for VideoFileComponent {
    fn as_value(&self) -> serde_json::Value {
        let mut json = serde_json::to_value(self.as_ref()).unwrap();
        let props = {
            let mut props = serde_json::Map::new();
            for (k,v) in self.prop.get_props() {
                props.insert(k, serde_json::to_value(v).unwrap());
            }

            props
        };

        json.as_object_mut().unwrap().insert("property".to_string(), json!(props));
        json
    }

    fn get_info(&self) -> String {
        format!("video\ndata_uri:\t{}\nclock:\t{:?}\n",
                self.prop.entity,
                self.data.get_clock()
        )
    }
}

