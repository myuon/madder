extern crate gstreamer as gst;

use crate::components::*;
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
    field project(&executor) -> FieldResult<ProjectInfo> {
        let context = executor.context();
        let editor = context.0.read()?;

        Ok(editor.project.clone())
    }

    field component(&executor, id: String) -> FieldResult<Component> {
        let context = executor.context();
        let editor = context.0.read()?;

        Ok(editor.components.get(&id).unwrap().clone())
    }

    field pixbuf(&executor) -> FieldResult<String> {
        let context = executor.context();
        let editor = context.0.read()?;

        Ok(editor.query_pixbuf()?.to_png_base64_string())
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field setScreenSize(&executor, width: i32, height: i32) -> FieldResult<ScreenSize> {
        let context = executor.context();
        let mut editor = context.0.write()?;
        editor.project.size = ScreenSize {
            width: U32::from_i32(width),
            height: U32::from_i32(height),
        };

        Ok(editor.project.size.clone())
    }

    field newVideoComponent(&executor, startTime: i32, uri: String) -> FieldResult<String> {
        let context = executor.context();
        let mut editor = context.0.write()?;
        let video_component = editor.add_video_component(ClockTime::new(gst::ClockTime::from_mseconds(startTime as u64)), &uri);

        Ok(video_component.id)
    }

    field newVideoTestComponent(&executor, startTime: i32) -> FieldResult<String> {
        let context = executor.context();
        let mut editor = context.0.write()?;
        let video_component = editor.add_video_test_component(ClockTime::new(gst::ClockTime::from_mseconds(startTime as u64)));

        Ok(video_component.id)
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
