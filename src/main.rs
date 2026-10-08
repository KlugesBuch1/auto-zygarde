use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

const SHOT: &str = "data/frame.png";

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        eprintln!("usage: auto-zygarde <frame|capture|watch> [tap]");
        return ExitCode::from(2);
    };
    let do_tap = match args.next() {
        None => false,
        Some(arg) if arg == "tap" => true,
        Some(_) => {
            eprintln!("usage: auto-zygarde <frame|capture|watch> [tap]");
            return ExitCode::from(2);
        }
    };

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
    let path = PathBuf::from(SHOT);
    loop {
        if let Err(err) = auto_zygarde::capture(&path) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
        let Some(frame) = load_frame(&path) else {
            return ExitCode::FAILURE;
        };
        if let Some(point) = auto_zygarde::find_cell(&frame) {
            return finish(Some(point), do_tap);
        }
        thread::sleep(Duration::from_secs(1));
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
