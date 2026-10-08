use crate::config::{
    COMPLETE_ROUTE_BTN, FIRST_ROUTE_ENTRY, FOLLOW_ROUTE_BTN, ORANGE_ROUTE_ICON, POKEMON_MENU_BTN,
    REWARD_CLAIM_AREA, ROUTES_TAB, RelativeTarget, SEE_NEARBY_ROUTES_BTN,
};
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

pub fn target(button: Button) -> RelativeTarget {
    match button {
        Button::MainMenu => POKEMON_MENU_BTN,
        Button::RoutesTab => ROUTES_TAB,
        Button::SeeNearby => SEE_NEARBY_ROUTES_BTN,
        Button::FirstRoute => FIRST_ROUTE_ENTRY,
        Button::Follow => FOLLOW_ROUTE_BTN,
        Button::RouteIcon => ORANGE_ROUTE_ICON,
        Button::Complete => COMPLETE_ROUTE_BTN,
        Button::Dismiss => REWARD_CLAIM_AREA,
    }
}

pub fn labels(button: Button) -> &'static [&'static str] {
    match button {
        Button::RoutesTab => &["Routes", "Routen"],
        Button::SeeNearby => &["See Nearby Routes"],
        Button::Follow => &["Follow", "Folgen"],
        Button::Complete => &["Complete", "Route beendet"],
        _ => &[],
    }
}

pub fn point(button: Button, width: u32, height: u32) -> Point {
    let (x, y) = target(button).to_absolute(width, height);
    Point { x, y }
}

pub fn overworld(icon: &Icon, frame: &Frame) -> bool {
    popup::shows_icon_rows(icon, frame, 0.7, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_button_uses_the_relative_spot() {
        assert_eq!(
            point(Button::MainMenu, 1000, 2000),
            Point { x: 500, y: 1790 }
        );
    }
}
