extern crate gdk_pixbuf;
extern crate gstreamer as gst;

use std::cmp;
use gdk_pixbuf::prelude::*;
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

    pub fn list(&self) -> &Vec<COMPONENT> {
        &self.components
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

    pub fn get_components_at_layer(&self, layer_index: usize) -> &Vec<COMPONENT> {
        self.layers[layer_index].list()
    }

    pub fn get_pixbuf(&self, position: gst::ClockTime) -> gdk_pixbuf::Pixbuf {
        let pixbuf = gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, false, 8, self.size.0, self.size.1);

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        for layer in self.layers.iter().rev() {
            for component in layer.list().iter().filter(|component| {
                component.component().start_time <= position &&
                    position <= component.component().end_time()
            }) {
                let dest = component.get_pixbuf(position);
                let coordinate = (0,0);
                let scale = (1.0,1.0);
                let alpha = 255;

                &dest.composite(
                    &pixbuf, coordinate.0, coordinate.1,
                    cmp::min(dest.get_width(), self.size.0 - coordinate.0),
                    cmp::min(dest.get_height(), self.size.1 - coordinate.1),
                    coordinate.0.into(), coordinate.1.into(),
                    scale.0, scale.1,
                    gdk_pixbuf::InterpType::Nearest, alpha);
            }
        }

        pixbuf
    }
}

pub trait HaveProject {
    type COMPONENT : HaveComponent;
    fn project(&self) -> &Project<Self::COMPONENT>;
    fn project_mut(&mut self) -> &mut Project<Self::COMPONENT>;
}


