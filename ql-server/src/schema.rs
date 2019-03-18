use juniper::{FieldResult, RootNode};

#[derive(GraphQLObject)]
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

pub struct QueryRoot;

graphql_object!(QueryRoot: () |&self| {
    field screenSize(&executor) -> FieldResult<ScreenSize> {
        Ok(ScreenSize {
            width: 1280,
            height: 720,
        })
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: () |&self| {
    field setScreenSize(width: i32, height: i32) -> FieldResult<ScreenSize> {
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

