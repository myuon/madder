extern crate gdk_pixbuf;
extern crate gstreamer as gst;
use util::*;
use spec::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    components: Vec<String>,
}

impl Layer {
    pub fn new() -> Layer {
        Layer {
            components: vec![],
        }
    }

    pub fn push(&mut self, component: String) {
        self.components.push(component);
    }

    pub fn list(&self) -> &Vec<String> {
        &self.components
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub layers: Vec<Layer>,

    #[serde(serialize_with = "SerIntPair::serialize_pair")]
    #[serde(deserialize_with = "SerIntPair::deserialize_pair")]
    pub size: (i32, i32),

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub length: gst::ClockTime,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub position: gst::ClockTime,
}

impl Project {
    pub fn new(width: i32, height: i32, length: gst::ClockTime, position: gst::ClockTime) -> Project {
        Project {
            layers: vec![Layer::new()],
            size: (width, height),
            length: length,
            position: position,
        }
    }

    pub fn insert_layer(&mut self, index: usize) {
        self.layers.insert(index, Layer::new());
    }

    pub fn list_layers(&self) -> &Vec<Layer> {
        &self.layers
    }

    pub fn add_component_at(&mut self, layer_index: usize, component: String) {
        self.layers[layer_index].push(component);
    }

    pub fn get_components_at_layer(&self, layer_index: usize) -> &Vec<String> {
        self.layers[layer_index].list()
    }
}

pub trait HaveProject {
    type COMPONENT : HaveComponent;
    fn project(&self) -> &Project;
    fn project_mut(&mut self) -> &mut Project;
}

