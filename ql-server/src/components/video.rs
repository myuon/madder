use crate::util::ClockTime;
use gst::prelude::*;

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
    fn create_pipeline(uri: &str) -> Result<gst::Element, failure::Error> {
        let pipeline = gst::Pipeline::new(None);
        let src =
            gst::ElementFactory::make("filesrc", None).ok_or(failure::err_msg("make filesrc"))?;
        let decodebin = gst::ElementFactory::make("decodebin", None)
            .ok_or(failure::err_msg("make decodebin"))?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)
            .ok_or(failure::err_msg("make videoconvert"))?;
        let appsink =
            gst::ElementFactory::make("appsink", None).ok_or(failure::err_msg("make appsink"))?;
        src.set_property("location", &gst::Value::from(uri))?;

        pipeline.add_many(&[&src, &decodebin, &videoconvert, &appsink])?;

        Ok(appsink)
    }

    pub fn load(start_time: ClockTime, uri: &str) -> ComponentRecord {
        ComponentRecord {
            component: VideoComponent {
                id: "<uuid>".to_string(),
                start_time,
                length: ClockTime::new(10 * gst::MSECOND),
                uri: uri.to_string(),
            },
            gst_element: VideoComponent::create_pipeline(uri).unwrap(),
        }
    }
}
