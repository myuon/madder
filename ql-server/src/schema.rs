extern crate gstreamer as gst;

use std::sync::RwLock;
use std::collections::HashMap;
use juniper::{FieldResult, RootNode};
use crate::util::ClockTime;

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
                length: ClockTime::new(1 * gst::SECOND),
                position: ClockTime::new(0 * gst::SECOND),
            },
            gst_elements: HashMap::new(),
        }
    }
}

pub struct Context(RwLock<Madder>);

impl juniper::Context for Context {}

impl Context {
    pub fn new() -> Context {
        Context(RwLock::new(Madder::new()))
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {
    field screenSize(&executor) -> FieldResult<ScreenSize> {
        let context = executor.context();
        let madder = context.0.read()?;

        Ok(madder.project.size.clone())
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

