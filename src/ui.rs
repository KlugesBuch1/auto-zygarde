use crate::popup::{self, Icon};
use crate::{Frame, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    MainMenu,
    RoutesTab,
    SeeNearby,
    FirstRoute,
    Follow,
    RouteIcon,
    Complete,
    Dismiss,
}

pub fn point(button: Button, width: u32, height: u32) -> Point {
    let (x, y) = spot(button);
    Point {
        x: (x * width as f32).round() as u32,
        y: (y * height as f32).round() as u32,
    }
}

pub fn overworld(icon: &Icon, frame: &Frame) -> bool {
    popup::shows_icon_rows(icon, frame, 0.7, 1.0)
}

fn spot(button: Button) -> (f32, f32) {
    match button {
        Button::MainMenu => (0.90, 0.92),
        Button::RoutesTab => (0.70, 0.20),
        Button::SeeNearby => (0.50, 0.32),
        Button::FirstRoute => (0.50, 0.44),
        Button::Follow => (0.50, 0.86),
        Button::RouteIcon => (0.90, 0.78),
        Button::Complete => (0.50, 0.60),
        Button::Dismiss => (0.50, 0.55),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_button_sits_at_the_bottom_right() {
        assert_eq!(
            point(Button::MainMenu, 1000, 2000),
            Point { x: 900, y: 1840 }
        );
    }
}
