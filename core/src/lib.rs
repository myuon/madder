#![feature(box_patterns)]
#![feature(box_syntax)]
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
extern crate serde_json;

#[macro_use] extern crate derive_builder;

pub mod component;
pub use self::component::*;

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
    pub elements: Vec<Box<ComponentLike>>,
    pub position: gst::ClockTime,
    pub width: i32,
    pub height: i32,
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

    pub fn register(&mut self, component: Box<ComponentLike>) -> usize {
        self.elements.push(component);
        self.elements.len() - 1
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    pub fn request_component_property(&self, index: usize) -> Properties {
        self.elements[index].get_properties()
    }

    pub fn set_component_property(&mut self, index: usize, name: String, prop: Property) {
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
            elem.get_component().start_time <= self.position
            && self.position <= elem.get_component().start_time + elem.get_component().length
        }).collect::<Vec<_>>();
        elems.sort_by_key(|elem| elem.get_component().layer_index);

        for elem in elems.iter().rev() {
            if let Some(mut dest) = elem.peek(self.position) {
                let mut elem = elem.get_component();
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


