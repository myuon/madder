extern crate ql_server;

use std::iter::FromIterator;
use juniper::{Variables};
use ql_server::schema::{Schema, QueryRoot, MutationRoot, Context};

#[test]
fn default_screen_size_should_be_1280x720() {
    let (res, _errors) = juniper::execute(
        "query { screenSize { width height } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &Context::new(),
    ).unwrap();

    assert_eq!(
        res,
        juniper::Value::object(
            juniper::Object::from_iter(vec![
                ("screenSize", juniper::Value::object(juniper::Object::from_iter(vec![
                    ("width", juniper::Value::scalar(1280)),
                    ("height", juniper::Value::scalar(720)),
                ])))
            ])
        )
    );
    assert_eq!(_errors.len(), 0);
}

