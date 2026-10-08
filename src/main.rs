use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

const SHOT: &str = "data/frame.png";

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        eprintln!("usage: auto-zygarde <frame|capture|watch|routes> [tap]");
        return ExitCode::from(2);
    };
    let do_tap = match args.next() {
        None => false,
        Some(arg) if arg == "tap" => true,
        Some(_) => {
            eprintln!("usage: auto-zygarde <frame|capture|watch|routes> [tap]");
            return ExitCode::from(2);
        }
    };

    if first == "routes" {
        return match auto_zygarde::run_routes(Path::new("assets/gpx")) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        };
    }

    if first == "watch" {
        return watch(do_tap);
    }

    let path = if first == "capture" {
        let path = PathBuf::from(SHOT);
        if let Err(err) = auto_zygarde::capture(&path) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
        path
    } else {
        PathBuf::from(first)
    };

    let Some(frame) = load_frame(&path) else {
        return ExitCode::FAILURE;
    };
    finish(auto_zygarde::find_cell(&frame), do_tap)
}

fn watch(do_tap: bool) -> ExitCode {
    let icon = match auto_zygarde::Icon::load("assets/templates/zygarde_cell.png") {
        Ok(icon) => icon,
        Err(err) => {
            eprintln!("assets/templates/zygarde_cell.png: {err}");
            return ExitCode::FAILURE;
        }
    };
    let path = PathBuf::from(SHOT);
    let mut route = auto_zygarde::Route::new();
    let mut ready = true;
    loop {
        if let Err(err) = auto_zygarde::capture(&path) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
        let Some(frame) = load_frame(&path) else {
            return ExitCode::FAILURE;
        };
        let Some(point) = auto_zygarde::find_cell(&frame) else {
            ready = true;
            thread::sleep(Duration::from_secs(1));
            continue;
        };
        if !ready {
            thread::sleep(auto_zygarde::POPUP_POLL);
            continue;
        }
        println!("{} {}", point.x, point.y);
        if do_tap {
            if let Err(err) = auto_zygarde::tap(&point) {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        }
        ready = false;
        let Some(popup) = popup_after_click(&path, &icon) else {
            return ExitCode::FAILURE;
        };
        if route.after_click(popup) {
            println!("popup {}", route.popups());
            return ExitCode::SUCCESS;
        }
        if popup {
            println!("popup {}", route.popups());
        }
    }
}

fn popup_after_click(path: &Path, icon: &auto_zygarde::Icon) -> Option<bool> {
    let started = Instant::now();
    loop {
        if let Err(err) = auto_zygarde::capture(path) {
            eprintln!("{err}");
            return None;
        }
        let frame = load_frame(path)?;
        if auto_zygarde::shows_cell_popup(icon, &frame) {
            return Some(true);
        }
        if started.elapsed() >= auto_zygarde::POPUP_WINDOW {
            return Some(false);
        }
        let left = auto_zygarde::POPUP_WINDOW.saturating_sub(started.elapsed());
        thread::sleep(auto_zygarde::POPUP_POLL.min(left));
    }
}

fn load_frame(path: &Path) -> Option<auto_zygarde::Frame> {
    match auto_zygarde::Frame::load(path) {
        Ok(frame) => Some(frame),
        Err(err) => {
            eprintln!("{path:?}: {err}");
            None
        }
    }
}

fn finish(point: Option<auto_zygarde::Point>, do_tap: bool) -> ExitCode {
    let Some(point) = point else {
        eprintln!("no match");
        return ExitCode::FAILURE;
    };
    println!("{} {}", point.x, point.y);
    if do_tap {
        if let Err(err) = auto_zygarde::tap(&point) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
