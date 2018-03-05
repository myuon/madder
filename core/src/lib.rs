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

mod avi_renderer;
use avi_renderer::AviRenderer;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
use serde_json::Value;

pub mod component;
pub use self::component::*;

pub mod json_patch;
pub use self::json_patch::*;

#[derive(Serialize, Deserialize)]
pub struct EditorStructure {
    pub components: Box<[Component]>,
    pub width: i32,
    pub height: i32,
}

impl EditorStructure {
    pub fn new_from_file(file: &str) -> EditorStructure {
        let file = File::open(file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        serde_json::from_str(&contents).unwrap()
    }
}

pub struct Editor {
    elements: Vec<Box<ComponentLike>>,
    position: gst::ClockTime,
    width: i32,
    height: i32,
    renderer: Option<AviRenderer>,
}

impl Editor {
    pub fn new(width: i32, height: i32) -> Editor {
        Editor {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
            renderer: None,
        }
    }

    pub fn new_from_structure(structure: &EditorStructure) -> Editor {
        let mut editor = Editor::new(structure.width, structure.height);
        structure.components.iter().for_each(|item| {
            editor.register(Component::new_from_structure(item));
        });
        editor
    }

    fn register(&mut self, component: Box<ComponentLike>) -> usize {
        self.elements.push(component);
        self.elements.len() - 1
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    pub fn request_component_property(&self, index: usize) -> Properties {
        self.elements[index].get_properties()
    }

    pub fn request_effect_property(&self, index: usize) -> Vec<Properties> {
        self.elements[index].get_effect_properties()
    }

    pub fn set_effect_property(&mut self, index: usize, i: usize, name: &str, prop: Property) {
        self.elements[index].effect[i].set_property(name, prop);
    }

    pub fn remove_effect_property(&mut self, index: usize, i: usize) {
        self.elements[index].as_mut().effect.remove(i);
    }

    pub fn set_component_property(&mut self, index: usize, name: &str, prop: Property) {
        self.elements[index].set_property(name, prop);
    }

    pub fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(0, false, 8, self.width, self.height).unwrap() };

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        let mut elems = self.elements.iter().filter(|&elem| {
            elem.as_ref().start_time <= self.position
            && self.position <= elem.start_time + elem.length
        }).collect::<Vec<_>>();
        elems.sort_by_key(|elem| elem.layer_index);

        for elem in elems.iter().rev() {
            if let Some(mut dest) = elem.peek(self.position) {
                let mut elem = elem.as_ref().as_ref().clone();
                dest = Effect::get_rotated_pixbuf(dest, elem.rotate);

                for eff in elem.effect.clone() {
                    let start_time = elem.start_time;
                    let length = elem.length;
                    let position = (self.position - start_time).mseconds().unwrap() as f64 / length.mseconds().unwrap() as f64;

                    elem = eff.effect_on_component(elem, position);
                    dest = eff.effect_on_pixbuf(dest, position);
                }

                &dest.composite(
                    &pixbuf, elem.coordinate.0, elem.coordinate.1,
                    cmp::min(dest.get_width(), self.width - elem.coordinate.0),
                    cmp::min(dest.get_height(), self.height - elem.coordinate.1),
                    elem.coordinate.0.into(), elem.coordinate.1.into(), elem.scale.0, elem.scale.1,
                    GdkInterpType::Nearest.to_i32(), elem.alpha);
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
    fn add_components(&mut self, value: Value) {
        self.register(Component::new_from_structure(&serde_json::from_value(value).unwrap()));
    }

    fn add_components_n(&mut self, index: IndexRange, value: Value) {
        use IndexRange::*;
        let component = Component::new_from_structure(&serde_json::from_value(value).unwrap());

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
    }

    fn add_components_n_effect(&mut self, index: IndexRange, value: Value) {
        let effect = serde_json::from_value::<Effect>(value).unwrap();
        self.elements.as_index_mut(index).effect.push(effect);
    }

    fn add_components_n_key(&mut self, n: IndexRange, key: &str, value: Value) {
        self.elements.as_index_mut(n).as_mut().set_property(key, serde_json::from_value(value).unwrap());
    }

    fn add_components_n_effect_n_key(&mut self, n: IndexRange, m: IndexRange, key: &str, value: Value) {
        self.elements.as_index_mut(n).effect.as_index_mut(m).set_property(key, serde_json::from_value(value).unwrap());
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
}

impl Patch for Editor {
    fn get_by_pointer(&self, path: Pointer) -> Value {
        match path.0.as_slice() {
            &[ref c] if c == "width" => json!(self.width),
            &[ref c] if c == "height" => json!(self.height),
            &[ref c] if c == "position" => json!(self.position.mseconds().unwrap()),
            &[ref c] if c == "components" => serde_json::to_value(self.elements.iter().map(|c| c.as_ref().as_ref().clone()).collect::<Vec<Component>>()).unwrap(),
            &[ref c, ref n] if c == "components" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_ref().as_ref()).unwrap(),
            &[ref c, ref n, ref e] if c == "components" && e == "effect" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).effect.clone()).unwrap(),
            &[ref c, ref n, ref e, ref m] if c == "components" && e == "effect" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).effect.as_index(IndexRange::from_str(m).unwrap())).unwrap(),
            &[ref c, ref n, ref e, ref m, ref key] if c == "components" && e == "effect" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).effect.as_index(IndexRange::from_str(m).unwrap())).unwrap().as_object().unwrap()[key].clone(),
            &[ref c, ref n, ref e] if c == "components" && e == "info" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).get_info()).unwrap(),
            &[ref c, ref n, ref key] if c == "components" => serde_json::to_value(self.elements.as_index(IndexRange::from_str(n).unwrap()).as_ref().as_ref()).unwrap().as_object().unwrap()[key].clone(),
            _ => unimplemented!(),
        }
    }

    fn patch_once(&mut self, op: Operation) -> Result<(), PatchError> {
        use Operation::*;

        match op {
            Add(path, v) => {
                match path.0.as_slice() {
                    &[] => panic!("add"),
                    &[ref c] if c == "width" => panic!("update_width"),
                    &[ref c] if c == "height" => panic!("update_height"),
                    &[ref c] if c == "components" => self.add_components(v),
                    &[ref c, ref n] if c == "components" => self.add_components_n(IndexRange::from_str(n).unwrap(),v),
                    &[ref c, ref n, ref e] if c == "components" && e == "effect" => self.add_components_n_effect(IndexRange::from_str(n).unwrap(), v),
                    &[ref c, ref n, ref e, ref m, ref key] if c == "components" && e == "effect" => self.add_components_n_effect_n_key(IndexRange::from_str(n).unwrap(), IndexRange::from_str(m).unwrap(), key.as_str(), v),
                    &[ref c, ref n, ref key] if c == "components" => self.add_components_n_key(IndexRange::from_str(n).unwrap(), key.as_str(), v),
                    _ => unimplemented!(),
                }
            },
            Remove(path) => {
                match path.0.as_slice() {
                    &[ref c, ref n] if c == "components" => self.remove_components_n(IndexRange::from_str(n).unwrap()),
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}


