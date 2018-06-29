extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde_json;
extern crate madder_util as util;

use util::serde_impl::*;
use std::{ fs::File, io::BufReader };

pub trait Presenter {
    fn get(path: &str) -> Document;
    fn create(path: &str, object: Document);
    fn update(path: &str, object: Document);
    fn delete(path: &str);
}

pub trait EditorController {
    fn get_project_info() -> serde_json::Value;
    fn list_components() -> Vec<Component>;
    fn get_component(usize) -> Component;
    fn get_pixbuf() -> gdk_pixbuf::Pixbuf;
}

#[derive(Serialize, Deserialize)]
pub struct Editor {
    pub components: Vec<Component>,

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

impl Editor {
    pub fn new(width: i32, height: i32, length: gst::ClockTime) -> Editor {
        Editor {
            components: vec![],
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
        self.components.push(component);
        self.components.len() - 1
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

        let mut elems = self.components.iter().filter(|&elem| {
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


