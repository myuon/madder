extern crate gstreamer as gst;

use std::sync::RwLock;
use juniper::{FieldResult, RootNode, GraphQLType, Registry};
use juniper::meta::MetaType;

#[derive(Clone, GraphQLObject)]
#[graphql(description = "Screen size")]
pub struct ScreenSize {
    width: i32,
    height: i32,
}

pub struct ClockTime(gst::ClockTime);

impl<S> GraphQLType<S> for ClockTime where
    S: juniper::ScalarValue,
    for<'b> &'b S: juniper::ScalarRefValue<'b>
{
    type Context = ();
    type TypeInfo = ();

    fn name(_: &()) -> Option<&'static str> {
        Some("ClockTime")
    }

    fn meta<'r>(_: &(), registry: &mut Registry<'r, S>) -> MetaType<'r, S>
        where S: 'r
    {
        let fields = &[
            registry.field::<&i32>("0", &())
        ];
        let builder = registry.build_object_type::<Project>(&(), fields);
        let builder = builder.description("ClockTime");
        builder.into_meta()
    }
}

#[derive(GraphQLObject)]
pub struct Project {
    size: ScreenSize,
    length: ClockTime,
}

#[derive(GraphQLObject)]
#[graphql(description = "Madder object")]
pub struct Madder {
    project: Project,
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            project: Project {
                size: ScreenSize {
                    width: 1280,
                    height: 720,
                },
                length: ClockTime(1 * gst::SECOND),
            }
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

