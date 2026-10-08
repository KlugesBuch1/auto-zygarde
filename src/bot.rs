use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::gpx::{self, GpxRoute};
use crate::mock_location::{
    clear_route_log, load_and_start_gpx, route_is_active, stop, teleport_to,
};
use crate::popup::{self, Icon, POPUP_GOAL, POPUP_POLL, POPUP_WINDOW};
use crate::ui::{self, Button};
use crate::{Frame, Point, capture, find_cell, tap};

const SHOT: &str = "data/frame.png";
const MENU_PAUSE: Duration = Duration::from_millis(800);
const MAP_SETTLE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    StartInGameRoute,
    Scanning,
    Collecting,
    CompleteInGameRoute,
    Finished,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Teleport { lat: f64, lon: f64 },
    StartGpx(PathBuf),
    Press(Button),
    CheckMap,
    Scan,
    Tap(Point),
    Stop,
    Exit,
}

pub struct Bot {
    routes: Vec<GpxRoute>,
    index: usize,
    cells: u32,
    state: State,
}

impl Bot {
    pub fn new(routes: Vec<GpxRoute>) -> Self {
        Self {
            routes,
            index: 0,
            cells: 0,
            state: State::Idle,
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
        self.advance()
    }

    pub fn on_frame(&mut self, cell: Option<Point>, route_active: bool) -> Vec<Action> {
        if self.state != State::Scanning {
            return Vec::new();
        }
        if !route_active {
            return self.begin_complete();
        }
        if let Some(point) = cell {
            self.state = State::Collecting;
            return vec![Action::Tap(point)];
        }
        vec![Action::Scan]
    }

    pub fn on_popup(&mut self, seen: bool) -> Vec<Action> {
        if self.state != State::Collecting {
            return Vec::new();
        }
        if seen {
            self.cells += 1;
        }
        if self.cells >= POPUP_GOAL {
            return self.begin_complete();
        }
        self.state = State::Scanning;
        vec![Action::Scan]
    }

    pub fn on_overworld(&mut self, map_visible: bool) -> Vec<Action> {
        if self.state != State::CompleteInGameRoute {
            return Vec::new();
        }
        if !map_visible {
            return vec![Action::Press(Button::Dismiss), Action::CheckMap];
        }
        self.advance()
    }

    fn begin_complete(&mut self) -> Vec<Action> {
        self.state = State::CompleteInGameRoute;
        vec![
            Action::Stop,
            Action::Press(Button::RouteIcon),
            Action::Press(Button::Complete),
            Action::CheckMap,
        ]
    }

    fn advance(&mut self) -> Vec<Action> {
        if self.cells >= POPUP_GOAL || self.index >= self.routes.len() {
            self.state = State::Finished;
            return vec![Action::Stop, Action::Exit];
        }
        let route = self.routes[self.index].clone();
        self.index += 1;
        self.state = State::StartInGameRoute;
        self.state = State::Scanning;
        vec![
            Action::Teleport {
                lat: route.start_lat,
                lon: route.start_lon,
            },
            Action::Press(Button::MainMenu),
            Action::Press(Button::RoutesTab),
            Action::Press(Button::SeeNearby),
            Action::Press(Button::FirstRoute),
            Action::Press(Button::Follow),
            Action::StartGpx(route.path),
            Action::Scan,
        ]
    }
}

pub fn run_routes(dir: &Path) -> io::Result<()> {
    let routes = gpx::load_routes(dir)?;
    let cell_icon =
        Icon::load("assets/templates/zygarde_cell.png").map_err(|err| io::Error::other(err))?;
    let menu_icon =
        Icon::load("assets/templates/action_menu.png").map_err(|err| io::Error::other(err))?;
    let mut bot = Bot::new(routes);
    let mut actions = bot.start();
    let mut moving = false;
    let shot = Path::new(SHOT);
    loop {
        if actions.is_empty() {
            return Ok(());
        }
        let mut next = Vec::new();
        for action in actions {
            match action {
                Action::Teleport { lat, lon } => {
                    teleport_to(lat, lon)?;
                    thread::sleep(MAP_SETTLE);
                }
                Action::StartGpx(path) => {
                    clear_route_log();
                    load_and_start_gpx(&path)?;
                    moving = false;
                }
                Action::Press(button) => {
                    press(shot, button)?;
                }
                Action::CheckMap => {
                    thread::sleep(MENU_PAUSE);
                    capture(shot)?;
                    let frame = load_shot(shot)?;
                    next.extend(bot.on_overworld(ui::overworld(&menu_icon, &frame)));
                }
                Action::Stop => stop()?,
                Action::Exit => return Ok(()),
                Action::Tap(point) => {
                    println!("{} {}", point.x, point.y);
                    tap(&point)?;
                    let seen = wait_for_popup(shot, &cell_icon)?;
                    let before = bot.cells_collected();
                    let follow = bot.on_popup(seen);
                    if bot.cells_collected() > before {
                        println!("popup {}", bot.cells_collected());
                    }
                    next.extend(follow);
                }
                Action::Scan => {
                    thread::sleep(Duration::from_secs(1));
                    capture(shot)?;
                    let frame = load_shot(shot)?;
                    let active = route_is_active().unwrap_or(true);
                    if active {
                        moving = true;
                    }
                    let ended = moving && !active;
                    next.extend(bot.on_frame(find_cell(&frame), !ended));
                }
            }
        }
        actions = next;
    }
}

fn press(shot: &Path, button: Button) -> io::Result<()> {
    capture(shot)?;
    let frame = load_shot(shot)?;
    let point = ui::point(button, frame.width, frame.height);
    println!("{} {}", point.x, point.y);
    tap(&point)?;
    thread::sleep(MENU_PAUSE);
    Ok(())
}

fn load_shot(path: &Path) -> io::Result<Frame> {
    Frame::load(path).map_err(|err| io::Error::other(err))
}

fn wait_for_popup(path: &Path, icon: &Icon) -> io::Result<bool> {
    let started = Instant::now();
    loop {
        capture(path)?;
        let frame = load_shot(path)?;
        if popup::shows_cell_popup(icon, &frame) {
            return Ok(true);
        }
        if started.elapsed() >= POPUP_WINDOW {
            return Ok(false);
        }
        let left = POPUP_WINDOW.saturating_sub(started.elapsed());
        thread::sleep(POPUP_POLL.min(left));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(name: &str, lat: f64, lon: f64) -> GpxRoute {
        GpxRoute {
            path: PathBuf::from(name),
            start_lat: lat,
            start_lon: lon,
            points: vec![(lat, lon)],
        }
    }

    fn open_route(name: &str, lat: f64, lon: f64) -> Vec<Action> {
        vec![
            Action::Teleport { lat, lon },
            Action::Press(Button::MainMenu),
            Action::Press(Button::RoutesTab),
            Action::Press(Button::SeeNearby),
            Action::Press(Button::FirstRoute),
            Action::Press(Button::Follow),
            Action::StartGpx(PathBuf::from(name)),
            Action::Scan,
        ]
    }

    #[test]
    fn starts_the_in_game_route() {
        let mut bot = Bot::new(vec![route("a.gpx", 48.5, 11.25)]);
        assert_eq!(bot.start(), open_route("a.gpx", 48.5, 11.25));
        assert_eq!(bot.state(), State::Scanning);
    }

    #[test]
    fn collects_until_three_popups() {
        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0)]);
        bot.start();
        let point = Point { x: 10, y: 20 };
        assert_eq!(bot.on_frame(Some(point), true), vec![Action::Tap(point)]);
        assert_eq!(bot.state(), State::Collecting);
        assert_eq!(bot.on_popup(false), vec![Action::Scan]);
        assert_eq!(bot.cells_collected(), 0);
        bot.on_frame(Some(point), true);
        bot.on_popup(true);
        bot.on_frame(Some(point), true);
        bot.on_popup(true);
        bot.on_frame(Some(point), true);
        assert_eq!(bot.on_popup(true), complete_actions());
        assert_eq!(bot.cells_collected(), 3);
        assert_eq!(bot.state(), State::CompleteInGameRoute);
        assert_eq!(bot.on_overworld(true), vec![Action::Stop, Action::Exit]);
        assert_eq!(bot.state(), State::Finished);
    }

    #[test]
    fn dismisses_rewards_until_the_map_returns() {
        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0), route("b.gpx", 3.0, 4.0)]);
        bot.start();
        assert_eq!(bot.on_frame(None, false), complete_actions());
        assert_eq!(
            bot.on_overworld(false),
            vec![Action::Press(Button::Dismiss), Action::CheckMap]
        );
        assert_eq!(bot.state(), State::CompleteInGameRoute);
        assert_eq!(bot.on_overworld(true), open_route("b.gpx", 3.0, 4.0));
        assert_eq!(bot.state(), State::Scanning);
    }

    #[test]
    fn stops_when_no_routes_remain() {
        let mut bot = Bot::new(vec![route("a.gpx", 1.0, 2.0)]);
        bot.start();
        assert_eq!(bot.on_frame(None, false), complete_actions());
        assert_eq!(bot.on_overworld(true), vec![Action::Stop, Action::Exit]);
        assert_eq!(bot.state(), State::Finished);
    }

    #[test]
    fn empty_folder_finishes_immediately() {
        let mut bot = Bot::new(Vec::new());
        assert_eq!(bot.start(), vec![Action::Stop, Action::Exit]);
        assert_eq!(bot.state(), State::Finished);
    }

    fn complete_actions() -> Vec<Action> {
        vec![
            Action::Stop,
            Action::Press(Button::RouteIcon),
            Action::Press(Button::Complete),
            Action::CheckMap,
        ]
    }
}
