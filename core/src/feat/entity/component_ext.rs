extern crate gstreamer as gst;
use spec::{Component, HaveComponent};

#[derive(Clone)]
pub enum ComponentExt {
    Video(Component),
}

impl HaveComponent for ComponentExt {
    fn component(&self) -> &Component {
        use ComponentExt::*;

        match self {
            Video(c) => c,
        }
    }

    fn component_mut(&mut self) -> &mut Component {
        use ComponentExt::*;

        match self {
            Video(c) => c,
        }
    }
}

impl ComponentExt {
}

