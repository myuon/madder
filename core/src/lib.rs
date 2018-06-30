extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
#[macro_use] extern crate maplit;
extern crate madder_util as util;

pub mod spec;
pub mod feat;

use spec::*;
use feat::*;

pub struct Madder {
    project: Project<ComponentExt>,
    server: ApiServer,
}

impl HaveProject for Madder {
    type COMPONENT = ComponentExt;

    fn project(&self) -> &Project<Self::COMPONENT> {
        &self.project
    }

    fn project_mut(&mut self) -> &mut Project<Self::COMPONENT> {
        &mut self.project
    }
}

impl HaveApiServer for Madder {
    fn server(&self) -> &ApiServer {
        &self.server
    }

    fn server_mut(&mut self) -> &mut ApiServer {
        &mut self.server
    }
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project::new(640, 480, 100 * gst::MSECOND),
            server: ApiServer::new(),
        }
    }
}

#[test]
fn test_json_api() {
    let madder = Madder::new();
    madder.get("/components").unwrap();
}

