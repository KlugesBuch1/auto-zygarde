use std::path::Path;

use image::ImageError;

pub struct Template {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Template {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = image::open(path)?.into_luma8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw(),
        })
    }
}
