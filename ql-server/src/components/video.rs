use crate::util::*;
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
    pub appsink: gsta::AppSink,
    pub element: gst::Element,
}

impl VideoComponent {
    fn create_pipeline(uri: &str) -> Result<(gsta::AppSink, gst::Element), failure::Error> {
        let pipeline = gst::Pipeline::new(None);
        let src =
            gst::ElementFactory::make("filesrc", None).ok_or(failure::err_msg("make filesrc"))?;
        let decodebin = gst::ElementFactory::make("decodebin", None)
            .ok_or(failure::err_msg("make decodebin"))?;
        let videoconvert = gst::ElementFactory::make("videoconvert", None)
            .ok_or(failure::err_msg("make videoconvert"))?;
        let sink = gst::ElementFactory::make("appsink", Some("sink"))
            .ok_or(failure::err_msg("make appsink"))?;
        src.set_property("location", &gst::Value::from(uri))?;

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

    pub fn load(start_time: ClockTime, uri: &str) -> ComponentRecord {
        let (appsink, sink) = VideoComponent::create_pipeline(uri).unwrap();

        ComponentRecord {
            component: VideoComponent {
                id: "<uuid>".to_string(),
                start_time,
                length: ClockTime::new(10 * gst::MSECOND),
                uri: uri.to_string(),
            },
            appsink,
            element: sink,
        }
    }

    pub fn query_seek(sink: &gst::Element) -> Result<(), failure::Error> {
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

        Ok(())
    }

    pub fn query_pixbuf(appsink: &gsta::AppSink) -> Result<Pixbuf, failure::Error> {
        let sample = appsink
            .pull_sample()
            .ok_or(failure::err_msg("pull_sample"))?;

        Pixbuf::new_from_gst_sample(sample)
    }
}
