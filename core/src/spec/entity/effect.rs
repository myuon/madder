extern crate gdk_pixbuf;
use gdk_pixbuf::prelude::*;
use std::f32::consts::PI;

#[derive(Clone, Serialize, Deserialize)]
pub struct EffectPoint {
    transition: Transition,
    position: f32,
    value: f32,
}

#[derive(Clone, Serialize, Deserialize)]
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

    fn get_in_interval(&self, x: f32) -> f32 {
        use Transition::*;

        match self {
            &Linear => x,
            &Ease => Transition::cubic_bezier(0.25, 0.1, 0.25, 1.0, x),
            &EaseIn => Transition::cubic_bezier(0.42, 0.0, 1.0, 1.0, x),
            &EaseOut => Transition::cubic_bezier(0.0, 0.0, 0.58, 1.0, x),
            &EaseInOut => Transition::cubic_bezier(0.42, 0.0, 0.58, 1.0, x),
        }
    }

    fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, x: f32) -> f32 {
        // cubic bezier calculation by Newton method
        //
        // x = (3 P2.x - 3 P3.x + 1) t^3 + (-6 P2.x + 3 P3.x) t^2 + (3 P2.x) t
        // y = (3 P2.y - 3 P3.y + 1) t^3 + (-6 P2.y + 3 P3.y) t^2 + (3 P2.y) t
        // (0 <= t <= 1)
        //
        // x' = 3 (3 P2.x - 3 P3.x + 1) t^2 + 2 (-6 P2.x + 3 P3.x) t + 3 P2.x
        const MAX_ITERATION: i32 = 50;
        const NEIGHBOR: f32 = 0.01;

        fn _bezier_params(u: f32, v: f32) -> (f32, f32, f32) {
            let k3 = 3.0 * u - 3.0 * v + 1.0;
            let k2 = -6.0 * u + 3.0 * v;
            let k1 = 3.0 * u;

            (k1,k2,k3)
        }

        fn bezier(u: f32, v: f32, t: f32) -> f32 {
            let (k1,k2,k3) = _bezier_params(u,v);
            (((k3 * t + k2) * t) + k1) * t
        }

        fn bezier_dt(u: f32, v: f32, t: f32) -> f32 {
            let (k1,k2,k3) = _bezier_params(u,v);
            ((3.0 * k3 * t + 2.0 * k2) * t) + k1
        }

        let bezier_x = |t: f32| { bezier(p0, p2, t) };
        let bezier_dt_x = |t: f32| { bezier_dt(p0, p2, t) };
        let bezier_y = |t: f32| { bezier(p1, p3, t) };

        let get_t_at_x = |x: f32| {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Effect {
    pub effect_type: String,
    pub transition: Transition,
    pub start_value: f32,
    pub end_value: f32,
    pub intervals: Vec<EffectPoint>,
}

impl Effect {
    pub fn rotate(arg: f32, x: i32, y: i32) -> (i32, i32) {
        ((x as f32 * arg.cos() + y as f32 * arg.sin()) as i32,
         (x as f32 * -arg.sin() + y as f32 * arg.cos()) as i32,
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

    pub fn get_rotated_pixbuf(pixbuf: gdk_pixbuf::Pixbuf, arg: f32) -> gdk_pixbuf::Pixbuf {
        if arg == 0.0 { return pixbuf; }
        let arg = arg * PI / 180.0;

        let new_width = (pixbuf.get_width() as f32 * arg.cos().abs() + pixbuf.get_height() as f32 * arg.sin().abs()) as i32;
        let new_height = (pixbuf.get_width() as f32 * arg.sin().abs() + pixbuf.get_height() as f32 * arg.cos().abs()) as i32;
        let new_pixbuf = gdk_pixbuf::Pixbuf::new(
            pixbuf.get_colorspace(),
            true,
            pixbuf.get_bits_per_sample(),
            new_width,
            new_height,
        );

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

    pub fn effect_on_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf, current: f32) -> gdk_pixbuf::Pixbuf {
        match self.effect_type.as_str() {
            "rotate" => Effect::get_rotated_pixbuf(pixbuf, self.value(current)),
            _ => pixbuf,
        }
    }

    pub fn value(&self, current: f32) -> f32 {
        let find_corresponding_interval = || -> (f32, f32, f32, Transition) {
            let mut prev_time = 0.0;
            let mut prev_value = self.start_value;

            for intermed in &self.intervals {
                if current <= intermed.position {
                    return ((current - prev_time) / (intermed.position - prev_time), prev_value, intermed.value, intermed.transition.clone())
                }

                prev_time = intermed.position;
                prev_value = intermed.value;
            }

            (current, prev_value, self.end_value, self.transition.clone())
        };

        let (current, start, end, transition) = find_corresponding_interval();
        start + transition.get_in_interval(current) * (end - start)
    }
}

