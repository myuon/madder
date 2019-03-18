use juniper::{Registry, GraphQLType};
use juniper::meta::MetaType;

#[derive(Clone)]
pub struct ClockTime(gst::ClockTime);

impl ClockTime {
    pub fn new(time: gst::ClockTime) -> ClockTime {
        ClockTime(time)
    }
}

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
        let builder = registry.build_object_type::<ClockTime>(&(), fields);
        let builder = builder.description("ClockTime");
        builder.into_meta()
    }
}

