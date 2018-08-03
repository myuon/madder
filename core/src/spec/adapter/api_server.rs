extern crate route_recognizer as router;
extern crate maplit;
extern crate serde_json;
extern crate madder_util as util;

use spec::*;
use util::serde_impl::*;
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq)]
enum Method {
    Create,
    Get,
    Update,
    Delete,
}

pub struct ApiServer {
    router: HashMap<Method, router::Router<&'static str>>,
}

impl ApiServer {
    pub fn new() -> ApiServer {
        use self::Method::*;

        let mut router = hashmap!(
            Create => router::Router::new(),
            Get => router::Router::new(),
            Update => router::Router::new(),
            Delete => router::Router::new(),
        );

        // tsura
        router[&Get].add("/component", "mapper_list_component");
        router[&Create].add("/component", "mapper_create_component");
        
        router[&Get].add("/component/:component_id", "mapper_get_component");

        router[&Update].add("/component/:component_id/attribute/:key", "mapper_update_component_attribute");

        router[&Get].add("/component/:component_id/effect", "mapper_list_component_effect");
        router[&Create].add("/component/:component_id/effect", "mapper_create_component_effect");

        router[&Get].add("/component/:component_id/effect/:effect_id", "mapper_get_component_effect");
        router[&Create].add("/component/:component_id/effect/:effect_id", "mapper_insert_component_effect");

        ApiServer {
            router: router,
        }
    }
}

pub trait HaveApiServer : HaveProject + HaveComponentRepository {
    fn server(&self) -> &ApiServer;
    fn server_mut(&mut self) -> &mut ApiServer;

    fn create(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        let matcher = self.server().router[&Method::Create].recognize(path)?;
        let result = match *matcher.handler {
            "mapper_create_component" => self.mapper_create_component(matcher.params, entity),
        };

        Ok(result)
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let matcher = self.server().router[&Method::Get].recognize(path)?;
        let result = match *matcher.handler {
            "mapper_list_component" => self.mapper_list_component(matcher.params),
            "mapper_get_component" => self.mapper_get_component(matcher.params),
            _ => unreachable!(),
        };
        
        Ok(result)
    }

    fn update(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        let matcher = self.server().router[&Method::Update].recognize(path)?;
        let result = match *matcher.handler {
            _ => unreachable!(),
        };
        
        Ok(result)
    }

    fn delete(&mut self, path: &str) -> Result<(), String> {
        let matcher = self.server().router[&Method::Delete].recognize(path)?;
        let result = match *matcher.handler {
            _ => unreachable!(),
        };
        
        Ok(result)
    }

    fn mapper_list_component(&self, _: router::Params) -> serde_json::Value {
        json!(self.component_repo().list())
    }

    fn mapper_create_component(&self, _: router::Params, entity: serde_json::Value) {
        self.component_repo_mut().create(serde_json::from_value(entity).unwrap());
    }

    fn mapper_get_component(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").and_then(|x| x.parse().ok()).unwrap();
        json!(self.component_repo().get(component_id).entity)
    }

    fn mapper_get_component_attribute(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").and_then(|x| x.parse().ok()).unwrap();
        json!(self.component_repo().get(component_id).entity.component().attributes[params.find("key").unwrap()])
    }

    fn mapper_update_component_attribute(&self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").and_then(|x| x.parse().ok()).unwrap();
        let key = params.find("key").unwrap();
        let mut component = self.component_repo().get(component_id).entity;
        component.component_mut().attributes[key] = entity;
        self.component_repo_mut().update(component_id, component);
    }

    fn mapper_list_effect(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").and_then(|x| x.parse().ok()).unwrap();
        json!(self.component_repo().get(component_id).entity)
    }

    fn mapper_create_effect(&self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").and_then(|x| x.parse().ok()).unwrap();
        let effect_id = params.find("effect_id").and_then(|x| x.parse().ok()).unwrap();
        let mut component = self.component_repo().get(component_id).entity;
        component.effect.insert(effect_id, serde_json::from_value(entity));
        self.component_repo_mut().update(component_id, component);
    }
}

