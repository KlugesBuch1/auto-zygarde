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

const ZOOM_PINCHES: usize = 5;
const ZOOM_SWIPE_MS: u32 = 300;
const ZOOM_PINCH_GAP: Duration = Duration::from_millis(200);

#[derive(Clone, Copy)]
struct Pinch {
    from_a: (u32, u32),
    to_a: (u32, u32),
    from_b: (u32, u32),
    to_b: (u32, u32),
    duration_ms: u32,
}

pub fn ensure_zoomed_out() -> io::Result<()> {
    let (width, height) = display_size()?;
    for (index, pinch) in zoom_out_pinches(width, height).iter().enumerate() {
        if index > 0 {
            thread::sleep(ZOOM_PINCH_GAP);
        }
        run_pinch(pinch)?;
    }
    Ok(())
}

fn display_size() -> io::Result<(u32, u32)> {
    let output = Command::new("wm").arg("size").output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "wm size exited with {}",
            output.status
        )));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_display_size(&text).ok_or_else(|| io::Error::other(format!("wm size: {text}")))
}

fn parse_display_size(text: &str) -> Option<(u32, u32)> {
    let mut physical = None;
    let mut override_size = None;
    for line in text.lines() {
        let Some((label, size)) = line.split_once(':') else {
            continue;
        };
        let Some(parsed) = parse_size_pair(size) else {
            continue;
        };
        if label.contains("Override size") {
            override_size = Some(parsed);
        } else if label.contains("Physical size") {
            physical = Some(parsed);
        }
    }
    override_size.or(physical)
}

fn parse_size_pair(size: &str) -> Option<(u32, u32)> {
    let (width, height) = size.trim().split_once('x')?;
    let width = width.trim().parse().ok()?;
    let height = height.trim().parse().ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

fn zoom_out_pinches(width: u32, height: u32) -> Vec<Pinch> {
    let cx = width as f32 * 0.50;
    let cy = height as f32 * 0.45;
    let dx = width as f32 * 0.22;
    let dy = height as f32 * 0.12;
    let pinch = Pinch {
        from_a: (screen_axis(cx - dx, width), screen_axis(cy - dy, height)),
        to_a: (
            screen_axis(cx - dx * 0.15, width),
            screen_axis(cy - dy * 0.15, height),
        ),
        from_b: (screen_axis(cx + dx, width), screen_axis(cy + dy, height)),
        to_b: (
            screen_axis(cx + dx * 0.15, width),
            screen_axis(cy + dy * 0.15, height),
        ),
        duration_ms: ZOOM_SWIPE_MS,
    };
    vec![pinch; ZOOM_PINCHES]
}

fn screen_axis(value: f32, limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    value.round().clamp(0.0, (limit - 1) as f32) as u32
}

fn run_pinch(pinch: &Pinch) -> io::Result<()> {
    let mut inward_a = swipe_command(
        pinch.from_a.0,
        pinch.from_a.1,
        pinch.to_a.0,
        pinch.to_a.1,
        pinch.duration_ms,
    );
    let mut inward_b = swipe_command(
        pinch.from_b.0,
        pinch.from_b.1,
        pinch.to_b.0,
        pinch.to_b.1,
        pinch.duration_ms,
    );
    let mut inward_a = inward_a.spawn()?;
    let mut inward_b = inward_b.spawn()?;
    let status_a = inward_a.wait()?;
    let status_b = inward_b.wait()?;
    if status_a.success() && status_b.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "zoom out exited with {status_a} and {status_b}"
        )))
    }
}

fn swipe_command(x1: u32, y1: u32, x2: u32, y2: u32, duration_ms: u32) -> Command {
    let mut command = Command::new("input");
    command.args([
        "swipe",
        &x1.to_string(),
        &y1.to_string(),
        &x2.to_string(),
        &y2.to_string(),
        &duration_ms.to_string(),
    ]);
    command
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

    #[test]
    fn display_size_prefers_the_override() {
        assert_eq!(
            parse_display_size("Physical size: 1440x3120\nOverride size: 1080x2340\n"),
            Some((1080, 2340))
        );
        assert_eq!(
            parse_display_size("Physical size: 1080x2400\n"),
            Some((1080, 2400))
        );
    }

    #[test]
    fn zoom_out_pinch_is_an_inward_swipe() {
        let pinches = zoom_out_pinches(1080, 2400);
        assert_eq!(pinches.len(), ZOOM_PINCHES);
        let pinch = pinches[0];
        assert!(pinch.from_a.0 < pinch.to_a.0);
        assert!(pinch.from_a.1 < pinch.to_a.1);
        assert!(pinch.from_b.0 > pinch.to_b.0);
        assert!(pinch.from_b.1 > pinch.to_b.1);
        let command = swipe_command(
            pinch.from_a.0,
            pinch.from_a.1,
            pinch.to_a.0,
            pinch.to_a.1,
            pinch.duration_ms,
        );
        assert_eq!(command.get_program(), "input");
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "swipe".to_string(),
                pinch.from_a.0.to_string(),
                pinch.from_a.1.to_string(),
                pinch.to_a.0.to_string(),
                pinch.to_a.1.to_string(),
                pinch.duration_ms.to_string(),
            ]
        );
    }
}
