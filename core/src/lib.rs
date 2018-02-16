use std::cmp;

extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;

extern crate gdk_pixbuf;
extern crate pango;

mod avi_renderer;
use avi_renderer::AviRenderer;

#[macro_use] extern crate serde_derive;

pub mod serializer;
use serializer::*;

pub mod component;
pub use self::component::*;

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
            ("start_time", component.start_time.mseconds().unwrap().to_string()),
            ("end_time", component.end_time.mseconds().unwrap().to_string()),
            ("coordinate", format!("{:?}", component.coordinate)),
        ]
    }

    pub fn get_current_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        let pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(0, false, 8, self.width, self.height).unwrap() };

        for p in unsafe { pixbuf.get_pixels().chunks_mut(3) } {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        }

        for elem in self.elements.iter().filter(|elem| { elem.start_time <= self.position && self.position <= elem.end_time }) {
            if let Some(dest) = elem.component.peek(self.position) {
                &dest.composite(
                    &pixbuf, elem.coordinate.0, elem.coordinate.1,
                    cmp::min(dest.get_width(), self.width - elem.coordinate.0),
                    cmp::min(dest.get_height(), self.height - elem.coordinate.1),
                    elem.coordinate.0.into(), elem.coordinate.1.into(), 1f64, 1f64, 0, 255);
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


