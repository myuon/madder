extern crate gstreamer as gst;

use crate::components::video;
use crate::util::ClockTime;
use juniper::{FieldResult, RootNode};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone, GraphQLObject)]
#[graphql(description = "Screen size")]
pub struct ScreenSize {
    width: i32,
    height: i32,
}

#[derive(Clone, GraphQLObject)]
pub struct Project {
    size: ScreenSize,
    length: ClockTime,
    position: ClockTime,
    components: Vec<video::VideoComponent>,
}

#[derive(Clone)]
pub struct Madder {
    project: Project,
    gst_elements: HashMap<String, gst::Element>,
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project {
                size: ScreenSize {
                    width: 1280,
                    height: 720,
                },
                length: ClockTime::new(gst::ClockTime::from_seconds(1)),
                position: ClockTime::new(gst::ClockTime::from_seconds(0)),
                components: vec![],
            },
            gst_elements: HashMap::new(),
        }
    }

    pub fn add_video_component(&mut self, start_time: ClockTime, uri: &str) {
        let loaded = video::VideoComponent::load(start_time, uri);
        self.gst_elements
            .insert(loaded.component.id.clone(), loaded.gst_element);
        self.project.components.push(loaded.component);
    }
}

impl Default for Madder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Context(RwLock<Madder>);

impl juniper::Context for Context {}

impl Context {
    pub fn new() -> Context {
        Context(RwLock::new(Madder::new()))
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {
    field project(&executor) -> FieldResult<Project> {
        let context = executor.context();
        let madder = context.0.read()?;

        Ok(madder.project.clone())
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field setScreenSize(&executor, width: i32, height: i32) -> FieldResult<ScreenSize> {
        let context = executor.context();
        let mut madder = context.0.write()?;
        madder.project.size = ScreenSize {
            width: width,
            height: height,
        };

        Ok(ScreenSize {
            width: width,
            height: height,
        })
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
