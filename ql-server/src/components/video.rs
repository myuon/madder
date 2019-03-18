use crate::util::ClockTime;

#[derive(Clone, GraphQLObject)]
pub struct VideoComponent {
    pub id: String,
    pub start_time: ClockTime,
    pub length: ClockTime,
    pub uri: String,
}

pub struct ComponentRecord {
    pub component: VideoComponent,
    pub gst_element: gst::Element,
}

impl VideoComponent {
    fn create_pipeline(uri: &str) -> gst::Element {
        unimplemented!()
    }

    pub fn load(start_time: ClockTime, uri: &str) -> ComponentRecord {
        ComponentRecord {
            component: VideoComponent {
                id: "<uuid>".to_string(),
                start_time: start_time,
                length: ClockTime::new(10 * gst::MSECOND),
                uri: uri.to_string(),
            },
            gst_element: VideoComponent::create_pipeline(uri),
        }
    }
}

