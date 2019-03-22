pub mod video;
pub mod video_test;

#[derive(Clone)]
pub enum Component {
    VideoComponent(video::VideoComponent),
    VideoTestComponent(video_test::VideoTestComponent),
}

graphql_object!(Component: () |&self| {
    field type() -> &str {
        use Component::*;

        match self {
            VideoComponent(_) => "video_component",
            VideoTestComponent(_) => "video_test_component",
        }
    }
});
