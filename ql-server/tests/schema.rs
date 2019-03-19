extern crate ql_server;

use juniper::Variables;
use ql_server::schema::{Context, MutationRoot, QueryRoot, Schema};
use std::iter::FromIterator;

#[test]
fn default_screen_size_should_be_1280x720() {
    let (res, _errors) = juniper::execute(
        "query { screenSize { width height } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &Context::new(),
    )
    .unwrap();

    assert_eq!(
        res,
        juniper::Value::object(juniper::Object::from_iter(vec![(
            "screenSize",
            juniper::Value::object(juniper::Object::from_iter(vec![
                ("width", juniper::Value::scalar(1280)),
                ("height", juniper::Value::scalar(720)),
            ]))
        )]))
    );
    assert_eq!(_errors.len(), 0);
}

#[test]
fn can_update_screen_size() {
    let new_size = (200, 150);
    let context = Context::new();

    let (res, _errors) = juniper::execute(
        &format!(
            "mutation {{ setScreenSize(width: {}, height: {}) {{ width height }} }}",
            new_size.0, new_size.1
        ),
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &context,
    )
    .unwrap();

    assert_eq!(
        res,
        juniper::Value::object(juniper::Object::from_iter(vec![(
            "setScreenSize",
            juniper::Value::object(juniper::Object::from_iter(vec![
                ("width", juniper::Value::scalar(new_size.0)),
                ("height", juniper::Value::scalar(new_size.1)),
            ]))
        )]))
    );
    assert_eq!(_errors.len(), 0);

    let (res, _errors) = juniper::execute(
        "query { screenSize { width height } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &context,
    )
    .unwrap();

    assert_eq!(
        res,
        juniper::Value::object(juniper::Object::from_iter(vec![(
            "screenSize",
            juniper::Value::object(juniper::Object::from_iter(vec![
                ("width", juniper::Value::scalar(new_size.0)),
                ("height", juniper::Value::scalar(new_size.1)),
            ]))
        )]))
    );
    assert_eq!(_errors.len(), 0);
}
