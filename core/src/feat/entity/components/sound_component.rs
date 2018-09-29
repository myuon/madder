extern crate gstreamer as gst;
extern crate gdk_pixbuf;
extern crate glib;
extern crate serde_json;
use gst::prelude::*;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct SoundComponent {
    #[serde(flatten)]
    component: Component,

    data_path: String,

    #[serde(skip)]
    #[serde(deserialize_with = "Vec::new")]
    data: Vec<gst::Element>,
}

impl SoundComponent {
    pub fn new(json: serde_json::Value) -> SoundComponent {
        let mut comp: SoundComponent = serde_json::from_value(json).unwrap();
        comp.load();
        comp
    }

    fn load(&mut self) {
        self.data = SoundComponent::create_data(&self.data_path);
    }

    fn create_data(uri: &str) -> Vec<gst::Element> {
        // let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("filesrc", None).unwrap();
        let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();
        let convert = gst::ElementFactory::make("audioconvert", None).unwrap();

        src.set_property("location", &glib::Value::from(uri)).unwrap();

        // pipeline.add_many(&[&src, &decodebin, &convert, &audiosink]).unwrap();
        gst::Element::link_many(&[&src, &decodebin]).unwrap();

        let convert_ = convert.clone();
        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = convert_.get_static_pad("sink").unwrap();
            let _ = src_pad.link(&sink_pad);
        });

        // pipeline.set_state(gst::State::Paused).into_result().unwrap();

        vec![
            src,
            decodebin,
            convert,
        ]
    }
}

impl HaveComponent for SoundComponent {
    fn component(&self) -> &Component {
        &self.component
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.component
    }

    fn get_audio_elements(&self) -> Vec<gst::Element> {
        self.data.clone()
    }
}

