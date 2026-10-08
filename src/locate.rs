use crate::Template;

pub struct Hit {
    pub x: u32,
    pub y: u32,
    pub similarity: f32,
}

pub fn locate(template: &Template, frame: &Template) -> Option<Hit> {
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

    let mut best_sad = u64::MAX;
    let mut best_x = 0u32;
    let mut best_y = 0u32;

    for y in 0..=(fh - th) {
        for x in 0..=(fw - tw) {
            let mut sad = 0u64;
            for ty in 0..th {
                let frame_row = (y + ty) * fw + x;
                let template_row = ty * tw;
                for tx in 0..tw {
                    let diff = template.pixels[template_row + tx] as i16
                        - frame.pixels[frame_row + tx] as i16;
                    sad += diff.unsigned_abs() as u64;
                }
            }
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

    let worst = (tw * th) as f32 * 255.0;
    Some(Hit {
        x: best_x,
        y: best_y,
        similarity: 1.0 - best_sad as f32 / worst,
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
        let hit = locate(&template, &frame).unwrap();
        assert_eq!((hit.x, hit.y), (1, 1));
        assert_eq!(hit.similarity, 1.0);
    }

    #[test]
    fn rejects_template_larger_than_frame() {
        let frame = image(2, 2, vec![1, 2, 3, 4]);
        let template = image(3, 1, vec![1, 2, 3]);
        assert!(locate(&template, &frame).is_none());
    }
}
