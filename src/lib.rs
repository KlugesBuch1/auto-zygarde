mod capture;
mod detection;
mod input;
mod locate;
mod popup;
mod template;

pub use capture::capture;
pub use detection::{Frame, Point, find_cell};
pub use input::tap;
pub use locate::{Hit, locate};
pub use popup::{Icon, POPUP_GOAL, POPUP_POLL, POPUP_WINDOW, Route, shows_cell_popup};
pub use template::Template;
