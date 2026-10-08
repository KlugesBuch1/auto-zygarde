use std::io;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::WAYPOINT_PAUSE;
use crate::gpx::{self, GpxRoute};

pub fn teleport_to(lat: f64, lon: f64) -> io::Result<()> {
    finish(teleport_command(lat, lon), "teleport")
}

pub fn teleport_route_waypoints(waypoints: &[(f64, f64)]) -> io::Result<()> {
    hop_waypoints(waypoints, WAYPOINT_PAUSE, teleport_to)
}

fn hop_waypoints(
    waypoints: &[(f64, f64)],
    pause: Duration,
    mut hop: impl FnMut(f64, f64) -> io::Result<()>,
) -> io::Result<()> {
    for (index, &(lat, lon)) in waypoints.iter().enumerate() {
        if index > 0 {
            thread::sleep(pause);
        }
        hop(lat, lon)?;
    }
    Ok(())
}

pub fn load_and_start_gpx(path: &Path) -> io::Result<()> {
    let route = read_route(path)?;
    finish(walk_command(&route.points), "walk")
}

pub fn stop() -> io::Result<()> {
    finish(stop_command(), "stop")
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

fn finish(mut command: Command, name: &str) -> io::Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{name} exited with {status}")))
    }
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
    fn hops_waypoints_in_order() {
        let mut seen = Vec::new();
        hop_waypoints(
            &[(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)],
            Duration::ZERO,
            |lat, lon| {
                seen.push((lat, lon));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(seen, vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)]);
        assert!(hop_waypoints(&[], Duration::ZERO, |_, _| unreachable!()).is_ok());
    }
}
