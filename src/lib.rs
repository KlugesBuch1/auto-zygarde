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
    ACTION_MENU_BAND, ACTION_MENU_BTN, ACTIVE_ROUTE_ICON, CANCEL_ROUTE_COMPLETION, CELL_GOAL,
    CLICK_DELAY, COMPLETE_ROUTE_BTN, COMPLETE_WAIT, CONFIRM_COMPLETE_ROUTE_BTN, END_SCAN,
    FIRST_ROUTE_ENTRY, FOLLOW_ROUTE_BTN, GLOW_HUE_MAX, GLOW_HUE_MIN, GLOW_MAX_AREA, GLOW_MIN_AREA,
    GLOW_SAT_MIN, GLOW_VAL_MIN, ICON_ALPHA_MIN, MATCH_MIN_SIDE, MATCH_MIN_SIMILARITY, MATCH_SCALES,
    MATCH_TOP, MATCH_WORK_WIDTH, OCR_GLYPH_GAP, OCR_MAX_MISS, POKEMON_MENU_BTN, POPUP_GOAL,
    POPUP_POLL, POPUP_WINDOW, ROI_HEIGHT_PCT, ROI_WIDTH_PCT, ROUTES_TAB, Rect, RelativeTarget,
    SCAN_POLL, SEE_NEARBY_ROUTES_BTN, TELEPORT_DELAY, WALK_POLL, WAYPOINT_PAUSE,
};
pub use detection::{Frame, Point, find_cell};
pub use gpx::{GpxRoute, load_routes};
pub use input::tap;
pub use locate::{Hit, locate};
pub use mock_location::{load_and_start_gpx, stop, teleport_route_waypoints, teleport_to};
pub use ocr::find_text_in_region;
pub use popup::{Icon, Route, shows_cell_popup};
pub use template::Template;
pub use ui::{Button, point};
