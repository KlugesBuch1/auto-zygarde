use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: auto-zygarde <frame>");
        return ExitCode::from(2);
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
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("no match");
            ExitCode::FAILURE
        }
    }
}
