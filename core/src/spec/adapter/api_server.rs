extern crate route_recognizer as router;
extern crate maplit;
extern crate serde_json;
extern crate gstreamer as gst;
extern crate base64;

use spec::*;
use std::collections::HashMap;
use std::num::{ParseIntError, ParseFloatError};

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct ApiServer {
    router: HashMap<Method, router::Router<&'static str>>,
}

pub struct ParamHolder(router::Params);

impl ParamHolder {
    fn find(&self, param: &str) -> Result<&str, String> {
        self.0.find(param).ok_or(format!("No such parameter: {}", param))
    }

    fn find_as_usize(&self, param: &str) -> Result<usize, String> {
        self.find(param)?.parse().map_err(|x: ParseIntError| x.to_string())
    }

    fn find_as_u64(&self, param: &str) -> Result<u64, String> {
        self.find(param)?.parse().map_err(|x: ParseIntError| x.to_string())
    }

    fn find_as_f32(&self, param: &str) -> Result<f32, String> {
        self.find(param)?.parse().map_err(|x: ParseFloatError| x.to_string())
    }
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
        match *matcher.handler {
            "mapper_create_component" => self.mapper_create_component(ParamHolder(matcher.params), entity),
            "mapper_create_component_effect" => self.mapper_create_component_effect(ParamHolder(matcher.params), entity),
            "mapper_insert_component_effect" => self.mapper_insert_component_effect(ParamHolder(matcher.params), entity),
            "mapper_create_effet_intermed" => self.mapper_create_effect_intermed(ParamHolder(matcher.params), entity),
            _ => unreachable!("{}", path),
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let matcher = self.server().router[&Method::Get].recognize(path)?;
        match *matcher.handler {
            "mapper_list_component" => self.mapper_list_component(ParamHolder(matcher.params)),
            "mapper_get_component" => self.mapper_get_component(ParamHolder(matcher.params)),
            "mapper_get_component_attribute" => self.mapper_get_component_attribute(ParamHolder(matcher.params)),
            "mapper_list_component_effect" => self.mapper_list_component_effect(ParamHolder(matcher.params)),
            "mapper_get_component_effect" => self.mapper_get_component_effect(ParamHolder(matcher.params)),
            "mapper_list_effect" => self.mapper_list_effect(ParamHolder(matcher.params)),
            "mapper_get_effect" => self.mapper_get_effect(ParamHolder(matcher.params)),
            "mapper_get_effect_value" => self.mapper_get_effect_value(ParamHolder(matcher.params)),
            "mapper_get_project_yaml" => self.mapper_get_project_yaml(ParamHolder(matcher.params)),
            "mapper_get_screen" => self.mapper_get_screen(ParamHolder(matcher.params)),
            _ => unreachable!("{}", path),
        }
    }

    fn update(&mut self, path: &str, entity: serde_json::Value) -> Result<(), String> {
        let r = self.server().router[&Method::Update].clone();
        let matcher = r.recognize(path)?;
        match *matcher.handler {
            "mapper_update_component" => self.mapper_update_component(ParamHolder(matcher.params), entity),
            "mapper_update_component_attribute" => self.mapper_update_component_attribute(ParamHolder(matcher.params), entity),
            "mapper_update_project_yaml" => self.mapper_update_project_yaml(ParamHolder(matcher.params), entity),
            _ => unreachable!("{}", path),
        }
    }

    fn delete(&mut self, path: &str) -> Result<(), String> {
        let r = self.server().router[&Method::Delete].clone();
        let matcher = r.recognize(path)?;
        match *matcher.handler {
            "mapper_delete_component" => self.mapper_delete_component(ParamHolder(matcher.params)),
            _ => unreachable!("{}", path),
        }
    }

    fn mapper_list_component(&self, _: ParamHolder) -> Result<serde_json::Value, String> {
        Ok(json!(self.component_repo().list()))
    }

    fn mapper_list_component_effect(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let component_id = params.find("component_id")?;
        let result = json!(self.component_repo().get(component_id).component().effect.iter().map(|effect_id| {
            self.effect_repo().get(effect_id)
        }).collect::<Vec<&Effect>>());
        Ok(result)
    }

    fn mapper_list_effect(&self, _: ParamHolder) -> Result<serde_json::Value, String> {
        Ok(json!(self.effect_repo().list()))
    }

    fn mapper_get_component(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let component_id = params.find("component_id")?;
        Ok(json!(self.component_repo().get(component_id)))
    }

    fn mapper_get_component_attribute(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let component_id = params.find("component_id")?;
        Ok(json!(self.component_repo().get(component_id).component().attributes[params.find("key")?]))
    }

    fn mapper_get_component_effect(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let component_id = params.find("component_id")?;
        let index: usize = params.find_as_usize("index")?;
        let effect_id = &self.component_repo().get(component_id).component().effect[index];
        Ok(json!(self.effect_repo().get(effect_id)))
    }

    fn mapper_get_effect(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let effect_id = params.find("effect_id")?;
        Ok(json!(self.effect_repo().get(effect_id)))
    }

    fn mapper_get_effect_value(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
        let effect_id = params.find("effect_id")?;
        let time = params.find_as_f32("time")?;
        Ok(json!(self.effect_repo().value(effect_id, time)))
    }

    fn mapper_get_screen(&self, params: ParamHolder) -> Result<serde_json::Value, String> {
       let time: u64 = params.find_as_u64("time")?;
        let encoded = base64::encode(&self.get_pixbuf(time * gst::MSECOND).save_to_bufferv("png", &[]).unwrap());
        Ok(json!(format!("data:image/png;base64,{}", encoded)))
    }

    fn mapper_get_project_yaml(&self, _: ParamHolder) -> Result<serde_json::Value, String> {
        Ok(json!(self.to_yaml_string().map_err(|t| t.to_string())?))
    }

    fn mapper_create_component(&mut self, _: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let key = self.component_repo_mut().create(<Self as HaveComponentRepository>::new_from_json(entity));
        self.project_mut().add_component_at(0, key);

        Ok(())
    }

    fn mapper_create_component_effect(&mut self, params: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let component_id = params.find("component_id")?;
        let effect_id = self.effect_repo_mut().create(serde_json::from_value(entity).map_err(|t| t.to_string())?).to_string();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().effect.push(effect_id);

        Ok(())
    }

    fn mapper_create_effect_intermed(&mut self, params: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let effect_id = params.find("effect_id").unwrap();
        self.effect_repo_mut().create_intermed(effect_id, serde_json::from_value(entity).unwrap());

        Ok(())
    }

    fn mapper_insert_component_effect(&mut self, params: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let component_id = params.find("component_id")?;
        let index = params.find_as_usize("index")?;
        let effect_id = self.effect_repo_mut().create(serde_json::from_value(entity).map_err(|t| t.to_string())?).to_string();
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().effect.insert(index, effect_id);

        Ok(())
    }

    fn mapper_delete_component(&mut self, params: ParamHolder) -> Result<(), String> {
        let component_id = params.find("component_id").unwrap();
        self.component_repo_mut().delete(component_id);

        Ok(())
    }

    fn mapper_update_component(&mut self, params: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let component_id = params.find("component_id")?;
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().partial_update(entity.as_object().unwrap());

        Ok(())
    }

    fn mapper_update_component_attribute(&mut self, params: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        let component_id = params.find("component_id")?;
        let key = params.find("key")?;
        let component = self.component_repo_mut().get_mut(component_id);
        component.component_mut().attributes.insert(key.to_string(), entity);

        Ok(())
    }

    fn mapper_update_project_yaml(&mut self, _: ParamHolder, entity: serde_json::Value) -> Result<(), String> {
        self.from_yaml_string(entity.as_str().unwrap()).unwrap();

        Ok(())
    }
}

