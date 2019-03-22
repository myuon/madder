use crate::util::*;

pub trait ComponentObject {
    fn id(&self) -> &String;
    fn start_time(&self) -> &ClockTime;
    fn length(&self) -> &ClockTime;
}

pub mod video;
pub mod video_test;

#[derive(Clone)]
pub enum Component {
    VideoComponent(video::VideoComponent),
    VideoTestComponent(video_test::VideoTestComponent),
}

impl ComponentObject for Component {
    fn id(&self) -> &String {
        use Component::*;

        match self {
            VideoComponent(c) => &c.id,
            VideoTestComponent(c) => &c.id,
        }
    }

    fn start_time(&self) -> &ClockTime {
        use Component::*;

        match self {
            VideoComponent(c) => &c.start_time,
            VideoTestComponent(c) => &c.start_time,
        }
    }

    fn length(&self) -> &ClockTime {
        use Component::*;

        match self {
            VideoComponent(c) => &c.length,
            VideoTestComponent(c) => &c.length,
        }
    }
}

graphql_object!(Component: () |&self| {
    field id() -> &String {
        self.id()
    }

    field start_time() -> &ClockTime {
        self.start_time()
    }

    field length() -> &ClockTime {
        self.length()
    }
});
