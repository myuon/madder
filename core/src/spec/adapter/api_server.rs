extern crate route_recognizer as router;
extern crate maplit;
extern crate serde_json;
extern crate gstreamer as gst;
extern crate base64;

use spec::*;
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    Create,
    Get,
    Update,
    Delete,
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    method: Method,
    pub path: String,
    pub entity: serde_json::Value,
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
        let mapper = hashmap!{
            "/component" => vec![
                (Get, "mapper_list_component"),
                (Create, "mapper_create_component"),
            ],
            "/component/:component_id" => vec![
                (Get, "mapper_get_component"),
                (Delete, "mapper_delete_component"),
                (Update, "mapper_update_component"),
            ],
            "/component/:component_id/attribute/:key" => vec![
                (Get, "mapper_get_component_attribute"),
                (Update, "mapper_update_component_attribute"),
            ],
            "/component/:component_id/effect" => vec![
                (Get, "mapper_list_component_effect"),
                (Create, "mapper_create_component_effect"),
            ],
            "/component/:component_id/effect/:index" => vec![
                (Get, "mapper_get_component_effect"),
                (Create, "mapper_insert_component_effect"),
            ],
            "/effect" => vec![
                (Get, "mapper_list_effect"),
            ],
            "/effect/:effect_id" => vec![
                (Get, "mapper_get_effect"),
            ],
            "/effect/:effect_id/intermed" => vec![
                (Create, "mapper_create_effet_intermed"),
            ],
            "/effect/:effect_id/value/:time" => vec![
                (Get, "mapper_get_effect_value"),
            ],
            "/project/yaml" => vec![
                (Get, "mapper_get_project_yaml"),
                (Update, "mapper_update_project_yaml"),
            ],
            "/screen/:time" => vec![
                (Get, "mapper_get_screen"),
            ],
        };

        for (k,v) in mapper {
            for (method, func) in v {
                router.get_mut(&method).unwrap().add(k, func);
            }
        }

        ApiServer {
            router: router,
        }
    }
}

pub trait HaveApiServer : HavePresenter + ProjectLoader {
    fn server(&self) -> &ApiServer;
    fn server_mut(&mut self) -> &mut ApiServer;

    fn req(&mut self, req: Request) -> Result<serde_json::Value, String> {
        self.request(req.method, &req.path, req.entity)
    }

    fn request(&mut self, method: Method, path: &str, entity: serde_json::Value) -> Result<serde_json::Value, String> {
        use self::Method::*;

        match method {
            Create => self.create(path, entity).map(|v| json!(v)),
            Get => self.get(path),
            Update => self.update(path, entity).map(|v| json!(v)),
            Delete => self.delete(path).map(|v| json!(v)),
        }
    }

    fn create(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        let r = self.server().router[&Method::Create].clone();
        let matcher = r.recognize(path)?;
        let result = match *matcher.handler {
            "mapper_create_component" => self.mapper_create_component(matcher.params, entity),
            "mapper_create_component_effect" => self.mapper_create_component_effect(matcher.params, entity),
            "mapper_insert_component_effect" => self.mapper_insert_component_effect(matcher.params, entity),
            "mapper_create_effet_intermed" => self.mapper_create_effect_intermed(matcher.params, entity),
            _ => unreachable!("{}", path),
        };

        Ok(result)
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let matcher = self.server().router[&Method::Get].recognize(path)?;
        let result = match *matcher.handler {
            "mapper_list_component" => self.mapper_list_component(matcher.params),
            "mapper_get_component" => self.mapper_get_component(matcher.params),
            "mapper_get_component_attribute" => self.mapper_get_component_attribute(matcher.params),
            "mapper_list_component_effect" => self.mapper_list_component_effect(matcher.params),
            "mapper_get_component_effect" => self.mapper_get_component_effect(matcher.params),
            "mapper_list_effect" => self.mapper_list_effect(matcher.params),
            "mapper_get_effect" => self.mapper_get_effect(matcher.params),
            "mapper_get_effect_value" => self.mapper_get_effect_value(matcher.params),
            "mapper_get_project_yaml" => self.mapper_get_project_yaml(matcher.params),
            "mapper_get_screen" => self.mapper_get_screen(matcher.params),
            _ => unreachable!("{}", path),
        };

        Ok(result)
    }

    fn update(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        let r = self.server().router[&Method::Update].clone();
        let matcher = r.recognize(path)?;
        let result = match *matcher.handler {
            "mapper_update_component" => self.mapper_update_component(matcher.params, entity),
            "mapper_update_component_attribute" => self.mapper_update_component_attribute(matcher.params, entity),
            "mapper_update_project_yaml" => self.mapper_update_project_yaml(matcher.params, entity),
            _ => unreachable!("{}", path),
        };

        Ok(result)
    }

    fn delete(&mut self, path: &str) -> Result<(), String> {
        let r = self.server().router[&Method::Delete].clone();
        let matcher = r.recognize(path)?;
        let _result = match *matcher.handler {
            "mapper_delete_component" => self.mapper_delete_component(matcher.params),
            _ => unreachable!("{}", path),
        };

        Ok(())
    }

    fn mapper_list_component(&self, _: router::Params) -> serde_json::Value {
        json!(self.component_repo().list())
    }

    fn mapper_list_component_effect(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").unwrap();
        json!(self.component_repo().get(component_id).component().effect.iter().map(|effect_id| {
            self.effect_repo().get(effect_id)
        }).collect::<Vec<&Effect>>())
    }

    fn mapper_list_effect(&self, _: router::Params) -> serde_json::Value {
        json!(self.effect_repo().list())
    }

    fn mapper_get_component(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").unwrap();
        json!(self.component_repo().get(component_id))
    }

    fn mapper_get_component_attribute(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").unwrap();
        json!(self.component_repo().get(component_id).component().attributes[params.find("key").unwrap()])
    }

    fn mapper_get_component_effect(&self, params: router::Params) -> serde_json::Value {
        let component_id = params.find("component_id").unwrap();
        let index: usize = params.find("index").and_then(|x| x.parse().ok()).unwrap();
        let effect_id = &self.component_repo().get(component_id).component().effect[index];
        json!(self.effect_repo().get(effect_id))
    }

    fn mapper_get_effect(&self, params: router::Params) -> serde_json::Value {
        let effect_id = params.find("effect_id").unwrap();
        json!(self.effect_repo().get(effect_id))
    }

    fn mapper_get_effect_value(&self, params: router::Params) -> serde_json::Value {
        let effect_id = params.find("effect_id").unwrap();
        let time = params.find("time").and_then(|x| x.parse().ok()).unwrap();
        json!(self.effect_repo().value(effect_id, time))
    }

    fn mapper_get_screen(&self, params: router::Params) -> serde_json::Value {
        let time: u64 = params.find("time").and_then(|x| x.parse().ok()).unwrap();
        let encoded = base64::encode(&self.get_pixbuf(time * gst::MSECOND).save_to_bufferv("png", &[]).unwrap());
        json!(format!("data:image/png;base64,{}", encoded))
    }

    fn mapper_get_project_yaml(&self, _: router::Params) -> serde_json::Value {
        json!(self.to_yaml_string().unwrap())
    }

    fn mapper_create_component(&mut self, _: router::Params, entity: serde_json::Value) {
        let key = self.component_repo_mut().create(<Self as HaveComponentRepository>::new_from_json(entity));
        self.project_mut().add_component_at(0, key);
    }

    fn mapper_create_component_effect(&mut self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").unwrap();
        let effect_id = self.effect_repo_mut().create(serde_json::from_value(entity).unwrap()).to_string();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().effect.push(effect_id);
    }

    fn mapper_create_effect_intermed(&mut self, params: router::Params, entity: serde_json::Value) {
        let effect_id = params.find("effect_id").unwrap();
        self.effect_repo_mut().create_intermed(effect_id, serde_json::from_value(entity).unwrap());
    }

    fn mapper_insert_component_effect(&mut self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").unwrap();
        let index: usize = params.find("index").and_then(|x| x.parse().ok()).unwrap();
        let effect_id = self.effect_repo_mut().create(serde_json::from_value(entity).unwrap()).to_string();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().effect.insert(index, effect_id);
    }

    fn mapper_delete_component(&mut self, params: router::Params) {
        let component_id = params.find("component_id").unwrap();
        self.component_repo_mut().delete(component_id);
    }

    fn mapper_update_component(&mut self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").unwrap();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().partial_update(entity.as_object().unwrap());
    }

    fn mapper_update_component_attribute(&mut self, params: router::Params, entity: serde_json::Value) {
        let component_id = params.find("component_id").unwrap();
        let key = params.find("key").unwrap();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().attributes.insert(key.to_string(), entity);
    }

    fn mapper_update_project_yaml(&mut self, _: router::Params, entity: serde_json::Value) {
        self.from_yaml_string(entity.as_str().unwrap()).unwrap();
    }
}

