extern crate gstreamer as gst;

pub trait Component {
    fn start_time(&self) -> gst::ClockTime;
    fn length(&self) -> gst::ClockTime;
    fn end_time(&self) -> gst::ClockTime {
        self.start_time() + self.length()
    }
}

pub trait HaveComponent {
    type COMPONENT : Component;
    fn component(&self) -> &Self::COMPONENT;
    fn component_mut(&mut self) -> &mut Self::COMPONENT;
}


