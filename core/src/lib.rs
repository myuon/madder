extern crate gdk_pixbuf;
extern crate gstreamer as gst;

pub mod spec;
pub mod feat;

use spec::*;
use feat::*;

pub struct Madder {
    project: Project<ComponentExt>,
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project::new(640, 480, 100 * gst::MSECOND),
        }
    }
}

