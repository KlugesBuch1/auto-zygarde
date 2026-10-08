use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct GpxRoute {
    pub path: PathBuf,
    pub start_lat: f64,
    pub start_lon: f64,
    pub points: Vec<(f64, f64)>,
}

pub fn load_routes(dir: &Path) -> io::Result<Vec<GpxRoute>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("gpx") {
            names.push(path);
        }
    }
    names.sort();
    let mut routes = Vec::with_capacity(names.len());
    for path in names {
        let text = fs::read_to_string(&path)?;
        let points = track_points(&text);
        let Some(&(start_lat, start_lon)) = points.first() else {
            return Err(io::Error::other(format!(
                "{} has no track points",
                path.display()
            )));
        };
        routes.push(GpxRoute {
            path,
            start_lat,
            start_lon,
            points,
        });
    }
    Ok(routes)
}

pub fn track_points(xml: &str) -> Vec<(f64, f64)> {
    for kind in ["trkpt", "rtept", "wpt"] {
        let points = points_named(xml, kind);
        if !points.is_empty() {
            return points;
        }
    }
    Vec::new()
}

fn points_named(xml: &str, kind: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    for piece in xml.split('<').skip(1) {
        let name = piece
            .split_whitespace()
            .next()
            .unwrap_or("")
            .rsplit(':')
            .next()
            .unwrap_or("");
        if name != kind {
            continue;
        }
        let Some(lat) = attr(piece, "lat") else {
            continue;
        };
        let Some(lon) = attr(piece, "lon") else {
            continue;
        };
        points.push((lat, lon));
    }
    points
}

fn attr(tag: &str, key: &str) -> Option<f64> {
    for quote in ['"', '\''] {
        let needle = format!("{key}={quote}");
        let Some(start) = tag.find(&needle) else {
            continue;
        };
        let value = &tag[start + needle.len()..];
        let end = value.find(quote)?;
        return value[..end].parse().ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_track_points_and_keeps_order() {
        let xml = r#"
            <gpx>
              <wpt lat="1" lon="2"/>
              <trk><trkseg>
                <trkpt lon="11.25" lat="48.5"/>
                <trkpt lat="48.75" lon="11.5"/>
              </trkseg></trk>
            </gpx>
        "#;
        assert_eq!(track_points(xml), vec![(48.5, 11.25), (48.75, 11.5)]);
    }

    #[test]
    fn loads_files_in_name_order() {
        let dir = std::env::temp_dir().join(format!("auto-zygarde-gpx-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("b.gpx"), "<gpx><trkpt lat=\"2\" lon=\"3\"/></gpx>").unwrap();
        fs::write(
            dir.join("a.gpx"),
            "<gpx><trkpt lat=\"1.5\" lon=\"4\"/></gpx>",
        )
        .unwrap();
        fs::write(dir.join("note.txt"), "nope").unwrap();
        let routes = load_routes(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].start_lat, 1.5);
        assert_eq!(routes[1].start_lon, 3.0);
    }
}
