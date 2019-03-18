use std::sync::RwLock;
use juniper::{FieldResult, RootNode};

#[derive(Clone, GraphQLObject)]
#[graphql(description = "Screen size")]
pub struct ScreenSize {
    width: i32,
    height: i32,
}

#[derive(GraphQLObject)]
#[graphql(description = "Madder object")]
pub struct Madder {
    size: ScreenSize,
}

impl Madder {
    pub fn new() -> Madder {
        Madder {
            size: ScreenSize {
                width: 1280,
                height: 720,
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
        let madder = context.0.read().unwrap();

        Ok(madder.size.clone())
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field setScreenSize(&executor, width: i32, height: i32) -> FieldResult<ScreenSize> {
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

