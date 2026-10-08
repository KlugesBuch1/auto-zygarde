use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(first) = args.next() else {
        eprintln!("usage: auto-zygarde <frame|capture> [tap]");
        return ExitCode::from(2);
    };
    let do_tap = match args.next() {
        None => false,
        Some(arg) if arg == "tap" => true,
        Some(_) => {
            eprintln!("usage: auto-zygarde <frame|capture> [tap]");
            return ExitCode::from(2);
        }
    };

    let path = if first == "capture" {
        let path = PathBuf::from("data/frame.png");
        if let Err(err) = auto_zygarde::capture(&path) {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
        path
    } else {
        PathBuf::from(first)
    };

    let frame = match auto_zygarde::Frame::load(&path) {
        Ok(frame) => frame,
        Err(err) => {
            eprintln!("{path:?}: {err}");
            return ExitCode::FAILURE;
        }
    };

    match auto_zygarde::find_cell(&frame) {
        Some(point) => {
            println!("{} {}", point.x, point.y);
            if do_tap {
                if let Err(err) = auto_zygarde::tap(&point) {
                    eprintln!("{err}");
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("no match");
            ExitCode::FAILURE
        }
    }
}
