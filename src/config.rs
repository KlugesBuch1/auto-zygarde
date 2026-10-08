use crate::ocr::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelativeTarget {
    pub x_pct: f32,
    pub y_pct: f32,
}

pub const POKEMON_MENU_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.861,
    y_pct: 0.9218,
};
pub const ROUTES_TAB: RelativeTarget = RelativeTarget {
    x_pct: 0.8472,
    y_pct: 0.1406,
};
pub const SEE_NEARBY_ROUTES_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.7343,
};
pub const FIRST_ROUTE_ENTRY: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.7890,
};
pub const FOLLOW_ROUTE_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.7773,
};
pub const ACTIVE_ROUTE_ICON: RelativeTarget = RelativeTarget {
    x_pct: 0.9166,
    y_pct: 0.6937,
};
pub const COMPLETE_ROUTE_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.8281,
};
pub const CONFIRM_COMPLETE_ROUTE_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.5312,
};
pub const REWARD_CLAIM_AREA: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.50,
};
pub const CANCEL_ROUTE_COMPLETION: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.9218,
};

impl RelativeTarget {
    pub fn to_absolute(&self, screen_width: u32, screen_height: u32) -> (u32, u32) {
        if screen_width == 0 || screen_height == 0 {
            return (0, 0);
        }
        let x = (self.x_pct.clamp(0.0, 1.0) * screen_width as f32).round() as u32;
        let y = (self.y_pct.clamp(0.0, 1.0) * screen_height as f32).round() as u32;
        (x.min(screen_width - 1), y.min(screen_height - 1))
    }

    pub fn roi(&self, screen_width: u32, screen_height: u32) -> Rect {
        let (cx, cy) = self.to_absolute(screen_width, screen_height);
        let width = ((screen_width as f32) * 0.44).round().max(1.0) as u32;
        let height = ((screen_height as f32) * 0.08).round().max(1.0) as u32;
        let x = cx.saturating_sub(width / 2);
        let y = cy.saturating_sub(height / 2);
        let width = width.min(screen_width.saturating_sub(x));
        let height = height.min(screen_height.saturating_sub(y));
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_a_1080x2400_menu_point() {
        assert_eq!(POKEMON_MENU_BTN.to_absolute(1080, 2400), (540, 2148));
    }

    #[test]
    fn roi_stays_inside_the_frame() {
        let region = SEE_NEARBY_ROUTES_BTN.roi(1080, 2400);
        assert!(region.x + region.width <= 1080);
        assert!(region.y + region.height <= 2400);
        assert!(region.width > 0 && region.height > 0);
    }
}
