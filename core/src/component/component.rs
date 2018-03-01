use std::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::f64::consts::PI;

extern crate gdk;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate serde;

pub trait Peekable {
    fn get_duration(&self) -> gst::ClockTime;
    fn peek(&self, time: gst::ClockTime) -> Option<gdk_pixbuf::Pixbuf>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ComponentType {
    Video,
    Image,
    Text,
    Sound,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EffectType {
    CoordinateX,
    CoordinateY,
    Rotate,
    ScaleX,
    ScaleY,
    Alpha,
}

impl EffectType {
    pub fn types() -> Vec<EffectType> {
        use EffectType::*;

        vec![
            CoordinateX,
            CoordinateY,
            Rotate,
            ScaleX,
            ScaleY,
            Alpha
        ]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Transition {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Transition {
    pub fn transitions() -> Vec<Transition> {
        use Transition::*;

        vec![
            Linear,
            Ease,
            EaseIn,
            EaseOut,
            EaseInOut,
        ]
    }

    fn get_in_interval(&self, x: f64) -> f64 {
        use Transition::*;

        match self {
            &Linear => x,
            &Ease => Transition::cubic_bezier(0.25, 0.1, 0.25, 1.0, x),
            &EaseIn => Transition::cubic_bezier(0.42, 0.0, 1.0, 1.0, x),
            &EaseOut => Transition::cubic_bezier(0.0, 0.0, 0.58, 1.0, x),
            &EaseInOut => Transition::cubic_bezier(0.42, 0.0, 0.58, 1.0, x),
        }
    }

    fn cubic_bezier(p0: f64, p1: f64, p2: f64, p3: f64, x: f64) -> f64 {
        // cubic bezier calculation by Newton method
        //
        // x = (3 P2.x - 3 P3.x + 1) t^3 + (-6 P2.x + 3 P3.x) t^2 + (3 P2.x) t
        // y = (3 P2.y - 3 P3.y + 1) t^3 + (-6 P2.y + 3 P3.y) t^2 + (3 P2.y) t
        // (0 <= t <= 1)
        //
        // x' = 3 (3 P2.x - 3 P3.x + 1) t^2 + 2 (-6 P2.x + 3 P3.x) t + 3 P2.x
        const MAX_ITERATION: i32 = 50;
        const NEIGHBOR: f64 = 0.01;

        fn _bezier_params(u: f64, v: f64) -> (f64, f64, f64) {
            let k3 = 3.0 * u - 3.0 * v + 1.0;
            let k2 = -6.0 * u + 3.0 * v;
            let k1 = 3.0 * u;

            (k1,k2,k3)
        }

        fn bezier(u: f64, v: f64, t: f64) -> f64 {
            let (k1,k2,k3) = _bezier_params(u,v);
            (((k3 * t + k2) * t) + k1) * t
        }

        fn bezier_dt(u: f64, v: f64, t: f64) -> f64 {
            let (k1,k2,k3) = _bezier_params(u,v);
            ((3.0 * k3 * t + 2.0 * k2) * t) + k1
        }

        let bezier_x = |t: f64| { bezier(p0, p2, t) };
        let bezier_dt_x = |t: f64| { bezier_dt(p0, p2, t) };
        let bezier_y = |t: f64| { bezier(p1, p3, t) };

        let get_t_at_x = |x: f64| {
            let mut t = x;
            let mut new_t = x;

            for _ in 0..MAX_ITERATION {
                let f_t = bezier_x(t) - x;
                let fp_t = bezier_dt_x(t);
                new_t = t - (f_t / fp_t);
                if (new_t - t).abs() < NEIGHBOR {
                    break;
                }

                t = new_t;
            }

            new_t
        };

        bezier_y(get_t_at_x(x))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Effect {
    pub effect_type: EffectType,
    pub transition: Transition,
    pub start_value: f64,
    pub end_value: f64,
}

impl Effect {
    pub fn effect_on_component(&self, component: Component, current: f64) -> Component {
        use EffectType::*;

        match self.effect_type {
            CoordinateX => {
                let mut comp = component;
                comp.coordinate.0 += self.value(current) as i32;
                comp
            },
            CoordinateY => {
                let mut comp = component;
                comp.coordinate.1 += self.value(current) as i32;
                comp
            },
            ScaleX => {
                let mut comp = component;
                comp.scale.0 *= self.value(current);
                comp
            },
            ScaleY => {
                let mut comp = component;
                comp.scale.1 *= self.value(current);
                comp
            },
            Alpha => {
                let mut comp = component;
                comp.alpha = (comp.alpha as f64 * self.value(current) / 255.0) as i32;
                comp
            },
            _ => component,
        }
    }

    pub fn rotate(arg: f64, x: i32, y: i32) -> (i32, i32) {
        ((x as f64 * arg.cos() + y as f64 * arg.sin()) as i32,
         (x as f64 * -arg.sin() + y as f64 * arg.cos()) as i32,
        )
    }

    pub fn get_pixel(pixbuf: &gdk_pixbuf::Pixbuf, x: i32, y: i32) -> (u8,u8,u8,u8) {
        let pos = (y * pixbuf.get_rowstride() + x * pixbuf.get_n_channels()) as usize;
        let pixels = unsafe { pixbuf.get_pixels() };

        (pixels[pos],
         pixels[pos + 1],
         pixels[pos + 2],
         if pixbuf.get_has_alpha() { pixels[pos + 3] } else { 0 },
        )
    }

    pub fn get_rotated_pixbuf(pixbuf: gdk_pixbuf::Pixbuf, arg: f64) -> gdk_pixbuf::Pixbuf {
        if arg == 0.0 { return pixbuf; }
        let arg = arg * PI / 180.0;

        let new_width = (pixbuf.get_width() as f64 * arg.cos().abs() + pixbuf.get_height() as f64 * arg.sin().abs()) as i32;
        let new_height = (pixbuf.get_width() as f64 * arg.sin().abs() + pixbuf.get_height() as f64 * arg.cos().abs()) as i32;
        let new_pixbuf = unsafe { gdk_pixbuf::Pixbuf::new(
            pixbuf.get_colorspace(),
            true,
            pixbuf.get_bits_per_sample(),
            new_width,
            new_height,
        ).unwrap() };

        let width = pixbuf.get_width();
        let height = pixbuf.get_height();

        for iy in 0..new_height {
            for ix in 0..new_width {
                let (ix_prev, iy_prev) = {
                    let (x,y) = Effect::rotate(-arg, ix - new_width / 2, iy - new_height / 2);
                    (x + width / 2, y + height / 2)
                };
                if 0 <= ix_prev && ix_prev < width &&
                    0 <= iy_prev && iy_prev < height {
                        let (r,g,b,a) = Effect::get_pixel(&pixbuf, ix_prev, iy_prev);
                        new_pixbuf.put_pixel(ix, iy, r, g, b, a);
                    }
                else {
                    new_pixbuf.put_pixel(ix, iy, 0, 0, 0, 255);
                }
            }
        }

        new_pixbuf
    }

    pub fn effect_on_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf, current: f64) -> gdk_pixbuf::Pixbuf {
        use EffectType::*;

        match self.effect_type {
            Rotate => Effect::get_rotated_pixbuf(pixbuf, self.value(current)),
            _ => pixbuf,
        }
    }

    pub fn value(&self, current: f64) -> f64 {
        self.start_value + self.transition.get_in_interval(current) * (self.end_value - self.start_value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(public)]
pub struct Component {
    pub component_type: ComponentType,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub length: gst::ClockTime,

    #[serde(default = "coordinate_default")]
    #[builder(default = "coordinate_default()")]
    pub coordinate: (i32,i32),

    pub layer_index: usize,

    #[serde(default = "rotate_default")]
    #[builder(default = "rotate_default()")]
    pub rotate: f64,

    #[serde(default = "alpha_default")]
    #[builder(default = "alpha_default()")]
    pub alpha: i32,

    pub entity: String,

    #[serde(default = "scale_default")]
    #[builder(default = "scale_default()")]
    pub scale: (f64, f64),

    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub effect: Vec<Effect>,
}

fn coordinate_default() -> (i32, i32) { (0,0) }
fn rotate_default() -> f64 { 0.0 }
fn alpha_default() -> i32 { 255 }
fn scale_default() -> (f64, f64) { (1.0,1.0) }

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

#[derive(Debug, Clone)]
pub enum Property {
    I32(i32),
    F64(f64),
    Usize(usize),
    Time(gst::ClockTime),
    Pair(Box<Property>, Box<Property>),
    FilePath(String),
    Document(String),
    Font(String),
    Color(gdk::RGBA),
    ReadOnly(String),
    Choose(String,i32),
    EffectInfo(EffectType, Transition, f64, f64),
}

impl Property {
    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Property::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        use Property::*;

        match self {
            &I32(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        use Property::*;

        match self {
            &F64(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_choose(&self) -> Option<i32> {
        use Property::*;

        match self {
            &Choose(_,t) => Some(t),
            _ => None,
        }
    }

    pub fn as_effect(&self) -> Option<Effect> {
        use Property::*;

        match self {
            &EffectInfo(ref typ, ref trans, ref start, ref end) => Some(Effect {
                effect_type: typ.clone(),
                transition: trans.clone(),
                start_value: *start,
                end_value: *end,
            }),
            _ => None,
        }
    }
}

pub type Properties = HashMap<String, Property>;

pub trait ComponentWrapper : AsRef<Component> + AsMut<Component> {
    fn get_properties(&self) -> Properties {
        self.as_ref().get_properties()
    }

    fn set_property(&mut self, name: String, prop: Property) {
        self.as_mut().set_property(name, prop);
    }

    fn get_effect_properties(&self) -> Vec<Property> {
        self.as_ref().get_effect_properties()
    }

    fn set_effect_property(&mut self, i: usize, prop: Property) {
        self.as_mut().set_effect_property(i, prop);
    }

    fn add_effect_property(&mut self, prop: Property) {
        self.as_mut().add_effect_property(prop);
    }
}

impl AsRef<Component> for Component {
    fn as_ref(&self) -> &Component {
        self
    }
}

impl AsMut<Component> for Component {
    fn as_mut(&mut self) -> &mut Component {
        self
    }
}

impl ComponentWrapper for Component {
    fn get_properties(&self) -> Properties {
        use Property::*;

        [
            ("component_type".to_string(), ReadOnly(format!("{:?}", self.component_type))),
            ("start_time".to_string(), Time(self.start_time)),
            ("length".to_string(), Time(self.length)),
            ("coordinate".to_string(), Pair(box I32(self.coordinate.0), box I32(self.coordinate.1))),
            ("entity".to_string(), ReadOnly(self.entity.clone())),
            ("layer_index".to_string(), Usize(self.layer_index)),
            ("rotate".to_string(), F64(self.rotate)),
            ("alpha".to_string(), I32(self.alpha)),
            ("scale".to_string(), Pair(box F64(self.scale.0), box F64(self.scale.1))),
        ].iter().cloned().collect()
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("start_time", Time(v)) => self.start_time = v,
            ("length", Time(v)) => self.length = v,
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("layer_index", Usize(v)) => self.layer_index = v,
            ("rotate", F64(v)) => self.rotate = v,
            ("alpha", I32(v)) => self.alpha = v,
            ("scale", Pair(box F64(x), box F64(y))) => self.scale = (x,y),
            _ => unimplemented!(),
        }
    }

    fn get_effect_properties(&self) -> Vec<Property> {
        use Property::*;

        self.effect.iter().map(|eff| {
            EffectInfo(eff.effect_type.clone(), eff.transition.clone(), eff.start_value, eff.end_value)
        }).collect()
    }

    fn set_effect_property(&mut self, i: usize, prop: Property) {
        self.effect[i] = prop.as_effect().unwrap();
    }

    fn add_effect_property(&mut self, prop: Property) {
        self.effect.push(prop.as_effect().unwrap());
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable {}
impl<T: ComponentWrapper + Peekable> ComponentLike for T {}

impl Deref for ComponentLike {
    type Target = Component;

    fn deref(&self) -> &Component {
        self.as_ref()
    }
}

impl DerefMut for ComponentLike {
    fn deref_mut(&mut self) -> &mut Component {
        self.as_mut()
    }
}

pub enum GdkInterpType {
    Nearest,
    Tiles,
    Bilinear,
    Hyper
}

impl GdkInterpType {
    pub fn to_i32(&self) -> i32 {
        use GdkInterpType::*;

        match self {
            &Nearest => 0,
            &Tiles => 1,
            &Bilinear => 2,
            &Hyper => 3,
        }
    }
}

