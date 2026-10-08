use crate::{Frame, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn find_text_in_region(frame: &Frame, target_text: &str, region: Rect) -> Option<Point> {
    let binary = binarize(frame, region)?;
    let read = read_glyphs(&binary);
    if !same_text(&read, target_text) {
        return None;
    }
    let (sum_x, sum_y, count) = ink_center(&binary);
    if count == 0 {
        return None;
    }
    Some(Point {
        x: region.x + ((sum_x as f32) / (count as f32)).round() as u32,
        y: region.y + ((sum_y as f32) / (count as f32)).round() as u32,
    })
}

struct Binary {
    width: usize,
    height: usize,
    ink: Vec<bool>,
}

fn binarize(frame: &Frame, region: Rect) -> Option<Binary> {
    let frame_w = frame.width as usize;
    let frame_h = frame.height as usize;
    if frame_w == 0 || frame_h == 0 || region.width == 0 || region.height == 0 {
        return None;
    }
    if frame.pixels.len() != frame_w * frame_h * 3 {
        return None;
    }
    let x0 = region.x as usize;
    let y0 = region.y as usize;
    let width = region.width as usize;
    let height = region.height as usize;
    if x0 + width > frame_w || y0 + height > frame_h {
        return None;
    }

    let mut gray = Vec::with_capacity(width * height);
    for y in y0..y0 + height {
        for x in x0..x0 + width {
            let i = (y * frame_w + x) * 3;
            gray.push(luma(
                frame.pixels[i],
                frame.pixels[i + 1],
                frame.pixels[i + 2],
            ));
        }
    }
    let threshold = otsu(&gray).clamp(1, 254);
    let light = gray.iter().filter(|value| **value >= threshold).count();
    let ink_is_light = light * 2 <= gray.len();
    let ink = gray
        .into_iter()
        .map(|value| {
            if ink_is_light {
                value >= threshold
            } else {
                value < threshold
            }
        })
        .collect();
    Some(Binary { width, height, ink })
}

fn luma(r: u8, g: u8, b: u8) -> u8 {
    ((u32::from(r) * 2126 + u32::from(g) * 7152 + u32::from(b) * 722) / 10000) as u8
}

fn otsu(gray: &[u8]) -> u8 {
    let mut hist = [0u32; 256];
    for value in gray {
        hist[*value as usize] += 1;
    }
    let total = gray.len() as f32;
    let mut sum_all = 0.0;
    for (value, count) in hist.iter().enumerate() {
        sum_all += value as f32 * *count as f32;
    }
    let mut sum_bg = 0.0;
    let mut weight_bg = 0.0;
    let mut best_var = -1.0;
    let mut best = 128u8;
    for value in 0..256 {
        weight_bg += hist[value] as f32;
        if weight_bg == 0.0 {
            continue;
        }
        let weight_fg = total - weight_bg;
        if weight_fg == 0.0 {
            break;
        }
        sum_bg += value as f32 * hist[value] as f32;
        let mean_bg = sum_bg / weight_bg;
        let mean_fg = (sum_all - sum_bg) / weight_fg;
        let var = weight_bg * weight_fg * (mean_bg - mean_fg) * (mean_bg - mean_fg);
        if var > best_var {
            best_var = var;
            best = value as u8;
        }
    }
    best
}

fn read_glyphs(binary: &Binary) -> String {
    let mut cols = vec![false; binary.width];
    for x in 0..binary.width {
        cols[x] = (0..binary.height).any(|y| binary.ink[y * binary.width + x]);
    }
    let mut text = String::new();
    let mut x = 0;
    let mut gap = 0;
    while x < binary.width {
        if !cols[x] {
            gap += 1;
            x += 1;
            continue;
        }
        if gap >= 6 && !text.is_empty() {
            text.push(' ');
        }
        gap = 0;
        let start = x;
        while x < binary.width && cols[x] {
            x += 1;
        }
        if let Some(ch) = match_glyph(binary, start, x) {
            text.push(ch);
        }
    }
    text
}

fn match_glyph(binary: &Binary, x0: usize, x1: usize) -> Option<char> {
    let mut y0 = None;
    let mut y1 = 0;
    for y in 0..binary.height {
        if (x0..x1).any(|x| binary.ink[y * binary.width + x]) {
            if y0.is_none() {
                y0 = Some(y);
            }
            y1 = y + 1;
        }
    }
    let y0 = y0?;
    let grid = sample(binary, x0, x1, y0, y1);
    let mut best = None;
    let mut best_miss = usize::MAX;
    for ch in 'A'..='Z' {
        let Some(rows) = font(ch) else {
            continue;
        };
        let mut miss = 0;
        for row in 0..7 {
            for col in 0..5 {
                let on = (rows[row] >> (4 - col)) & 1 == 1;
                if on != grid[row * 5 + col] {
                    miss += 1;
                }
            }
        }
        if miss < best_miss {
            best_miss = miss;
            best = Some(ch);
        }
    }
    if best_miss <= 4 { best } else { None }
}

fn sample(binary: &Binary, x0: usize, x1: usize, y0: usize, y1: usize) -> [bool; 35] {
    let mut grid = [false; 35];
    let gw = (x1 - x0) as f32;
    let gh = (y1 - y0) as f32;
    for row in 0..7 {
        for col in 0..5 {
            let sx0 = x0 + ((col as f32) * gw / 5.0).floor() as usize;
            let sx1 = x0 + (((col + 1) as f32) * gw / 5.0).ceil() as usize;
            let sy0 = y0 + ((row as f32) * gh / 7.0).floor() as usize;
            let sy1 = y0 + (((row + 1) as f32) * gh / 7.0).ceil() as usize;
            let mut ink = false;
            for y in sy0..sy1.min(binary.height) {
                for x in sx0..sx1.min(binary.width) {
                    if binary.ink[y * binary.width + x] {
                        ink = true;
                    }
                }
            }
            grid[row * 5 + col] = ink;
        }
    }
    grid
}

fn ink_center(binary: &Binary) -> (u64, u64, u64) {
    let mut sum_x = 0u64;
    let mut sum_y = 0u64;
    let mut count = 0u64;
    for y in 0..binary.height {
        for x in 0..binary.width {
            if binary.ink[y * binary.width + x] {
                sum_x += x as u64;
                sum_y += y as u64;
                count += 1;
            }
        }
    }
    (sum_x, sum_y, count)
}

fn same_text(read: &str, target: &str) -> bool {
    let read = read.to_ascii_uppercase();
    let target = target.to_ascii_uppercase();
    !target.is_empty() && read.replace(' ', "") == target.replace(' ', "")
}

fn font(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint(frame: &mut Frame, text: &str, origin_x: u32, origin_y: u32) {
        let scale = 2u32;
        let mut cursor = origin_x;
        for ch in text.chars() {
            let Some(rows) = font(ch.to_ascii_uppercase()) else {
                cursor += scale * 4;
                continue;
            };
            for row in 0..7u32 {
                for col in 0..5u32 {
                    if (rows[row as usize] >> (4 - col)) & 1 == 0 {
                        continue;
                    }
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let x = cursor + col * scale + dx;
                            let y = origin_y + row * scale + dy;
                            let i = ((y * frame.width + x) * 3) as usize;
                            frame.pixels[i] = 255;
                            frame.pixels[i + 1] = 255;
                            frame.pixels[i + 2] = 255;
                        }
                    }
                }
            }
            cursor += 5 * scale + scale;
        }
    }

    #[test]
    fn finds_folgen_inside_the_region() {
        let mut frame = Frame {
            width: 120,
            height: 40,
            pixels: vec![0; 120 * 40 * 3],
        };
        paint(&mut frame, "FOLGEN", 8, 8);
        let point = find_text_in_region(
            &frame,
            "Folgen",
            Rect {
                x: 0,
                y: 0,
                width: 120,
                height: 40,
            },
        )
        .unwrap();
        assert!(point.x > 8 && point.x < 90);
        assert!(point.y > 8 && point.y < 24);
    }

    #[test]
    fn rejects_a_different_label() {
        let mut frame = Frame {
            width: 120,
            height: 40,
            pixels: vec![0; 120 * 40 * 3],
        };
        paint(&mut frame, "FOLGEN", 8, 8);
        assert!(
            find_text_in_region(
                &frame,
                "Routes",
                Rect {
                    x: 0,
                    y: 0,
                    width: 120,
                    height: 40,
                },
            )
            .is_none()
        );
    }
}
