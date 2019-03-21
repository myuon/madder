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
        let appsink = gst::ElementFactory::make("appsink", Some("sink"))
            .ok_or(failure::err_msg("make appsink"))?;
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

    pub fn query_sample(sink: gst::Element) -> Result<gst::sample::Sample, failure::Error> {
        // Set to PAUSED
        sink.set_state(gst::State::Paused)?;
        sink.get_state(5 * gst::MSECOND).0?;

        let duration: gst::ClockTime = sink
            .query_duration()
            .ok_or(failure::err_msg("query_duration"))?;
        let position = duration
            .map(|d| d * 5 / 100 * gst::MSECOND)
            .unwrap_or(gst::SECOND);

        let mut flags = gst::SeekFlags::empty();
        flags.insert(gst::SeekFlags::KEY_UNIT);
        flags.insert(gst::SeekFlags::FLUSH);

        sink.seek_simple(flags, position)?;

        let appsink = sink
            .downcast::<gsta::AppSink>()
            .map_err(|_| failure::err_msg("downcast to AppSink"))?;

        appsink.pull_sample().ok_or(failure::err_msg("pull_sample"))
    }
}
