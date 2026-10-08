mod bot;
mod capture;
mod detection;
mod gpx;
mod input;
mod locate;
mod mock_location;
mod popup;
mod template;

pub use bot::{Action, Bot, State, run_routes};
pub use capture::capture;
pub use detection::{Frame, Point, find_cell};
pub use gpx::{GpxRoute, load_routes};
pub use input::tap;
pub use locate::{Hit, locate};
pub use mock_location::{load_and_start_gpx, stop, teleport_to};
pub use popup::{Icon, POPUP_GOAL, POPUP_POLL, POPUP_WINDOW, Route, shows_cell_popup};
pub use template::Template;
