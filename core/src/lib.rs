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

#[derive(Serialize, Deserialize)]
pub struct Editor {
    #[serde(rename = "components")]
    pub elements: Vec<Component>,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    #[serde(default = "position_default")]
    position: gst::ClockTime,

    pub width: i32,
    pub height: i32,

    #[serde(serialize_with = "SerTime::serialize_time")]
    #[serde(deserialize_with = "SerTime::deserialize_time")]
    pub length: gst::ClockTime,

    #[serde(skip)]
    renderer: Option<AviRenderer>,
}

fn position_default() -> gst::ClockTime {
    gst::ClockTime::from_mseconds(0)
}

impl Editor {
    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> Editor {
        Editor {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
            length: length,
            renderer: None,
        }
    }

    pub fn new_from_json(json: Value) -> Editor {
        serde_json::from_value(json).unwrap()
    }

    pub fn new_from_file(file: &str) -> Editor {
        let file = File::open(file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        serde_json::from_str(&contents).unwrap()
    }

    fn register(&mut self, component: Component) -> usize {
        self.elements.push(component);
        self.elements.len() - 1
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    pub fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = gdk_pixbuf::Pixbuf::new(gdk_pixbuf::Colorspace::Rgb, false, 8, self.width, self.height);

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        let mut elems = self.elements.iter().filter(|&elem| {
            elem.as_component().start_time <= self.position
            && self.position <= elem.as_component().start_time + elem.as_component().length
        }).collect::<Vec<_>>();
        elems.sort_by_key(|elem| elem.as_component().layer_index);

        for elem in elems.iter().rev() {
            if let Some(mut dest) = elem.peek(self.position) {
                let mut common_prop = elem.as_geometry().cloned().unwrap_or(::std::default::Default::default());
                let mut elem = elem.as_component().clone();
                dest = Effect::get_rotated_pixbuf(dest, common_prop.rotate);

                for eff in elem.effect.clone() {
                    let start_time = elem.start_time;
                    let length = elem.length;
                    let position = (self.position - start_time).mseconds().unwrap() as f64 / length.mseconds().unwrap() as f64;

                    common_prop = eff.effect_on_component(common_prop, position);
                    dest = eff.effect_on_pixbuf(dest, position);
                }

                &dest.composite(
                    &pixbuf, common_prop.coordinate.0, common_prop.coordinate.1,
                    cmp::min(dest.get_width(), self.width - common_prop.coordinate.0),
                    cmp::min(dest.get_height(), self.height - common_prop.coordinate.1),
                    common_prop.coordinate.0.into(), common_prop.coordinate.1.into(),
                    common_prop.scale.0, common_prop.scale.1,
                    gdk_pixbuf::InterpType::Nearest, common_prop.alpha);
            }
        }

        pixbuf
    }

    pub fn write_init(&mut self, uri: &str, frames: i32, delta: u64) {
        self.renderer = Some(AviRenderer::new(uri, self.width as usize, self.height as usize, frames, delta));
    }

    pub fn write_next(&mut self) -> (bool, f64) {
        let current = self.renderer.as_ref().unwrap().current.clone();
        let frames = self.renderer.as_ref().unwrap().frames.clone();
        let delta = self.renderer.as_ref().unwrap().delta.clone();

        self.seek_to(current as u64 * delta * gst::MSECOND);

        let pixbuf = self.get_current_pixbuf();
        if self.renderer.as_mut().unwrap().render_step(&pixbuf) {
            (true, current as f64 / frames as f64)
        } else {
            self.renderer.as_ref().unwrap().render_finish();
            (false, 1.0)
        }
    }
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
                        self.elements.insert(i, component);
                    },
                    ReverseIndex(i) => {
                        let n = self.elements.len();
                        self.elements.insert(n-i, component);
                    },
                    All => {
                        self.elements.push(component);
                    },
                }
            },
            _ => unimplemented!(),
        }
    }

    fn add_components_key(&mut self, n: IndexRange, key: &str, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.elements.as_index_mut(n).as_component_mut().set_prop(key, serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                self.elements.as_index_mut(n).as_component_mut().set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_key_n(&mut self, n: IndexRange, key: &str, i: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                let mut seq = self.elements.as_index_mut(n.clone()).get_prop(key).as_array().unwrap().clone();
                *seq.as_index_mut(i) = serde_json::from_value(value).unwrap();

                self.elements.as_index_mut(n).as_component_mut().set_prop(key, json!(seq));
            },
            ContentType::Attribute => {
                self.elements.as_index_mut(n).as_component_mut().set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_effect(&mut self, index: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                let effect = serde_json::from_value::<Effect>(value).unwrap();
                self.elements.as_index_mut(index).as_component_mut().effect.push(effect);
            },
            _ => unimplemented!(),
        }
    }

    fn add_components_n_prop_key(&mut self, n: IndexRange, key: &str, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.elements.as_index_mut(n).set_prop(key, serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                self.elements.as_index_mut(n).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_prop_key_n(&mut self, n: IndexRange, key: &str, i: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                let mut seq = self.elements.as_index_mut(n.clone()).get_prop(key).as_array().unwrap().clone();
                *seq.as_index_mut(i) = serde_json::from_value(value).unwrap();

                self.elements.as_index_mut(n).set_prop(key, json!(seq));
            },
            ContentType::Attribute => {
                self.elements.as_index_mut(n).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_effect_n_key(&mut self, n: IndexRange, m: IndexRange, key: &str, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.elements.as_index_mut(n).as_component_mut().effect.as_index_mut(m).set_prop(key, serde_json::from_value(value).unwrap());
            },
            ContentType::Attribute => {
                self.elements.as_index_mut(n).as_component_mut().effect.as_index_mut(m).set_attr(key, serde_json::from_value(value).unwrap());
            },
        }
    }

    fn add_components_n_effect_n_intermeds(&mut self, n: IndexRange, m: IndexRange, value: Value, content_type: ContentType) {
        match content_type {
            ContentType::Value => {
                self.elements.as_index_mut(n).as_component_mut().effect.as_index_mut(m).intermeds.push(serde_json::from_value(value).unwrap());
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
                self.elements.remove(i);
            },
            ReverseIndex(i) => {
                let n = self.elements.len();
                self.elements.remove(n-i);
            },
            All => {
                self.elements.clear();
            },
        }
    }

    fn remove_components_n_effect_n(&mut self, index: IndexRange, index2: IndexRange) {
        use IndexRange::*;

        match index2 {
            Index(i) => {
                self.elements.as_index_mut(index).as_component_mut().effect.remove(i);
            },
            ReverseIndex(i) => {
                let effect = &mut self.elements.as_index_mut(index).as_component_mut().effect;
                let n = effect.len();
                effect.remove(n-i);
            },
            All => {
                self.elements.as_index_mut(index).as_component_mut().effect.clear();
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
                serde_json::to_value(self.elements.iter().map(|c| serde_json::to_value(c).unwrap()).collect::<Vec<_>>()).unwrap()
            },
            &["components", ref n] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap())).unwrap()
            },
            &["components", ref n, "effect"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.clone()).unwrap()
            },
            &["components", ref n, "effect", ref m] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap())).unwrap()
            },
            &["components", ref n, "effect", ref m, "intermeds", "value", ref v] => {
                json!(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).value(v.parse().unwrap()))
            },
            &["components", ref n, "effect", ref m, key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap())).unwrap().as_object().unwrap()[key].clone()
            },
            &["components", ref n, "info"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_info()).unwrap()
            },
            &["components", ref n, "prop"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_props()).unwrap()
            },
            &["components", ref n, "prop", ref key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_prop(key)).unwrap()
            },
            &["components", ref n, "common", ref key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_prop(key)).unwrap()
            },
            &["components", ref n, key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_prop(key)).unwrap()
            },
            z => panic!("Call get_by_pointer_as_value with unexisting path: {:?}", z),
        }
    }

    fn get_by_pointer_as_attr(&self, path: Pointer) -> Value {
        match path.0.iter().map(|ref x| x.as_str()).collect::<Vec<&str>>().as_slice() {
            &["components", ref n] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_attrs()).unwrap()
            },
            &["components", ref n, "effect"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.iter().map(|x| x.get_props()).collect::<Vec<_>>()).unwrap()
            },
            &["components", ref n, "effect", ref m] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).get_attrs()).unwrap()
            },
            &["components", ref n, "effect", ref m, key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().effect.as_index(IndexRange::from_str(m).unwrap()).get_attr(key)).unwrap()
            },
            &["components", ref n, "prop"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_attrs()).unwrap()
            },
            &["components", ref n, "prop", ref key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_attr(key)).unwrap()
            },
            &["components", ref n, "common"] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_attrs()).unwrap()
            },
            &["components", ref n, "common_and_prop"] => {
                let mut attrs = self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_attrs();
                attrs.extend_from_slice(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_attrs().as_slice());
                serde_json::to_value(attrs).unwrap()
            },
            &["components", ref n, key] => {
                serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_component().get_attr(key)).unwrap()
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
                    &["components"] => self.add_components(v, content_type),
                    &["components", ref n] => self.add_components_n(IndexRange::from_str(n).unwrap(),v,content_type),
                    &["components", ref n, "effect"] => self.add_components_n_effect(IndexRange::from_str(n).unwrap(), v, content_type),
                    &["components", ref n, "effect", ref m, "intermeds"] => self.add_components_n_effect_n_intermeds(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap(), v, content_type),
                    &["components", ref n, "effect", ref m, key] => self.add_components_n_effect_n_key(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap(), key, v, content_type),
                    &["components", ref n, "prop", key] => self.add_components_n_prop_key(IndexRange::from_str(n).unwrap(), key, v, content_type),
                    &["components", ref n, "prop", key, ref i] => self.add_components_n_prop_key_n(IndexRange::from_str(n).unwrap(), key, IndexRange::from_str(i).unwrap(), v, content_type),
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


