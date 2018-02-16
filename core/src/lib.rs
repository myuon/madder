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

pub mod component;
pub use self::component::*;

#[derive(Serialize, Deserialize)]
pub struct EditorStructure {
    pub components: Box<[ComponentStructure]>,
    pub size: (i32, i32),
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
    pub elements: Vec<Box<Component>>,
    pub position: gst::ClockTime,
    pub width: i32,
    pub height: i32,
}

impl Editor {
    pub fn new(width: i32, height: i32) -> Editor {
        Editor {
            elements: vec![],
            position: 0 * gst::MSECOND,
            width: width,
            height: height,
        }
    }

    pub fn new_from_structure(structure: &EditorStructure) -> Editor {
        let mut editor = Editor::new(structure.size.0, structure.size.1);
        structure.components.iter().for_each(|item| {
            editor.register(Component::new_from_structure(item));
        });
        editor
    }

    pub fn register(&mut self, component: Component) -> usize {
        self.elements.push(Box::new(component));
        self.elements.len() - 1
    }

    pub fn seek_to(&mut self, time: gst::ClockTime) {
        self.position = time;
    }

    pub fn request_component_info(&self, index: usize) -> Vec<(&str, String)> {
        let component = &self.elements[index];

        vec![
            ("start_time", component.structure.start_time.mseconds().unwrap().to_string()),
            ("end_time", component.structure.end_time.mseconds().unwrap().to_string()),
            ("coordinate", format!("{:?}", component.structure.coordinate)),
        ]
    }

    pub fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(0, false, 8, self.width, self.height).unwrap() };

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        for elem in self.elements.iter().filter(|elem| { elem.structure.start_time <= self.position && self.position <= elem.structure.end_time }) {
            if let Some(dest) = elem.data.peek(self.position) {
                &dest.composite(
                    &pixbuf, elem.structure.coordinate.0, elem.structure.coordinate.1,
                    cmp::min(dest.get_width(), self.width - elem.structure.coordinate.0),
                    cmp::min(dest.get_height(), self.height - elem.structure.coordinate.1),
                    elem.structure.coordinate.0.into(), elem.structure.coordinate.1.into(), 1f64, 1f64, 0, 255);
            }
        }

        pixbuf
    }

    pub fn write(&mut self, uri: &str, frames: u64, delta: u64) {
        let avi_renderer = AviRenderer::new(uri, self.width as usize, self.height as usize);

        for i in 0..frames {
            if i % 10 == 0 {
                println!("{} / {}", i, frames);
            }
            &avi_renderer.render_step(&self.get_current_pixbuf(), i*delta*gst::MSECOND);
            self.seek_to(i*delta*gst::MSECOND);
        }

        avi_renderer.render_finish();
    }
}


