use std::io;
use std::path::Path;
use std::process::Command;

use crate::gpx::{self, GpxRoute};

pub fn teleport_to(lat: f64, lon: f64) -> io::Result<()> {
    finish(teleport_command(lat, lon), "teleport")
}

pub fn load_and_start_gpx(path: &Path) -> io::Result<()> {
    let route = read_route(path)?;
    finish(walk_command(&route.points), "walk")
}

pub fn stop() -> io::Result<()> {
    finish(stop_command(), "stop")
}

pub fn route_is_active() -> io::Result<bool> {
    let status = status_command().status()?;
    if !status.success() {
        return Err(io::Error::other(format!("status exited with {status}")));
    }
    let Ok(output) = Command::new("logcat").args(["-d", "-t", "100"]).output() else {
        return Ok(true);
    };
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(route_active_from_log(&text).unwrap_or(true))
}

pub fn clear_route_log() {
    let _ = Command::new("logcat").arg("-c").status();
}

fn read_route(path: &Path) -> io::Result<GpxRoute> {
    let text = std::fs::read_to_string(path)?;
    let points = gpx::track_points(&text);
    let Some(&(start_lat, start_lon)) = points.first() else {
        return Err(io::Error::other(format!(
            "{} has no track points",
            path.display()
        )));
    };
    Ok(GpxRoute {
        path: path.to_path_buf(),
        start_lat,
        start_lon,
        points,
    })
}

fn teleport_command(lat: f64, lon: f64) -> Command {
    let mut command = Command::new("am");
    command.args([
        "start-foreground-service",
        "-a",
        "theappninjas.gpsjoystick.TELEPORT",
        "--ef",
        "lat",
        &format!("{lat:.6}"),
        "--ef",
        "lng",
        &format!("{lon:.6}"),
    ]);
    command
}

fn walk_command(points: &[(f64, f64)]) -> Command {
    let waypoints = points
        .iter()
        .map(|(lat, lon)| format!("{lat:.6},{lon:.6}"))
        .collect::<Vec<_>>()
        .join(";");
    let mut command = Command::new("am");
    command.args([
        "start-foreground-service",
        "-a",
        "theappninjas.gpsjoystick.WALK",
        "--es",
        "waypoints",
        &waypoints,
    ]);
    command
}

fn stop_command() -> Command {
    let mut command = Command::new("am");
    command.args([
        "start-foreground-service",
        "-a",
        "theappninjas.gpsjoystick.STOP",
    ]);
    command
}

fn status_command() -> Command {
    let mut command = Command::new("am");
    command.args([
        "start-foreground-service",
        "-a",
        "theappninjas.gpsjoystick.STATUS",
    ]);
    command
}

fn finish(mut command: Command, name: &str) -> io::Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{name} exited with {status}")))
    }
}

pub(crate) fn route_active_from_log(text: &str) -> Option<bool> {
    let mut found = None;
    for line in text.lines() {
        let Some(rest) = line.split("is_route_active=").nth(1) else {
            continue;
        };
        let token = rest
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .next()
            .unwrap_or("");
        if token.eq_ignore_ascii_case("true") || token == "1" {
            found = Some(true);
        } else if token.eq_ignore_ascii_case("false") || token == "0" {
            found = Some(false);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teleport_command_uses_joystick_intent() {
        let command = teleport_command(48.5, -11.25);
        assert_eq!(command.get_program(), "am");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "start-foreground-service",
                "-a",
                "theappninjas.gpsjoystick.TELEPORT",
                "--ef",
                "lat",
                "48.500000",
                "--ef",
                "lng",
                "-11.250000",
            ]
            .map(std::ffi::OsStr::new)
        );
    }

    #[test]
    fn walk_command_sends_track_points() {
        let command = walk_command(&[(48.5, 11.25), (48.75, 11.5)]);
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[2], "theappninjas.gpsjoystick.WALK");
        assert_eq!(args[5], "48.500000,11.250000;48.750000,11.500000");
    }

    #[test]
    fn reads_latest_route_flag() {
        let text = "is_route_active=true\nis_route_active=false\n";
        assert_eq!(route_active_from_log(text), Some(false));
        assert_eq!(route_active_from_log("idle"), None);
    }
}
