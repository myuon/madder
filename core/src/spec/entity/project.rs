extern crate gstreamer as gst;
use spec::*;

pub struct Layer<COMPONENT: HaveComponent> {
    components: Vec<COMPONENT>,
}

impl<COMPONENT: HaveComponent> Layer<COMPONENT> {
    pub fn new() -> Layer<COMPONENT> {
        Layer {
            components: vec![],
        }
    }

    pub fn push(&mut self, component: COMPONENT) {
        self.components.push(component);
    }
}

pub struct Project<COMPONENT: HaveComponent> {
    layers: Vec<Layer<COMPONENT>>,
    size: (i32, i32),
    length: gst::ClockTime,
}

impl<COMPONENT: HaveComponent> Project<COMPONENT> {
    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> Project<COMPONENT> {
        Project {
            layers: vec![Layer::new()],
            size: (width, height),
            length: length
        }
    }

    pub fn insert_layer(&mut self, index: usize) {
        self.layers.insert(index, Layer::new());
    }

    pub fn add_component_at(&mut self, layer_index: usize, component: COMPONENT) {
        self.layers[layer_index].push(component);
    }
}


