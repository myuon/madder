use image::{ImageBuffer, RgbaImage};

pub struct Pixbuf(RgbaImage);

impl Pixbuf {
    pub fn new_from_gst_sample(sample: gst::sample::Sample) -> Result<Self, failure::Error> {
        let sample_ref = sample.as_ref();
        let buffer = sample_ref
            .get_buffer()
            .ok_or(failure::err_msg("get_buffer"))?;
        let caps = sample_ref.get_caps().ok_or(failure::err_msg("get_caps"))?;
        let structure_ref = caps
            .as_ref()
            .get_structure(0)
            .ok_or(failure::err_msg("get_structure"))?;

        let width = structure_ref
            .get::<i32>("width")
            .ok_or(failure::err_msg("get_value"))?;
        let height = structure_ref
            .get::<i32>("height")
            .ok_or(failure::err_msg("get_value"))?;

        let buffer_ref = buffer.as_ref();
        let data = buffer_ref
            .map_readable()
            .ok_or(failure::err_msg("map_readable"))?;

        Ok(Pixbuf(
            ImageBuffer::from_vec(width as u32, height as u32, data.to_vec())
                .ok_or(failure::err_msg("from_vec"))?,
        ))
    }
}
