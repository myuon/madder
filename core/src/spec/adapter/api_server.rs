extern crate route_recognizer as router;
extern crate maplit;
extern crate serde_json;
extern crate madder_util as util;

use spec::*;
use util::serde_impl::*;

enum Handler {
    Create(String),
    Get(&'static str),
    Update(String),
    Delete(String),
}

pub struct ApiServer {
    router: router::Router<Handler>,
}

impl ApiServer {
    pub fn new() -> ApiServer {
        let mut router = router::Router::new();
        router.add("/component", Handler::Get("mapper_list_component"));
        router.add("/component/:component_id", Handler::Get("mapper_get_component"));

        ApiServer {
            router: router,
        }
    }
}

pub trait HaveApiServer : HaveProject + HaveComponentRepository {
    fn server(&self) -> &ApiServer;
    fn server_mut(&mut self) -> &mut ApiServer;

    fn create(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        use self::Handler::*;

        let mapper = |name, params| {
            match name {
                "mapper_create_component" => self.mapper_create_component(params, entity),
            }
        };

        let matcher = self.server_mut().router.recognize(path)?;
        match matcher.handler {
            Create(name) => Ok(mapper(name, matcher.params)),
            _ => unreachable!(),
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        use self::Handler::*;

        let mapper = |name, params| {
            match name {
                "mapper_list_component" => self.mapper_list_component(params),
                "mapper_get_component" => self.mapper_get_component(params),
                _ => unreachable!(),
            }
        };

        let matcher = self.server().router.recognize(path)?;
        match matcher.handler {
            Get(name) => Ok(mapper(name, matcher.params)),
            _ => unreachable!(),
        }
    }

    fn update(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        use self::Handler::*;

        let matcher = self.server_mut().router.recognize(path)?;
        match matcher.handler {
//            Update(h) => Ok((h.as_ref())(matcher.params, entity)),
            _ => unreachable!(),
        }
    }

    fn delete(&mut self, path: &str) -> Result<(), String> {
        use self::Handler::*;

        let matcher = self.server_mut().router.recognize(path)?;
        match matcher.handler {
//            Delete(h) => Ok((h.as_ref())(matcher.params)),
            _ => unreachable!(),
        }
    }

    fn mapper_list_component(&self, _: router::Params) -> serde_json::Value {
        json!(self.component_repo().list())
    }

    fn mapper_create_component(&self, _: router::Params, entity: serde_json::Value) {
        self.component_repo_mut().create(serde_json::from_value(entity).unwrap());
    }

    fn mapper_get_component(&self, params: router::Params) -> serde_json::Value {
        json!(self.component_repo().get(params.find("component_id").and_then(|x| x.parse().ok()).unwrap()).entity)
    }

    fn mapper_get_component_attribute(&self, params: router::Params) -> serde_json::Value {
        json!(self.component_repo().get(params.find("component_id").and_then(|x| x.parse().ok()).unwrap()).entity.component().attributes[params.find("key").unwrap()])
    }
}

