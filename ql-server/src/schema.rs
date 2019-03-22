extern crate gstreamer as gst;

use crate::components::{video, video_test, Component};
use crate::editor::*;
use crate::util::*;
use juniper::{FieldResult, RootNode};
use std::sync::RwLock;

pub struct Context(RwLock<Editor>);

impl juniper::Context for Context {}

impl Context {
    pub fn new() -> Context {
        Context(RwLock::new(Editor::new()))
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

    field pixbuf(&executor) -> FieldResult<String> {
        let context = executor.context();
        let madder = context.0.read()?;

        Ok(madder.query_pixbuf()?.to_png_base64_string())
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field setScreenSize(&executor, width: i32, height: i32) -> FieldResult<ScreenSize> {
        let context = executor.context();
        let mut madder = context.0.write()?;
        madder.project.size = ScreenSize {
            width: width as u32,
            height: height as u32,
        };

        Ok(ScreenSize {
            width: width as u32,
            height: height as u32,
        })
    }

    field newVideoComponent(&executor, startTime: i32, uri: String) -> FieldResult<video::VideoComponent> {
        let context = executor.context();
        let mut madder = context.0.write()?;
        let video_component = madder.add_video_component(ClockTime::new(gst::ClockTime::from_mseconds(startTime as u64)), &uri);

        Ok(video_component)
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
