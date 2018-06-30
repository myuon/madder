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
        router.add("/components", Handler::Get("mapper_get_components"));

        ApiServer {
            router: router,
        }
    }
}

pub trait HaveApiServer : HaveProject {
    fn server(&self) -> &ApiServer;
    fn server_mut(&mut self) -> &mut ApiServer;

    fn create(&mut self, path: &str, entity: Document) -> Result<(), String> {
        use self::Handler::*;

        let matcher = self.server_mut().router.recognize(path)?;
        match matcher.handler {
//            Create(h) => Ok((h.as_ref())(self, matcher.params, entity)),
            _ => unreachable!(),
        }
    }

    fn mapper_get_components(&self, _: router::Params) -> Document {
        Document {
            payload: Payload::Data(PrimaryData::Multiple(
                self.project().get_components_at_layer(0).iter().enumerate().map(|(i,component)| {
                    Resource {
                        id: i.to_string(),
                        type_: "component".to_string(),
                        attributes: hashmap!{
                            "start_time".to_string() => json!(SerTime(component.component().start_time)),
                            "length".to_string() => json!(SerTime(component.component().length)),
                        },
                        relationships: Default::default(),
                    }
                }).collect()
            )),
            meta: serde_json::Value::Null,
            jsonapi: serde_json::Value::Null,
            links: None,
            included: vec![],
        }
    }

    fn mapper_get(&self, name: &str, params: router::Params) -> Document {
        match name {
            "mapper_get_components" => self.mapper_get_components(params),
            _ => unreachable!(),
        }
    }

    fn get(&self, path: &str) -> Result<Document, String> {
        use self::Handler::*;

        let matcher = self.server().router.recognize(path)?;
        match matcher.handler {
            Get(name) => Ok(self.mapper_get(name, matcher.params)),
            _ => unreachable!(),
        }
    }

    fn update(&mut self, path: &str, entity: Document) -> Result<(), String> {
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
}

