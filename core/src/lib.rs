extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate gdk_pixbuf;
extern crate gstreamer as gst;
extern crate gstreamer_video as gstv;
#[macro_use] extern crate maplit;
extern crate uuid;
extern crate base64;
extern crate glib;

pub mod util;

pub mod spec;
pub mod feat;

pub use spec::*;
pub use feat::*;

#[derive(Clone)]
pub struct Madder {
    project: Project,
    component_repo: ComponentRepositoryImpl,
    effect_repo: EffectRepositoryImpl,
    server: ApiServer,
    renderer: Option<AviRenderer>,
}

unsafe impl Send for Madder {}

impl HaveEffectRepository for Madder {
    type EffectRepository = EffectRepositoryImpl;

    fn effect_repo(&self) -> &Self::EffectRepository {
        &self.effect_repo
    }

    fn effect_repo_mut(&mut self) -> &mut Self::EffectRepository {
        &mut self.effect_repo
    }
}

impl HaveComponentRepository for Madder {
    type COMPONENT = ComponentExt;
    type ComponentRepository = ComponentRepositoryImpl;

    fn component_repo(&self) -> &Self::ComponentRepository {
        &self.component_repo
    }

    fn component_repo_mut(&mut self) -> &mut Self::ComponentRepository {
        &mut self.component_repo
    }
}

impl HaveProject for Madder {
    type COMPONENT = ComponentExt;

    fn project(&self) -> &Project {
        &self.project
    }

    fn project_mut(&mut self) -> &mut Project {
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

impl HavePresenter for Madder {}

impl ProjectLoader for Madder {}

impl HaveAviRenderer for Madder {
    fn renderer(&self) -> &AviRenderer {
        self.renderer.as_ref().unwrap()
    }

    fn renderer_mut(&mut self) -> &mut AviRenderer {
        self.renderer.as_mut().unwrap()
    }
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project::new(640, 480, 100 * gst::MSECOND),
            component_repo: ComponentRepositoryImpl::new(),
            effect_repo: EffectRepositoryImpl::new(),
            server: ApiServer::new(),
            renderer: None,
        }
    }
}

pub fn init() {
    gst::init().unwrap();
}

