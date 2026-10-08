use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::config::WAYPOINT_PAUSE;
use crate::gpx::{self, GpxRoute};

const FOCUS_PAUSE: Duration = Duration::from_millis(400);
const POKEMON_GO: &str = "com.nianticlabs.pokemongo";

pub fn teleport_to(lat: f64, lon: f64) -> io::Result<()> {
    finish(teleport_command(lat, lon), "teleport")?;
    thread::sleep(FOCUS_PAUSE);
    show_pokemon_go()
}

pub fn show_pokemon_go() -> io::Result<()> {
    finish(show_pokemon_go_command(), "pokemon go")
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
const ZOOM_PINCH_MS: u32 = 450;
const ZOOM_PINCH_GAP: Duration = Duration::from_millis(300);
const ZOOM_JAR: &[u8] = include_bytes!("../assets/zoom/zoomout.jar");

#[derive(Clone, Copy)]
struct Pinch {
    from_a: (u32, u32),
    to_a: (u32, u32),
    from_b: (u32, u32),
    to_b: (u32, u32),
    duration_ms: u32,
}

pub fn ensure_zoomed_out() -> io::Result<()> {
    let jar = write_zoom_jar()?;
    let (width, height) = display_size()?;
    for (index, pinch) in zoom_out_pinches(width, height).iter().enumerate() {
        if index > 0 {
            thread::sleep(ZOOM_PINCH_GAP);
        }
        finish(zoom_out_command(&jar, pinch), "zoom out")?;
    }
    Ok(())
}

fn write_zoom_jar() -> io::Result<PathBuf> {
    let path = PathBuf::from("data/zoomout.jar");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, ZOOM_JAR)?;
    std::fs::canonicalize(&path).or(Ok(path))
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
        duration_ms: ZOOM_PINCH_MS,
    };
    vec![pinch; ZOOM_PINCHES]
}

fn screen_axis(value: f32, limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    value.round().clamp(0.0, (limit - 1) as f32) as u32
}

fn zoom_out_command(jar: &Path, pinch: &Pinch) -> Command {
    let jar = jar.to_string_lossy().into_owned();
    let mut command = Command::new("app_process");
    command.env("CLASSPATH", &jar);
    command.arg(format!("-Djava.class.path={jar}"));
    command.arg("/");
    command.arg("com.autozygarde.ZoomOut");
    for value in [
        pinch.from_a.0,
        pinch.from_a.1,
        pinch.from_b.0,
        pinch.from_b.1,
        pinch.to_a.0,
        pinch.to_a.1,
        pinch.to_b.0,
        pinch.to_b.1,
    ] {
        command.arg(value.to_string());
    }
    command.arg(pinch.duration_ms.to_string());
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

fn show_pokemon_go_command() -> Command {
    let mut command = Command::new("am");
    command.args([
        "start",
        "--activity-single-top",
        "-a",
        "android.intent.action.MAIN",
        "-c",
        "android.intent.category.LAUNCHER",
        POKEMON_GO,
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
    fn show_pokemon_go_command_reopens_the_game() {
        let command = show_pokemon_go_command();
        assert_eq!(command.get_program(), "am");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "start",
                "--activity-single-top",
                "-a",
                "android.intent.action.MAIN",
                "-c",
                "android.intent.category.LAUNCHER",
                "com.nianticlabs.pokemongo",
            ]
            .map(std::ffi::OsStr::new)
        );
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
    fn zoom_out_pinch_is_one_two_finger_gesture() {
        let pinches = zoom_out_pinches(1080, 2400);
        assert_eq!(pinches.len(), ZOOM_PINCHES);
        let pinch = pinches[0];
        let start = span(pinch.from_a, pinch.from_b);
        let end = span(pinch.to_a, pinch.to_b);
        assert!(
            end < start * 0.5,
            "fingers must move together so the map zooms out"
        );
        let start_center = midpoint(pinch.from_a, pinch.from_b);
        let end_center = midpoint(pinch.to_a, pinch.to_b);
        assert!((start_center.0 - end_center.0).abs() < 2.0);
        assert!((start_center.1 - end_center.1).abs() < 2.0);

        let jar = Path::new("data/zoomout.jar");
        let command = zoom_out_command(jar, &pinch);
        assert_eq!(command.get_program(), "app_process");
        let classpath = command
            .get_envs()
            .find(|(key, _)| *key == "CLASSPATH")
            .and_then(|(_, value)| value)
            .map(|value| value.to_string_lossy().into_owned());
        assert_eq!(classpath.as_deref(), Some("data/zoomout.jar"));
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "-Djava.class.path=data/zoomout.jar".to_string(),
                "/".to_string(),
                "com.autozygarde.ZoomOut".to_string(),
                pinch.from_a.0.to_string(),
                pinch.from_a.1.to_string(),
                pinch.from_b.0.to_string(),
                pinch.from_b.1.to_string(),
                pinch.to_a.0.to_string(),
                pinch.to_a.1.to_string(),
                pinch.to_b.0.to_string(),
                pinch.to_b.1.to_string(),
                pinch.duration_ms.to_string(),
            ]
        );
    }

    #[test]
    fn zoom_out_jar_contains_a_dex() {
        assert_eq!(&ZOOM_JAR[..4], b"PK\x03\x04");
        assert!(
            ZOOM_JAR
                .windows(b"classes.dex".len())
                .any(|window| window == b"classes.dex")
        );
    }

    fn span(a: (u32, u32), b: (u32, u32)) -> f32 {
        let dx = a.0 as f32 - b.0 as f32;
        let dy = a.1 as f32 - b.1 as f32;
        (dx * dx + dy * dy).sqrt()
    }

    fn midpoint(a: (u32, u32), b: (u32, u32)) -> (f32, f32) {
        (
            (a.0 as f32 + b.0 as f32) / 2.0,
            (a.1 as f32 + b.1 as f32) / 2.0,
        )
    }
}
