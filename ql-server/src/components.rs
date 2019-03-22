pub mod video;
pub mod video_test;

use video::VideoComponent;
use video_test::VideoTestComponent;

#[derive(Clone)]
pub enum Component {
    VideoComponent(video::VideoComponent),
    VideoTestComponent(video_test::VideoTestComponent),
}

graphql_union!(Component: () |&self| {
    instance_resolvers: |_| {
        &VideoComponent => match *self { Component::VideoComponent(ref c) => Some(c), _ => None },
        &VideoTestComponent => match *self { Component::VideoTestComponent(ref c) => Some(c), _ => None },
    }
});
