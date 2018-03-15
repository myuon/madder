extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use std::marker::PhantomData;
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

impl HasPropertyBuilder for CommonProperty {
    fn keys(_: PhantomData<Self>) -> Vec<String> {
        strings!["coordinate", "rotate", "alpha", "scale"]
    }

    fn getter<T: AsAttribute>(&self, name: &str) -> T {
        match name {
            "coordinate" => {
                AsAttribute::from_pair(
                    box AsAttribute::from_i32(self.coordinate.0),
                    box AsAttribute::from_i32(self.coordinate.1)
                )
            },
            "rotate" => AsAttribute::from_f64(self.rotate),
            "alpha" => AsAttribute::from_i32(self.alpha),
            "scale" => {
                AsAttribute::from_pair(
                    box AsAttribute::from_f64(self.scale.0),
                    box AsAttribute::from_f64(self.scale.1)
                )
            },
            _ => unimplemented!(),
        }
    }

    fn setter<T: AsAttribute>(&mut self, name: &str, prop: T) {
        match name {
            "coordinate" => {
                let (x,y) = prop.as_pair().unwrap();
                self.coordinate = (x.as_i32().unwrap(), y.as_i32().unwrap());
            },
            "rotate" => self.rotate = prop.as_f64().unwrap(),
            "alpha" => self.alpha = prop.as_i32().unwrap(),
            "scale" => {
                let (x,y) = prop.as_pair().unwrap();
                self.scale = (x.as_f64().unwrap(), y.as_f64().unwrap());
            },
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



