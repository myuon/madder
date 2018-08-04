extern crate gdk_pixbuf;
extern crate gstreamer as gst;

use spec::*;

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

pub struct Project {
    layers: Vec<Layer>,
    pub size: (i32, i32),
    pub length: gst::ClockTime,
}

impl Project {
    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> Project {
        Project {
            layers: vec![Layer::new()],
            size: (width, height),
            length: length
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

