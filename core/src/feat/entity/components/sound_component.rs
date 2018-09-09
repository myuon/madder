extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate glib;
extern crate serde_json;
use std::rc::Rc;
use gst::prelude::*;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundComponent {
    #[serde(flatten)]
    component: Component,

    data_path: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Option::None")]
    data: Option<Rc<gst::Pipeline>>,
}

impl SoundComponent {
    pub fn new(json: serde_json::Value) -> SoundComponent {
        let mut comp: SoundComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn load(&mut self) {
        self.data = Some(Rc::new(SoundComponent::create_data(&self.data_path)));
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
}

impl HaveComponent for SoundComponent {
    fn component(&self) -> &Component {
        &self.component
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn get_audio_pipeline(&self) -> Option<Rc<gst::Pipeline>> {
        Some(self.data.as_ref().unwrap().clone())
    }
}

