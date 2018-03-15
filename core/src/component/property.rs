extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use component::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommonProperty {
    #[serde(default = "coordinate_default")]
    pub coordinate: (i32,i32),

    #[serde(default = "rotate_default")]
    pub rotate: f64,

    #[serde(default = "alpha_default")]
    pub alpha: i32,

    #[serde(default = "scale_default")]
    pub scale: (f64, f64),
}

impl CommonProperty {
    pub fn keys() -> Vec<String> {
        strings!["coordinate", "rotate", "alpha", "scale"]
    }
}

impl HasProperty for CommonProperty {
    fn get_attr(&self, name: &str) -> Attribute {
        use Attribute::*;

        match name {
            "coordinate" => Pair(box I32(self.coordinate.0), box I32(self.coordinate.1)),
            "rotate" => F64(self.rotate),
            "alpha" => I32(self.alpha),
            "scale" => Pair(box F64(self.scale.0), box F64(self.scale.1)),
            _ => unimplemented!(),
        }
    }

    fn get_attrs(&self) -> Vec<(String, Attribute)> {
        CommonProperty::keys().into_iter().map(|s| (s.clone(), self.get_attr(&s))).collect()
    }

    fn set_attr(&mut self, name: &str, prop: Attribute) {
        use Attribute::*;

        match (name, prop) {
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("rotate", F64(n)) => self.rotate = n,
            ("alpha", I32(n)) => self.alpha = n,
            ("scale", Pair(box F64(x), box F64(y))) => self.scale = (x,y),
            _ => unimplemented!(),
        }
    }

    fn set_prop(&mut self, name: &str, prop: serde_json::Value) {
        match name {
            "coordinate" => self.coordinate = serde_json::from_value(prop).unwrap(),
            "rotate" => self.rotate = serde_json::from_value(prop).unwrap(),
            "alpha" => self.alpha = serde_json::from_value(prop).unwrap(),
            "scale" => self.scale = serde_json::from_value(prop).unwrap(),
            _ => unimplemented!(),
        }
    }
}

use std::default::Default;
impl Default for CommonProperty {
    fn default() -> CommonProperty {
        CommonProperty {
            coordinate: coordinate_default(),
            rotate: rotate_default(),
            alpha: alpha_default(),
            scale: scale_default(),
        }
    }
}

fn coordinate_default() -> (i32, i32) { (0,0) }
fn rotate_default() -> f64 { 0.0 }
fn alpha_default() -> i32 { 255 }
fn scale_default() -> (f64, f64) { (1.0,1.0) }



