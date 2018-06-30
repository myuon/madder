extern crate gstreamer as gst;

pub mod spec;
pub mod feat;

use spec::*;
use feat::*;

pub struct Madder {
    project: Project<ComponentExt>,
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project::new(640, 480, 100 * gst::MSECOND),
        }
    }
}

/*
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(slice_patterns)]
#![feature(macro_at_most_once_rep)]
use std::cmp;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;

extern crate gdk_pixbuf;
extern crate pango;
use gdk_pixbuf::prelude::*;

mod avi_renderer;
use avi_renderer::AviRenderer;

extern crate serde;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
use serde_json::Value;

#[macro_use] extern crate madder_util as util;
use util::serde_impl::*;

pub mod component;
pub use self::component::*;

pub mod json_patch;
pub use self::json_patch::*;

pub mod spec;

fn position_default() -> gst::ClockTime {
    gst::ClockTime::from_mseconds(0)
}

// API for JSON Patch
impl Editor {
    fn add_components(&mut self, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.register(Component::new_from_json(value));
            },
            _ => unimplemented!(),
        }
    }

    fn add_components_n(&mut self, index: IndexRange, value: Value, content_type: ContentType) {
        use IndexRange::*;
        let component = Component::new_from_json(value);

        match content_type {
            ContentType::Value => {
                match index {
                    Index(i) => {
                        self.components.insert(i, component);
                    },
                    ReverseIndex(i) => {
                        let n = self.components.len();
                        self.components.insert(n-i, component);
                    },
                    All => {
                        self.components.push(component);
                    },
                }
            },
            _ => unimplemented!(),
        }
    }

    fn add_components_key(&mut self, n: IndexRange, key: &str, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.components.as_index_mut(n).set_prop(key, serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                self.components.as_index_mut(n).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_key_n(&mut self, n: IndexRange, key: &str, i: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                let mut seq = self.components.as_index_mut(n.clone()).get_prop(key).as_array().unwrap().clone();
                *seq.as_index_mut(i) = serde_json::from_value(value).unwrap();

                self.components.as_index_mut(n).set_prop(key, json!(seq));
            },
            ContentType::Attribute => {
                self.components.as_index_mut(n).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_effect(&mut self, index: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                let effect = serde_json::from_value::<Effect>(value).unwrap();
                self.components.as_index_mut(index).as_component_mut().effect.push(effect);
            },
            _ => unimplemented!(),
        }
    }

    fn add_components_n_effect_n_key(&mut self, n: IndexRange, m: IndexRange, key: &str, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.components.as_index_mut(n).as_component_mut().effect.as_index_mut(m).set_prop(key, serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                self.components.as_index_mut(n).as_component_mut().effect.as_index_mut(m).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_effect_n_intermeds(&mut self, n: IndexRange, m: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.components.as_index_mut(n).as_component_mut().effect.as_index_mut(m).intermeds.push(serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                unimplemented!();
            },
        }
    }

    fn remove_components_n(&mut self, index: IndexRange) {
        use IndexRange::*;

        match index {
            Index(i) => {
                self.components.remove(i);
            },
            ReverseIndex(i) => {
                let n = self.components.len();
                self.components.remove(n-i);
            },
            All => {
                self.components.clear();
            },
        }
    }

    fn remove_components_n_effect_n(&mut self, index: IndexRange, index2: IndexRange) {
        use IndexRange::*;

        match index2 {
            Index(i) => {
                self.components.as_index_mut(index).as_component_mut().effect.remove(i);
            },
            ReverseIndex(i) => {
                let effect = &mut self.components.as_index_mut(index).as_component_mut().effect;
                let n = effect.len();
                effect.remove(n-i);
            },
            All => {
                self.components.as_index_mut(index).as_component_mut().effect.clear();
            },
        }
    }

    fn get_by_pointer_as_value(&self, path: Pointer) -> Value {
        match path.0.iter().map(|ref x| x.as_str()).collect::<Vec<&str>>().as_slice() {
            &[] => {
                json!(self)
            },
            &["width"] => {
                json!(self.width)
            },
            &["height"] => {
                json!(self.height)
            },
            &["length"] => {
                json!(self.length.mseconds().unwrap())
            },
            &["position"] => {
                json!(self.position.mseconds().unwrap())
            },
            &["components"] => {
                serde_json::to_value(self.components.iter().map(|c| serde_json::to_value(c).unwrap()).collect::<Vec<_>>()).unwrap()
            },
            &["components", ref n] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap())).unwrap()
            },
            &["components", ref n, "list"] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).get_props()).unwrap()
            },
            &["components", ref n, "effect"] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.clone()).unwrap()
            },
            &["components", ref n, "effect", ref m] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap())).unwrap()
            },
            &["components", ref n, "effect", ref m, "intermeds", "value", ref v] => {
                json!(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).value(v.parse().unwrap()))
            },
            &["components", ref n, "effect", ref m, key] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap())).unwrap().as_object().unwrap()[key].clone()
            },
            &["components", ref n, "info"] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).get_info()).unwrap()
            },
            &["components", ref n, key] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).get_prop(key)).unwrap()
            },
            z => panic!("Call get_by_pointer_as_value with unexisting path: {:?}", z),
        }
    }

    fn get_by_pointer_as_attr(&self, path: Pointer) -> Value {
        match path.0.iter().map(|ref x| x.as_str()).collect::<Vec<&str>>().as_slice() {
            &["components", ref n] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap())).unwrap()
            },
            &["components", ref n, "list"] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).get_attrs()).unwrap()
            },
            &["components", ref n, "effect"] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.iter().map(|x| x.get_attrs()).collect::<Vec<_>>()).unwrap()
            },
            &["components", ref n, "effect", ref m] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).get_attrs()).unwrap()
            },
            &["components", ref n, "effect", ref m, key] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).get_attr(key)).unwrap()
            },
            &["components", ref n, "common_and_prop"] => {
                let mut attrs = self.components.as_index(IndexRange::from_str(n).unwrap()).as_component().get_attrs();
                attrs.extend_from_slice(self.components.as_index(IndexRange::from_str(n).unwrap()).get_attrs().as_slice());
                serde_json::to_value(attrs).unwrap()
            },
            &["components", ref n, key] => {
                serde_json::to_value(self.components.as_index(IndexRange::from_str(n).unwrap()).get_attr(key)).unwrap()
            },
            z => panic!("Call get_by_pointer_as_attr with unexisting path: {:?}", z),
        }
    }

    pub fn get_value(&self, path: Pointer) -> Value {
        self.get_by_pointer(path, ContentType::Value)
    }

    pub fn get_attr(&self, path: Pointer) -> Value {
        self.get_by_pointer(path, ContentType::Attribute)
    }
}

#[derive(Clone)]
pub enum ContentType {
    Value,
    Attribute,
}

impl Patch for Editor {
    type ContentType = ContentType;

    fn get_by_pointer(&self, path: Pointer, content_type: ContentType) -> Value {
        match content_type {
            ContentType::Value => self.get_by_pointer_as_value(path),
            ContentType::Attribute => serde_json::to_value(self.get_by_pointer_as_attr(path)).unwrap(),
        }
    }

    fn patch_once(&mut self, op: Operation, content_type: ContentType) -> Result<(), PatchError> {
        use Operation::*;

        match op {
            Add(path, v) => {
                match path.0.iter().map(|ref x| x.as_str()).collect::<Vec<&str>>().as_slice() {
                    &[] => panic!("add"),
                    &["width"] => panic!("update_width"),
                    &["height"] => panic!("update_height"),
                    &["length"] => self.length = v.as_time().unwrap(),
                    &["components"] => self.add_components(v, content_type),
                    &["components", ref n] => self.add_components_n(IndexRange::from_str(n).unwrap(),v,content_type),
                    &["components", ref n, "effect"] => self.add_components_n_effect(IndexRange::from_str(n).unwrap(), v, content_type),
                    &["components", ref n, "effect", ref m, "intermeds"] => self.add_components_n_effect_n_intermeds(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap(), v, content_type),
                    &["components", ref n, "effect", ref m, key] => self.add_components_n_effect_n_key(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap(), key, v, content_type),
                    &["components", ref n, key] => self.add_components_key(IndexRange::from_str(n).unwrap(), key, v, content_type),
                    &["components", ref n, key, ref i] => self.add_components_key_n(IndexRange::from_str(n).unwrap(), key, IndexRange::from_str(i).unwrap(), v, content_type),
                    z => panic!("Path Not Found: {:?}", z),
                }
            },
            Remove(path) => {
                match path.0.iter().map(|ref x| x.as_str()).collect::<Vec<&str>>().as_slice() {
                    &["components", ref n] => self.remove_components_n(IndexRange::from_str(n).unwrap()),
                    &["components", ref n, "effect", ref m] => self.remove_components_n_effect_n(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap()),
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}


*/
