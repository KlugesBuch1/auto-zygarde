use std::path::Path;

use image::ImageError;

use crate::config::{
    GLOW_HUE_MAX, GLOW_HUE_MIN, GLOW_MAX_AREA, GLOW_MIN_AREA, GLOW_SAT_MIN, GLOW_VAL_MIN,
};

const NEIGHBORS: [(isize, isize); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

struct Blob {
    sum_x: u64,
    sum_y: u64,
    sum_v: u64,
    area: u32,
}

impl Frame {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ImageError> {
        let image = image::open(path)?.into_rgb8();
        Ok(Self {
            width: image.width(),
            height: image.height(),
            pixels: image.into_raw(),
        })
    }
}

impl Blob {
    fn mean_v(&self) -> u64 {
        self.sum_v / u64::from(self.area)
    }
}

pub fn find_cell(frame: &Frame) -> Option<Point> {
    let width = frame.width as usize;
    let height = frame.height as usize;
    if width == 0
        || height == 0
        || frame.pixels.len() != width.saturating_mul(height).saturating_mul(3)
    {
        return None;
    }

    let mut value = vec![0u8; width * height];
    for i in 0..value.len() {
        let pixel = i * 3;
        let (hue, saturation, brightness) = rgb_to_hsv(
            frame.pixels[pixel],
            frame.pixels[pixel + 1],
            frame.pixels[pixel + 2],
        );
        if (GLOW_HUE_MIN..=GLOW_HUE_MAX).contains(&hue)
            && saturation > GLOW_SAT_MIN
            && brightness > GLOW_VAL_MIN
        {
            value[i] = brightness;
        }
    }

    let mut seen = vec![false; value.len()];
    let mut best: Option<Blob> = None;
    for start in 0..value.len() {
        if value[start] == 0 || seen[start] {
            continue;
        }
        let mut blob = Blob {
            sum_x: 0,
            sum_y: 0,
            sum_v: 0,
            area: 0,
        };
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(index) = stack.pop() {
            let x = index % width;
            let y = index / width;
            blob.sum_x += x as u64;
            blob.sum_y += y as u64;
            blob.sum_v += u64::from(value[index]);
            blob.area += 1;
            for (dx, dy) in NEIGHBORS {
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= height as isize {
                    continue;
                }
                let next = ny as usize * width + nx as usize;
                if value[next] == 0 || seen[next] {
                    continue;
                }
                seen[next] = true;
                stack.push(next);
            }
        }
        if blob.area < GLOW_MIN_AREA || blob.area > GLOW_MAX_AREA {
            continue;
        }
        let keep = match &best {
            None => true,
            Some(current) => {
                blob.mean_v() > current.mean_v()
                    || (blob.mean_v() == current.mean_v() && blob.area > current.area)
            }
        };
        if keep {
            best = Some(blob);
        }
    }

    best.map(|blob| Point {
        x: ((blob.sum_x as f32) / (blob.area as f32)).round() as u32,
        y: ((blob.sum_y as f32) / (blob.area as f32)).round() as u32,
    })
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = i32::from(r);
    let g = i32::from(g);
    let b = i32::from(b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let brightness = max as u8;
    let saturation = if max == 0 {
        0
    } else {
        (delta * 255 / max) as u8
    };
    if delta == 0 {
        return (0, saturation, brightness);
    }
    let mut hue = if max == r {
        60 * (g - b) / delta
    } else if max == g {
        60 * (b - r) / delta + 120
    } else {
        60 * (r - g) / delta + 240
    };
    if hue < 0 {
        hue += 360;
    }
    ((hue / 2) as u8, saturation, brightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint(frame: &mut Frame, x: u32, y: u32, rgb: [u8; 3]) {
        let index = ((y * frame.width + x) * 3) as usize;
        frame.pixels[index..index + 3].copy_from_slice(&rgb);
    }

    #[test]
    fn finds_brightest_neon_blob() {
        let mut frame = Frame {
            width: 150,
            height: 150,
            pixels: vec![0; 150 * 150 * 3],
        };
        for y in 0..140 {
            for x in 0..frame.width {
                paint(&mut frame, x, y, [0, 255, 0]);
            }
        }
        paint(&mut frame, 0, 142, [0, 255, 0]);
        paint(&mut frame, 21, 144, [0, 80, 0]);
        for y in 145..149 {
            for x in 20..24 {
                paint(&mut frame, x, y, [0, 255, 0]);
            }
        }

        let point = find_cell(&frame).unwrap();
        assert_eq!((point.x, point.y), (22, 147));
    }
}
