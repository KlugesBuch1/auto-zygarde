use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: auto-zygarde <frame> [tap]");
        return ExitCode::from(2);
    };
    let do_tap = match args.next() {
        None => false,
        Some(arg) if arg == "tap" => true,
        Some(_) => {
            eprintln!("usage: auto-zygarde <frame> [tap]");
            return ExitCode::from(2);
        }
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
