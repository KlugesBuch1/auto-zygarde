use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(template_path), Some(frame_path)) = (args.next(), args.next()) else {
        eprintln!("usage: auto-zygarde <template> <frame>");
        return ExitCode::from(2);
    };

    let template = match auto_zygarde::Template::load(&template_path) {
        Ok(template) => template,
        Err(err) => {
            eprintln!("{template_path:?}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let frame = match auto_zygarde::Template::load(&frame_path) {
        Ok(frame) => frame,
        Err(err) => {
            eprintln!("{frame_path:?}: {err}");
            return ExitCode::FAILURE;
        }
    };

    match auto_zygarde::locate(&template, &frame) {
        Some(hit) => {
            println!("{} {} {:.4}", hit.x, hit.y, hit.similarity);
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("template does not fit in frame");
            ExitCode::FAILURE
        }
    }
}
