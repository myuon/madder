use image::{GenericImage, ImageBuffer, RgbaImage};

pub struct Pixbuf(RgbaImage);

impl Pixbuf {
    pub fn new(width: u32, height: u32) -> Self {
        Pixbuf(ImageBuffer::from_fn(width, height, |_, _| {
            image::Rgba([0, 0, 0, 255])
        }))
    }

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

    pub fn copy_from(&mut self, other: &Pixbuf, x: u32, y: u32) -> Result<(), failure::Error> {
        let result = self.0.copy_from(&other.0, x, y);
        if result {
            Ok(())
        } else {
            Err(failure::err_msg("copy failed"))
        }
    }

    pub fn to_png_base64_string(self) -> String {
        base64::encode(&self.0.into_vec())
    }
}
