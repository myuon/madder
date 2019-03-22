use crate::components::*;
use crate::util::*;
use std::collections::HashMap;

#[derive(Clone, GraphQLObject)]
pub struct ScreenSize {
    width: U32,
    height: U32,
}

#[derive(Clone, GraphQLObject)]
pub struct Project {
    pub size: ScreenSize,
    length: ClockTime,
    position: ClockTime,
    components: Vec<Component>,
}

#[derive(Clone)]
pub struct Editor {
    pub project: Project,
    cache_appsink: HashMap<String, gsta::AppSink>,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            project: Project {
                size: ScreenSize {
                    width: U32::from_i32(1280),
                    height: U32::from_i32(720),
                },
                length: ClockTime::new(gst::ClockTime::from_seconds(1)),
                position: ClockTime::new(gst::ClockTime::from_seconds(0)),
                components: vec![],
            },
            cache_appsink: HashMap::new(),
        }
    }

    pub fn add_video_component(
        &mut self,
        start_time: ClockTime,
        uri: &str,
    ) -> video::VideoComponent {
        let loaded = video::VideoComponent::load(start_time, uri);
        self.cache_appsink
            .insert(loaded.component.id.clone(), loaded.appsink);
        self.project
            .components
            .push(Component::VideoComponent(loaded.component.clone()));

        loaded.component
    }

    pub fn add_video_test_component(
        &mut self,
        start_time: ClockTime,
    ) -> video_test::VideoTestComponent {
        let loaded = video_test::VideoTestComponent::load(start_time);
        self.cache_appsink
            .insert(loaded.component.id.clone(), loaded.appsink);
        self.project
            .components
            .push(Component::VideoTestComponent(loaded.component.clone()));

        loaded.component
    }

    pub fn query_pixbuf(&self) -> Result<Pixbuf, failure::Error> {
        let mut pixbuf = Pixbuf::new(
            self.project.size.width.as_u32(),
            self.project.size.height.as_u32(),
        );

        for (_, appsink) in &self.cache_appsink {
            pixbuf.copy_from(&video::VideoComponent::query_pixbuf(appsink)?, 0, 0)?;
        }

        Ok(pixbuf)
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}
