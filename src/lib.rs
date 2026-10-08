mod capture;
mod detection;
mod input;
mod locate;
mod template;

pub use capture::capture;
pub use detection::{Frame, Point, find_cell};
pub use input::tap;
pub use locate::{Hit, locate};
pub use template::Template;
