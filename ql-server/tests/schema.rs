extern crate ql_server;
#[macro_use]
extern crate juniper;
extern crate gstreamer as gst;

use juniper::Variables;
use ql_server::schema::{Context, MutationRoot, QueryRoot, Schema};

#[test]
fn default_screen_size_should_be_1280x720() {
    let (res, _errors) = juniper::execute(
        "query { project { size { width height } } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &Context::new(),
    )
    .unwrap();

    assert_eq!(
        res,
        graphql_value!({
            "project": {
                "size": {
                    "width": 1280,
                    "height": 720,
                }
            }
        })
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

    let w = new_size.0;
    let h = new_size.1;
    assert_eq!(
        res,
        graphql_value!({
            "setScreenSize": {
                "width": w,
                "height": h,
            },
        })
    );
    assert_eq!(_errors.len(), 0);

    let (res, _errors) = juniper::execute(
        "query { project { size { width height } } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &context,
    )
    .unwrap();

    assert_eq!(
        res,
        graphql_value!({
            "project": {
                "size": {
                    "width": w,
                    "height": h,
                }
            }
        })
    );
    assert_eq!(_errors.len(), 0);
}

#[test]
fn can_add_video_component() {
    gst::init().unwrap();

    let context = Context::new();

    let (_res, errors) = juniper::execute(
        "mutation { newComponent(startTime: 0, uri: \"\") { id } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &context,
    )
    .unwrap();
    assert_eq!(errors.len(), 0);

    let (res, errors) = juniper::execute(
        "query { project { components { id } } }",
        None,
        &Schema::new(QueryRoot, MutationRoot),
        &Variables::new(),
        &context,
    )
    .unwrap();
    assert_eq!(
        res.as_object_value()
            .unwrap()
            .get_field_value("project")
            .unwrap()
            .as_object_value()
            .unwrap()
            .get_field_value("components")
            .unwrap()
            .as_list_value()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(errors.len(), 0);
}
