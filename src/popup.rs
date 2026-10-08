use std::path::Path;

use image::imageops::FilterType;

use crate::config::{
    ICON_ALPHA_MIN, MATCH_MIN_SIDE, MATCH_MIN_SIMILARITY, MATCH_SCALES, MATCH_TOP,
    MATCH_WORK_WIDTH, POPUP_GOAL,
};
use crate::locate::locate_opaque;
use crate::{Frame, Template};

pub struct Icon {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Icon {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let image = image::open(path)?.into_rgba8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        })
    }
}

pub struct Route {
    popups: u32,
}

impl Route {
    pub fn new() -> Self {
        Self { popups: 0 }
    }

    pub fn popups(&self) -> u32 {
        self.popups
    }

    pub fn after_click(&mut self, popup: bool) -> bool {
        if popup {
            self.popups += 1;
        }
        self.popups >= POPUP_GOAL
    }
}

pub fn shows_cell_popup(icon: &Icon, frame: &Frame) -> bool {
    shows_icon_rows(icon, frame, 0.0, MATCH_TOP)
}

pub fn shows_icon_rows(icon: &Icon, frame: &Frame, from_y: f32, to_y: f32) -> bool {
    let Some(band) = band_luma(frame, from_y, to_y) else {
        return false;
    };
    matches(icon, &band, &MATCH_SCALES, MATCH_MIN_SIMILARITY)
}

fn band_luma(frame: &Frame, from_y: f32, to_y: f32) -> Option<Template> {
    let width = frame.width;
    let height = frame.height;
    if width == 0 || height == 0 {
        return None;
    }
    let expected = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(3)?;
    if frame.pixels.len() != expected {
        return None;
    }
    let y0 = ((height as f32) * from_y).floor() as u32;
    let y1 = ((height as f32) * to_y).ceil() as u32;
    let y0 = y0.min(height - 1);
    let y1 = y1.clamp(y0 + 1, height);
    let rows = y1 - y0;
    let start = (y0 as usize) * (width as usize) * 3;
    let bytes = (width as usize) * (rows as usize) * 3;
    let rgb = image::RgbImage::from_raw(width, rows, frame.pixels[start..start + bytes].to_vec())?;
    let gray = image::DynamicImage::ImageRgb8(rgb).into_luma8();
    Some(Template {
        width: gray.width(),
        height: gray.height(),
        pixels: gray.into_raw(),
    })
}

fn matches(icon: &Icon, band: &Template, scales: &[f32], min_similarity: f32) -> bool {
    let Some((band, ratio)) = fit_band(band) else {
        return false;
    };
    for scale in scales {
        let width = ((icon.width as f32) * scale * ratio).round() as u32;
        let height = ((icon.height as f32) * scale * ratio).round() as u32;
        if width < MATCH_MIN_SIDE
            || height < MATCH_MIN_SIDE
            || width > band.width
            || height > band.height
        {
            continue;
        }
        let Some((needle, opaque)) = scale_icon(icon, width, height) else {
            continue;
        };
        if locate_opaque(&needle, &band, &opaque, min_similarity).is_some() {
            return true;
        }
    }
    false
}

fn scale_icon(icon: &Icon, width: u32, height: u32) -> Option<(Template, Vec<bool>)> {
    if width == 0 || height == 0 {
        return None;
    }
    let image = image::RgbaImage::from_raw(icon.width, icon.height, icon.rgba.clone())?;
    let out = image::imageops::resize(&image, width, height, FilterType::Triangle);
    let mut pixels = Vec::with_capacity((width * height) as usize);
    let mut opaque = Vec::with_capacity((width * height) as usize);
    for pixel in out.pixels() {
        pixels.push(luma(pixel[0], pixel[1], pixel[2]));
        opaque.push(pixel[3] >= ICON_ALPHA_MIN);
    }
    Some((
        Template {
            width,
            height,
            pixels,
        },
        opaque,
    ))
}

fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((u32::from(r) * 2126 + u32::from(g) * 7152 + u32::from(b) * 722) / 10000) as u8
}

fn fit_band(band: &Template) -> Option<(Template, f32)> {
    if band.width <= MATCH_WORK_WIDTH {
        return Some((
            Template {
                width: band.width,
                height: band.height,
                pixels: band.pixels.clone(),
            },
            1.0,
        ));
    }
    let ratio = MATCH_WORK_WIDTH as f32 / band.width as f32;
    let height = ((band.height as f32) * ratio).round().max(1.0) as u32;
    Some((resize(band, MATCH_WORK_WIDTH, height)?, ratio))
}

fn resize(src: &Template, width: u32, height: u32) -> Option<Template> {
    if width == 0 || height == 0 {
        return None;
    }
    let image = image::GrayImage::from_raw(src.width, src.height, src.pixels.clone())?;
    let out = image::imageops::resize(&image, width, height, FilterType::Triangle);
    Some(Template {
        width: out.width(),
        height: out.height(),
        pixels: out.into_raw(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(gray: Template) -> Icon {
        let mut rgba = Vec::with_capacity(gray.pixels.len() * 4);
        for value in gray.pixels {
            rgba.extend([value, value, value, 255]);
        }
        Icon {
            width: gray.width,
            height: gray.height,
            rgba,
        }
    }

    fn pattern() -> Template {
        let (width, height) = (16, 16);
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                pixels.push((x * 13 + y * 29) as u8);
            }
        }
        Template {
            width,
            height,
            pixels,
        }
    }

    fn paste(dst: &mut Template, src: &Template, x0: u32, y0: u32) {
        for y in 0..src.height {
            for x in 0..src.width {
                let di = ((y0 + y) * dst.width + x0 + x) as usize;
                let si = (y * src.width + x) as usize;
                dst.pixels[di] = src.pixels[si];
            }
        }
    }

    #[test]
    fn finds_smaller_icon() {
        let icon = solid(pattern());
        let (small, _) = scale_icon(&icon, 8, 8).unwrap();
        let mut band = Template {
            width: 40,
            height: 24,
            pixels: vec![0; 40 * 24],
        };
        paste(&mut band, &small, 3, 2);
        assert!(matches(&icon, &band, &[0.5], 0.9));
    }

    #[test]
    fn finds_icon_only_in_the_top() {
        let icon = solid(pattern());
        let (small, _) = scale_icon(&icon, 8, 8).unwrap();
        let mut frame = Frame {
            width: 40,
            height: 80,
            pixels: vec![0; 40 * 80 * 3],
        };
        paint(&mut frame, &small, 3, 2);
        assert!(shows_cell_popup(&icon, &frame));
        let mut lower = Frame {
            width: 40,
            height: 80,
            pixels: vec![0; 40 * 80 * 3],
        };
        paint(&mut lower, &small, 3, 60);
        assert!(!shows_cell_popup(&icon, &lower));
    }

    fn paint(frame: &mut Frame, src: &Template, x0: u32, y0: u32) {
        for y in 0..src.height {
            for x in 0..src.width {
                let px = (((y0 + y) * frame.width + x0 + x) * 3) as usize;
                let v = src.pixels[(y * src.width + x) as usize];
                frame.pixels[px] = v;
                frame.pixels[px + 1] = v;
                frame.pixels[px + 2] = v;
            }
        }
    }

    #[test]
    fn ignores_transparent_border() {
        let mut rgba = vec![0u8; 8 * 8 * 4];
        for y in 2..6 {
            for x in 2..6 {
                let index = (y * 8 + x) * 4;
                let value = (x * 13 + y * 29) as u8;
                rgba[index] = value;
                rgba[index + 1] = value;
                rgba[index + 2] = value;
                rgba[index + 3] = 255;
            }
        }
        let icon = Icon {
            width: 8,
            height: 8,
            rgba,
        };
        let (small, opaque) = scale_icon(&icon, 8, 8).unwrap();
        let mut band = Template {
            width: 20,
            height: 20,
            pixels: vec![180; 20 * 20],
        };
        for y in 0..small.height {
            for x in 0..small.width {
                if !opaque[(y * small.width + x) as usize] {
                    continue;
                }
                let di = ((y + 2) * band.width + x + 2) as usize;
                band.pixels[di] = small.pixels[(y * small.width + x) as usize];
            }
        }
        assert!(matches(&icon, &band, &[1.0], 0.9));
    }

    #[test]
    fn counts_three_popups() {
        let mut route = Route::new();
        assert!(!route.after_click(false));
        assert!(!route.after_click(true));
        assert_eq!(route.popups(), 1);
        assert!(!route.after_click(false));
        assert!(!route.after_click(true));
        assert!(route.after_click(true));
        assert_eq!(route.popups(), POPUP_GOAL);
    }
}
