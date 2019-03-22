use crate::util::*;
use gst::prelude::*;

#[derive(Clone, GraphQLObject)]
pub struct VideoTestComponent {
    pub id: String,
    pub start_time: ClockTime,
    pub length: ClockTime,
}

pub struct ComponentRecord {
    pub component: VideoTestComponent,
    pub appsink: gsta::AppSink,
    pub element: gst::Element,
}

impl VideoTestComponent {
    fn create_pipeline() -> Result<(gsta::AppSink, gst::Element), failure::Error> {
        let pipeline = gst::Pipeline::new(None);
        let src = gst::ElementFactory::make("videotestsrc", None)
            .ok_or(failure::err_msg("make videotestsrc"))?;
        let decodebin = gst::ElementFactory::make("decodebin", None)
            .ok_or(failure::err_msg("make decodebin"))?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)
            .ok_or(failure::err_msg("make videoconvert"))?;
        let sink = gst::ElementFactory::make("appsink", Some("sink"))
            .ok_or(failure::err_msg("make appsink"))?;

        pipeline.add_many(&[&src, &decodebin, &videoconvert, &sink])?;
        gst::Element::link_many(&[&src, &decodebin])?;
        gst::Element::link_many(&[&videoconvert, &sink])?;

        decodebin.connect_pad_added(move |_, src_pad| {
            let sink_pad = videoconvert.get_static_pad("sink").unwrap();
            src_pad.link(&sink_pad).unwrap();
        });

        let appsink = sink
            .clone()
            .dynamic_cast::<gsta::AppSink>()
            .map_err(|_| failure::err_msg("dynamic_cast to AppSink"))?;
        appsink.set_caps(&gst::Caps::new_simple(
            "audit/x-raw",
            &[("format", &gstv::VideoFormat::Rgba)],
        ));

        Ok((appsink, sink))
    }

    pub fn new(start_time: ClockTime) -> ComponentRecord {
        let (appsink, sink) = VideoTestComponent::create_pipeline().unwrap();

        ComponentRecord {
            component: VideoTestComponent {
                id: uuid::Uuid::new_v4().to_string(),
                start_time,
                length: ClockTime::new(10 * gst::MSECOND),
            },
            appsink,
            element: sink,
        }
    }

    pub fn query_seek(sink: &gst::Element) -> Result<(), failure::Error> {
        Ok(())
    }

    pub fn query_pixbuf(appsink: &gsta::AppSink) -> Result<Pixbuf, failure::Error> {
        let sample = appsink
            .pull_sample()
            .ok_or(failure::err_msg("pull_sample"))?;

        Pixbuf::new_from_gst_sample(sample)
    }
}
