use std::io;
use std::process::Command;

use crate::Point;

pub fn tap(point: &Point) -> io::Result<()> {
    let status = tap_command("input", point).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("input tap exited with {status}")))
    }
}

fn tap_command(program: &str, point: &Point) -> Command {
    let mut command = Command::new(program);
    command.arg("tap");
    command.arg(point.x.to_string());
    command.arg(point.y.to_string());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tap_command_uses_input_coordinates() {
        let command = tap_command("input", &Point { x: 12, y: 34 });
        assert_eq!(command.get_program(), "input");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["tap", "12", "34"].map(std::ffi::OsStr::new)
        );
    }
}
