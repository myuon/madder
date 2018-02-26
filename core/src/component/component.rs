use std::collections::HashMap;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EffectType {
    Coordinate,
    Rotate,
    Scale,
    Alpha,
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
    pub fn make_effect(&self, component: Component, current: f64) -> Component {
        use EffectType::*;

        match self.effect_type {
            Coordinate => {
                let mut comp = component;
                comp.coordinate.0 += self.value(current) as i32;
                comp
            },
            _ => unimplemented!(),
        }
    }

    pub fn value(&self, current: f64) -> f64 {
        self.start_value + self.transition.get_in_interval(current) * (self.end_value - self.start_value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Component {
    pub component_type: ComponentType,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub start_time: gst::ClockTime,

    #[serde(serialize_with = "gst_clocktime_serialize")]
    #[serde(deserialize_with = "gst_clocktime_deserialize")]
    pub length: gst::ClockTime,

    pub coordinate: (i32,i32),

    pub layer_index: usize,

    pub entity: String,

    #[serde(default = "Vec::new")]
    pub effect: Vec<Effect>,
}

fn gst_clocktime_serialize<S: serde::Serializer>(g: &gst::ClockTime, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(g.mseconds().unwrap())
}

fn gst_clocktime_deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<gst::ClockTime, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(gst::ClockTime::from_mseconds)
}

#[derive(Debug, Clone)]
pub enum Property {
    I32(i32),
    Usize(usize),
    Time(gst::ClockTime),
    Pair(Box<Property>, Box<Property>),
    FilePath(String),
    Document(String),
    Font(String),
    Color(gdk::RGBA),
    ReadOnly(String),
    EffectInfo(EffectType, Transition, f64, f64),
}

#[derive(Debug, Clone)]
pub enum PropertyGroupTab {
    ComponentProperty,
    EffectProperty,
}

impl Property {
    pub fn as_time(&self) -> Option<gst::ClockTime> {
        use Property::*;

        match self {
            &Time(t) => Some(t),
            _ => None,
        }
    }

    pub fn get_group_tab(&self) -> PropertyGroupTab {
        use Property::*;
        use PropertyGroupTab::*;

        match self {
            &EffectInfo(_,_,_,_) => EffectProperty,
            _ => ComponentProperty,
        }
    }
}

pub type Properties = HashMap<String, Property>;

pub trait ComponentWrapper {
    fn get_component(&self) -> Component;
    fn get_properties(&self) -> Properties;
    fn set_property(&mut self, name: String, prop: Property);
}

impl ComponentWrapper for Component {
    fn get_component(&self) -> Component {
        self.clone()
    }

    fn get_properties(&self) -> Properties {
        use Property::*;

        let mut props: Properties = [
            ("component_type".to_string(), ReadOnly(format!("{:?}", self.component_type))),
            ("start_time".to_string(), Time(self.start_time)),
            ("length".to_string(), Time(self.length)),
            ("coordinate".to_string(), Pair(box I32(self.coordinate.0), box I32(self.coordinate.1))),
            ("entity".to_string(), ReadOnly(self.entity.clone())),
            ("layer_index".to_string(), Usize(self.layer_index)),
        ].iter().cloned().collect();

        self.effect.iter().enumerate().for_each(|(i,eff)| {
            props.insert(i.to_string(), EffectInfo(eff.effect_type.clone(), eff.transition.clone(), eff.start_value, eff.end_value));
        });

        props
    }

    fn set_property(&mut self, name: String, prop: Property) {
        use Property::*;

        match (name.as_str(), prop) {
            ("start_time", Time(v)) => self.start_time = v,
            ("length", Time(v)) => self.length = v,
            ("coordinate", Pair(box I32(x), box I32(y))) => self.coordinate = (x,y),
            ("layer_index", Usize(v)) => self.layer_index = v,
            (i, EffectInfo(effect_type, transition, start_value, end_value)) =>
                self.effect[i.parse::<usize>().unwrap()] = Effect {
                    effect_type: effect_type.clone(),
                    transition: transition.clone(),
                    start_value: start_value,
                    end_value: end_value,
                },
            _ => unimplemented!(),
        }
    }
}

impl<T: ComponentWrapper> ComponentWrapper for Box<T> {
    fn get_component(&self) -> Component {
        self.as_ref().get_component()
    }

    fn get_properties(&self) -> Properties {
        self.as_ref().get_properties()
    }

    fn set_property(&mut self, name: String, prop: Property) {
        self.as_mut().set_property(name, prop);
    }
}

pub trait ComponentLike: ComponentWrapper + Peekable {}
impl<T: ComponentWrapper + Peekable> ComponentLike for T {}

