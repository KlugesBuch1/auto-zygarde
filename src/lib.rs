mod detection;
mod locate;
mod template;

pub use detection::{Frame, Point, find_cell};
pub use locate::{Hit, locate};
pub use template::Template;
