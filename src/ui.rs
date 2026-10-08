use crate::config::{
    ACTION_MENU_BAND, ACTION_MENU_BTN, ACTIVE_ROUTE_ICON, CANCEL_ROUTE_COMPLETION,
    COMPLETE_ROUTE_BTN, FIRST_ROUTE_ENTRY, FOLLOW_ROUTE_BTN, POKEMON_MENU_BTN, ROUTES_TAB,
    RelativeTarget, SEE_NEARBY_ROUTES_BTN,
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
    ActiveRoute,
    Complete,
    CancelCompletion,
}

pub fn target(button: Button) -> RelativeTarget {
    match button {
        Button::MainMenu => POKEMON_MENU_BTN,
        Button::RoutesTab => ROUTES_TAB,
        Button::SeeNearby => SEE_NEARBY_ROUTES_BTN,
        Button::FirstRoute => FIRST_ROUTE_ENTRY,
        Button::Follow => FOLLOW_ROUTE_BTN,
        Button::ActiveRoute => ACTIVE_ROUTE_ICON,
        Button::Complete => COMPLETE_ROUTE_BTN,
        Button::CancelCompletion => CANCEL_ROUTE_COMPLETION,
    }
}

pub fn labels(button: Button) -> &'static [&'static str] {
    match button {
        Button::RoutesTab => &["Routes", "Routen"],
        Button::SeeNearby => &["See Nearby Routes"],
        Button::Follow => &["Follow", "Folgen"],
        Button::Complete => &["Complete", "Route beendet"],
        Button::CancelCompletion => &["Cancel", "Abbrechen"],
        _ => &[],
    }
}

pub fn point(button: Button, width: u32, height: u32) -> Point {
    let (x, y) = target(button).to_absolute(width, height);
    Point { x, y }
}

pub fn action_menu_visible(icon: &Icon, frame: &Frame) -> bool {
    let top = (ACTION_MENU_BTN.y_pct - ACTION_MENU_BAND).max(0.0);
    popup::shows_icon_rows(icon, frame, top, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_button_uses_the_relative_spot() {
        let (x, y) = POKEMON_MENU_BTN.to_absolute(1000, 2000);
        assert_eq!(point(Button::MainMenu, 1000, 2000), Point { x, y });
    }
}
