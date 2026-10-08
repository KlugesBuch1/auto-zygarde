use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub const CELL_GOAL: u32 = 3;
pub const TELEPORT_DELAY: Duration = Duration::from_millis(1200);
pub const CLICK_DELAY: Duration = Duration::from_millis(1000);
pub const COMPLETE_WAIT: Duration = Duration::from_secs(5);
pub const END_SCAN: Duration = Duration::from_secs(7);
pub const SCAN_POLL: Duration = Duration::from_millis(300);
pub const WALK_POLL: Duration = Duration::from_secs(1);
pub const WAYPOINT_PAUSE: Duration = Duration::from_millis(400);
pub const ZOOM_OUT_PAUSE: Duration = Duration::from_secs(1);

pub const POPUP_GOAL: u32 = CELL_GOAL;
pub const POPUP_WINDOW: Duration = Duration::from_secs(2);
pub const POPUP_POLL: Duration = Duration::from_millis(400);

pub const GLOW_HUE_MIN: u8 = 35;
pub const GLOW_HUE_MAX: u8 = 85;
pub const GLOW_SAT_MIN: u8 = 150;
pub const GLOW_VAL_MIN: u8 = 200;
pub const GLOW_MIN_AREA: u32 = 12;
pub const GLOW_MAX_AREA: u32 = 20_000;

pub const MATCH_SCALES: [f32; 4] = [0.35, 0.5, 0.75, 1.0];
pub const MATCH_TOP: f32 = 0.35;
pub const MATCH_WORK_WIDTH: u32 = 180;
pub const MATCH_MIN_SIDE: u32 = 8;
pub const MATCH_MIN_SIMILARITY: f32 = 0.8;
pub const ICON_ALPHA_MIN: u8 = 128;

pub const ROI_WIDTH_PCT: f32 = 0.44;
pub const ROI_HEIGHT_PCT: f32 = 0.08;
pub const ACTION_MENU_BAND: f32 = 0.08;

pub const OCR_GLYPH_GAP: usize = 6;
pub const OCR_MAX_MISS: usize = 4;

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
    y_pct: 0.789,
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
pub const CANCEL_ROUTE_COMPLETION: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.9218,
};
pub const ACTION_MENU_BTN: RelativeTarget = RelativeTarget {
    x_pct: 0.50,
    y_pct: 0.914,
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
        let width = ((screen_width as f32) * ROI_WIDTH_PCT).round().max(1.0) as u32;
        let height = ((screen_height as f32) * ROI_HEIGHT_PCT).round().max(1.0) as u32;
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
        let (x, y) = POKEMON_MENU_BTN.to_absolute(1080, 2400);
        assert_eq!(
            (x, y),
            (
                (POKEMON_MENU_BTN.x_pct * 1080.0).round() as u32,
                (POKEMON_MENU_BTN.y_pct * 2400.0).round() as u32,
            )
        );
    }

    #[test]
    fn roi_stays_inside_the_frame() {
        let region = SEE_NEARBY_ROUTES_BTN.roi(1080, 2400);
        assert!(region.x + region.width <= 1080);
        assert!(region.y + region.height <= 2400);
        assert!(region.width > 0 && region.height > 0);
    }
}
