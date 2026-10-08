use std::io;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crate::config::{
    CLICK_DELAY, COMPLETE_WAIT, END_SCAN, SCAN_POLL, TELEPORT_DELAY, ZOOM_OUT_PAUSE,
};
use crate::gpx::{self, GpxRoute};
use crate::mock_location::{ensure_zoomed_out, stop, teleport_route_waypoints, teleport_to};
use crate::popup::Icon;
use crate::ui::{self, Button};
use crate::{Frame, Point, capture, find_cell, tap};

const SHOT: &str = "data/frame.png";
const CELL_GOAL: u32 = crate::config::CELL_GOAL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    StartRoute,
    TeleportingRoute,
    ScanningAtEnd,
    Collecting,
    CompletingRoute,
    NextRoute,
    Finished,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Teleport { lat: f64, lon: f64 },
    Wait(Duration),
    Press(Button),
    TeleportWaypoints(Vec<(f64, f64)>),
    ScanEnd,
    Tap(Point),
    CheckMenu,
    ZoomOut,
    Stop,
    Exit,
}

pub struct Bot {
    routes: Vec<GpxRoute>,
    index: usize,
    cells: u32,
    state: State,
    waypoints: Vec<(f64, f64)>,
    menu_retries: u8,
}

impl Bot {
    pub fn new(routes: Vec<GpxRoute>) -> Self {
        Self {
            routes,
            index: 0,
            cells: 0,
            state: State::Idle,
            waypoints: Vec::new(),
            menu_retries: 0,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn cells_collected(&self) -> u32 {
        self.cells
    }

    pub fn start(&mut self) -> Vec<Action> {
        self.state = State::Idle;
        self.open_route()
    }

    pub fn route_opened(&mut self) -> Vec<Action> {
        if self.state != State::StartRoute {
            return Vec::new();
        }
        if self.waypoints.is_empty() {
            return self.finish();
        }
        self.state = State::TeleportingRoute;
        vec![Action::TeleportWaypoints(self.waypoints.clone())]
    }

    pub fn on_arrived(&mut self) -> Vec<Action> {
        if self.state != State::TeleportingRoute {
            return Vec::new();
        }
        self.state = State::ScanningAtEnd;
        vec![Action::ScanEnd]
    }

    pub fn on_end_scan(&mut self, cell: Option<Point>, timed_out: bool) -> Vec<Action> {
        if self.state != State::ScanningAtEnd {
            return Vec::new();
        }
        if let Some(point) = cell {
            self.state = State::Collecting;
            return vec![Action::Tap(point)];
        }
        if timed_out {
            return self.begin_complete();
        }
        vec![Action::ScanEnd]
    }

    pub fn confirm_cell(&mut self) -> Vec<Action> {
        if self.state != State::Collecting {
            return Vec::new();
        }
        self.cells += 1;
        self.begin_complete()
    }

    pub fn on_menu(&mut self, menu_visible: bool) -> Vec<Action> {
        if self.state != State::CompletingRoute {
            return Vec::new();
        }
        if menu_visible || self.menu_retries >= 1 {
            return self.after_route();
        }
        self.menu_retries += 1;
        vec![
            Action::Press(Button::CancelCompletion),
            Action::Wait(CLICK_DELAY),
            Action::CheckMenu,
        ]
    }

    fn open_route(&mut self) -> Vec<Action> {
        if self.cells >= CELL_GOAL || self.index >= self.routes.len() {
            return self.finish();
        }
        let zoom_out = self.index == 0;
        let route = self.routes[self.index].clone();
        self.index += 1;
        self.waypoints = route.points;
        self.menu_retries = 0;
        self.state = State::StartRoute;
        let mut actions = Vec::new();
        if zoom_out {
            actions.extend([Action::ZoomOut, Action::Wait(ZOOM_OUT_PAUSE)]);
        }
        actions.extend([
            Action::Teleport {
                lat: route.start_lat,
                lon: route.start_lon,
            },
            Action::Wait(TELEPORT_DELAY),
            Action::Press(Button::MainMenu),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::RoutesTab),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::SeeNearby),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::FirstRoute),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::Follow),
            Action::Wait(CLICK_DELAY),
        ]);
        actions
    }

    fn begin_complete(&mut self) -> Vec<Action> {
        self.state = State::CompletingRoute;
        self.menu_retries = 0;
        vec![
            Action::Press(Button::ActiveRoute),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::Complete),
            Action::Wait(COMPLETE_WAIT),
            Action::Press(Button::CancelCompletion),
            Action::Wait(CLICK_DELAY),
            Action::CheckMenu,
        ]
    }

    fn after_route(&mut self) -> Vec<Action> {
        self.state = State::NextRoute;
        self.open_route()
    }

    fn finish(&mut self) -> Vec<Action> {
        self.state = State::Finished;
        vec![Action::Stop, Action::Exit]
    }
}

pub fn run_routes(dir: &Path) -> io::Result<()> {
    let routes = gpx::load_routes(dir)?;
    let menu_icon =
        Icon::load("assets/templates/action_menu.png").map_err(|err| io::Error::other(err))?;
    let mut bot = Bot::new(routes);
    let mut actions = bot.start();
    let mut scan_started: Option<Instant> = None;
    let shot = Path::new(SHOT);
    loop {
        if actions.is_empty() {
            if bot.state() == State::StartRoute {
                actions = bot.route_opened();
                continue;
            }
            return Ok(());
        }
        let mut next = Vec::new();
        for action in actions {
            match action {
                Action::Teleport { lat, lon } => teleport_to(lat, lon)?,
                Action::Wait(delay) => thread::sleep(delay),
                Action::Press(button) => press(shot, button)?,
                Action::TeleportWaypoints(points) => {
                    teleport_route_waypoints(&points)?;
                    scan_started = None;
                    next.extend(bot.on_arrived());
                }
                Action::ScanEnd => {
                    let started = scan_started.get_or_insert_with(Instant::now);
                    capture(shot)?;
                    let frame = load_shot(shot)?;
                    let timed_out = started.elapsed() >= END_SCAN;
                    let follow = bot.on_end_scan(find_cell(&frame), timed_out);
                    if bot.state() == State::ScanningAtEnd {
                        thread::sleep(SCAN_POLL);
                    } else {
                        scan_started = None;
                    }
                    next.extend(follow);
                }
                Action::Tap(point) => {
                    println!("{} {}", point.x, point.y);
                    tap(&point)?;
                    let follow = bot.confirm_cell();
                    println!("popup {}", bot.cells_collected());
                    next.extend(follow);
                }
                Action::CheckMenu => {
                    capture(shot)?;
                    let frame = load_shot(shot)?;
                    next.extend(bot.on_menu(ui::action_menu_visible(&menu_icon, &frame)));
                }
                Action::ZoomOut => ensure_zoomed_out()?,
                Action::Stop => stop()?,
                Action::Exit => return Ok(()),
            }
        }
        actions = next;
    }
}

fn press(shot: &Path, button: Button) -> io::Result<()> {
    capture(shot)?;
    let frame = load_shot(shot)?;
    let spot = ui::target(button);
    let point = ui::labels(button).iter().find_map(|label| {
        crate::ocr::find_text_in_region(&frame, label, spot.roi(frame.width, frame.height))
    });
    let point = point.unwrap_or_else(|| ui::point(button, frame.width, frame.height));
    println!("{} {}", point.x, point.y);
    tap(&point)?;
    Ok(())
}

fn load_shot(path: &Path) -> io::Result<Frame> {
    Frame::load(path).map_err(|err| io::Error::other(err))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn route(name: &str, lat: f64, lon: f64) -> GpxRoute {
        route_points(name, vec![(lat, lon)])
    }

    fn route_points(name: &str, points: Vec<(f64, f64)>) -> GpxRoute {
        let (start_lat, start_lon) = points[0];
        GpxRoute {
            path: PathBuf::from(name),
            start_lat,
            start_lon,
            points,
        }
    }

    fn start_clicks(lat: f64, lon: f64) -> Vec<Action> {
        vec![
            Action::Teleport { lat, lon },
            Action::Wait(TELEPORT_DELAY),
            Action::Press(Button::MainMenu),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::RoutesTab),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::SeeNearby),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::FirstRoute),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::Follow),
            Action::Wait(CLICK_DELAY),
        ]
    }

    fn complete_clicks() -> Vec<Action> {
        vec![
            Action::Press(Button::ActiveRoute),
            Action::Wait(CLICK_DELAY),
            Action::Press(Button::Complete),
            Action::Wait(COMPLETE_WAIT),
            Action::Press(Button::CancelCompletion),
            Action::Wait(CLICK_DELAY),
            Action::CheckMenu,
        ]
    }

    #[test]
    fn starts_the_route_before_teleporting() {
        let points = vec![(48.5, 11.25), (48.6, 11.3)];
        let mut bot = Bot::new(vec![route_points("a.gpx", points.clone())]);
        let mut expected = vec![Action::ZoomOut, Action::Wait(ZOOM_OUT_PAUSE)];
        expected.extend(start_clicks(48.5, 11.25));
        assert_eq!(bot.start(), expected);
        assert_eq!(bot.state(), State::StartRoute);
        assert_eq!(bot.route_opened(), vec![Action::TeleportWaypoints(points)]);
        assert_eq!(bot.state(), State::TeleportingRoute);
    }

    #[test]
    fn scans_only_after_the_last_waypoint() {
        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0)]);
        bot.start();
        bot.route_opened();
        let point = Point { x: 10, y: 20 };
        assert_eq!(bot.on_arrived(), vec![Action::ScanEnd]);
        assert_eq!(bot.state(), State::ScanningAtEnd);
        assert_eq!(bot.on_end_scan(None, false), vec![Action::ScanEnd]);
        assert_eq!(bot.on_end_scan(None, true), complete_clicks());
        assert_eq!(bot.cells_collected(), 0);

        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0)]);
        bot.start();
        bot.route_opened();
        bot.on_arrived();
        assert_eq!(
            bot.on_end_scan(Some(point), false),
            vec![Action::Tap(point)]
        );
        assert_eq!(bot.confirm_cell(), complete_clicks());
        assert_eq!(bot.cells_collected(), 1);
    }

    #[test]
    fn retries_cancel_until_the_menu_is_back() {
        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0), route("b.gpx", 3.0, 4.0)]);
        bot.start();
        bot.route_opened();
        bot.on_arrived();
        bot.on_end_scan(None, true);
        assert_eq!(
            bot.on_menu(false),
            vec![
                Action::Press(Button::CancelCompletion),
                Action::Wait(CLICK_DELAY),
                Action::CheckMenu,
            ]
        );
        assert_eq!(bot.state(), State::CompletingRoute);
        assert_eq!(bot.on_menu(true), start_clicks(3.0, 4.0));
        assert_eq!(bot.state(), State::StartRoute);
    }

    #[test]
    fn stops_after_three_cells() {
        let mut bot = Bot::new(vec![
            route("a.gpx", 1.0, 2.0),
            route("b.gpx", 3.0, 4.0),
            route("c.gpx", 5.0, 6.0),
        ]);
        let point = Point { x: 4, y: 5 };
        bot.start();
        for _ in 0..3 {
            bot.route_opened();
            bot.on_arrived();
            bot.on_end_scan(Some(point), false);
            bot.confirm_cell();
            bot.on_menu(true);
        }
        assert_eq!(bot.cells_collected(), 3);
        assert_eq!(bot.state(), State::Finished);
    }

    #[test]
    fn empty_folder_finishes_immediately() {
        let mut bot = Bot::new(Vec::new());
        assert_eq!(bot.start(), vec![Action::Stop, Action::Exit]);
        assert_eq!(bot.state(), State::Finished);
    }
}
