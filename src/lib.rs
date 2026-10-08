mod bot;
mod capture;
mod config;
mod detection;
mod gpx;
mod input;
mod locate;
mod mock_location;
mod ocr;
mod popup;
mod template;
mod ui;

pub use bot::{Action, Bot, State, run_routes};
pub use capture::capture;
pub use config::{
    COMPLETE_ROUTE_BTN, FIRST_ROUTE_ENTRY, FOLLOW_ROUTE_BTN, ORANGE_ROUTE_ICON, POKEMON_MENU_BTN,
    REWARD_CLAIM_AREA, ROUTES_TAB, RelativeTarget, SEE_NEARBY_ROUTES_BTN,
};
pub use detection::{Frame, Point, find_cell};
pub use gpx::{GpxRoute, load_routes};
pub use input::tap;
pub use locate::{Hit, locate};
pub use mock_location::{load_and_start_gpx, stop, teleport_to};
pub use ocr::{Rect, find_text_in_region};
pub use popup::{Icon, POPUP_GOAL, POPUP_POLL, POPUP_WINDOW, Route, shows_cell_popup};
pub use template::Template;
pub use ui::{Button, point};
