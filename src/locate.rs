use crate::Template;

pub struct Hit {
    pub x: u32,
    pub y: u32,
    pub similarity: f32,
}

struct View {
    tw: usize,
    th: usize,
    fw: usize,
    fh: usize,
}

fn view(template: &Template, frame: &Template) -> Option<View> {
    let tw = template.width as usize;
    let th = template.height as usize;
    let fw = frame.width as usize;
    let fh = frame.height as usize;
    if tw == 0 || th == 0 || tw > fw || th > fh {
        return None;
    }
    if template.pixels.len() != tw * th || frame.pixels.len() != fw * fh {
        return None;
    }
    Some(View { tw, th, fw, fh })
}

fn sad_at(template: &Template, frame: &Template, view: &View, x: usize, y: usize) -> u64 {
    let mut sad = 0u64;
    for ty in 0..view.th {
        let frame_row = (y + ty) * view.fw + x;
        let template_row = ty * view.tw;
        for tx in 0..view.tw {
            let diff =
                template.pixels[template_row + tx] as i16 - frame.pixels[frame_row + tx] as i16;
            sad += diff.unsigned_abs() as u64;
        }
    }
    sad
}

fn similarity(sad: u64, pixels: usize) -> f32 {
    1.0 - sad as f32 / (pixels as f32 * 255.0)
}

pub fn locate(template: &Template, frame: &Template, min_similarity: f32) -> Option<Hit> {
    let view = view(template, frame)?;
    let mut best_sad = u64::MAX;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    for y in 0..=(view.fh - view.th) {
        for x in 0..=(view.fw - view.tw) {
            let sad = sad_at(template, frame, &view, x, y);
            if sad < best_sad {
                best_sad = sad;
                best_x = x as u32;
                best_y = y as u32;
                if sad == 0 {
                    return Some(Hit {
                        x: best_x,
                        y: best_y,
                        similarity: 1.0,
                    });
                }
            }
        }
    }

    let score = similarity(best_sad, view.tw * view.th);
    if score < min_similarity.clamp(0.0, 1.0) {
        return None;
    }
    Some(Hit {
        x: best_x,
        y: best_y,
        similarity: score,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(width: u32, height: u32, pixels: Vec<u8>) -> Template {
        Template {
            width,
            height,
            pixels,
        }
    }

    #[test]
    fn finds_exact_patch() {
        let frame = image(4, 3, vec![0, 0, 0, 0, 0, 9, 8, 0, 0, 7, 6, 0]);
        let template = image(2, 2, vec![9, 8, 7, 6]);
        let hit = locate(&template, &frame, 0.9).unwrap();
        assert_eq!((hit.x, hit.y), (1, 1));
        assert_eq!(hit.similarity, 1.0);
    }

    #[test]
    fn rejects_template_larger_than_frame() {
        let frame = image(2, 2, vec![1, 2, 3, 4]);
        let template = image(3, 1, vec![1, 2, 3]);
        assert!(locate(&template, &frame, 0.5).is_none());
    }

    #[test]
    fn rejects_weak_best_hit() {
        let frame = image(2, 2, vec![0, 0, 0, 0]);
        let template = image(2, 2, vec![255, 255, 255, 255]);
        assert!(locate(&template, &frame, 0.9).is_none());
    }
}
