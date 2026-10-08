use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

pub fn capture(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let status = capture_command("screencap", path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("screencap exited with {status}")))
    }
}

fn capture_command(program: &str, path: &Path) -> Command {
    let mut command = Command::new(program);
    command.arg("-p");
    command.arg(path);
    command
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn capture_command_writes_png() {
        let command = capture_command("screencap", Path::new("data/frame.png"));
        assert_eq!(command.get_program(), "screencap");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["-p", "data/frame.png"].map(std::ffi::OsStr::new)
        );
    }
}
