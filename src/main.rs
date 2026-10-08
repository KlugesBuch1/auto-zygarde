use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: auto-zygarde <template>");
        return ExitCode::from(2);
    };

    match auto_zygarde::Template::load(&path) {
        Ok(template) => {
            println!("{}x{}", template.width, template.height);
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{path:?}: {err}");
            ExitCode::FAILURE
        }
    }
}
