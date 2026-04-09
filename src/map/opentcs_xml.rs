//! Minimal OpenTCS 6.0 `model` XML parser (points + paths + pathLayout control points).

use std::collections::HashMap;

use quick_xml::events::Event;
use quick_xml::Reader;

use super::layout::{polyline_from_layout, preprocess_layout_xy};
use super::model::{MapModel, MapPath, MapPoint};

use super::MapError;

#[derive(Default)]
struct PathBuild {
    name: String,
    source: String,
    dest: String,
    length_mm: f64,
    max_velocity_mm_s: f64,
    layout_cps: Vec<(f64, f64)>,
}

pub fn parse_opentcs_model_xml(
    xml: &str,
    layout_scale: f64,
    layout_flip_y: bool,
) -> Result<MapModel, MapError> {
    let mut reader = Reader::from_reader(xml.as_bytes());
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut points: HashMap<String, MapPoint> = HashMap::new();
    let mut paths: HashMap<String, MapPath> = HashMap::new();

    let mut inside_point = false;
    let mut point_name = String::new();
    let mut px: f64 = 0.0;
    let mut py: f64 = 0.0;

    let mut inside_path = false;
    let mut path_build = PathBuild::default();
    let mut inside_path_layout = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"point" => {
                    inside_point = true;
                    read_point_attrs(e, &mut point_name, &mut px, &mut py);
                }
                b"path" => {
                    inside_path = true;
                    path_build = PathBuild::default();
                    read_path_attrs(e, &mut path_build);
                }
                b"pathLayout" => {
                    if inside_path {
                        inside_path_layout = true;
                        path_build.layout_cps.clear();
                    }
                }
                b"controlPoint" if inside_path_layout => {
                    let (cx, cy) = read_control_point_attrs(e);
                    let (lx, ly) = preprocess_layout_xy(cx, cy, layout_scale, layout_flip_y);
                    path_build.layout_cps.push((lx, ly));
                }
                _ => {}
            },
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"point" => {
                    let mut n = String::new();
                    let mut x = 0.0f64;
                    let mut y = 0.0f64;
                    read_point_attrs(e, &mut n, &mut x, &mut y);
                    if !n.is_empty() {
                        points.insert(
                            n,
                            MapPoint {
                                x_m: x * 1e-3,
                                y_m: y * 1e-3,
                            },
                        );
                    }
                }
                b"controlPoint" if inside_path_layout => {
                    let (cx, cy) = read_control_point_attrs(e);
                    let (lx, ly) = preprocess_layout_xy(cx, cy, layout_scale, layout_flip_y);
                    path_build.layout_cps.push((lx, ly));
                }
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"point" => {
                    if inside_point && !point_name.is_empty() {
                        points.insert(
                            point_name.clone(),
                            MapPoint {
                                x_m: px * 1e-3,
                                y_m: py * 1e-3,
                            },
                        );
                    }
                    inside_point = false;
                }
                b"pathLayout" => inside_path_layout = false,
                b"path" => {
                    if inside_path {
                        let pb = std::mem::take(&mut path_build);
                        let polyline = finish_path_polyline(&points, &pb);
                        if !pb.name.is_empty() && polyline.len() >= 2 {
                            paths.insert(
                                pb.name.clone(),
                                MapPath {
                                    name: pb.name,
                                    source: pb.source,
                                    dest: pb.dest,
                                    length_mm: pb.length_mm,
                                    max_velocity_mm_s: pb.max_velocity_mm_s,
                                    polyline_world_m: polyline,
                                },
                            );
                        }
                    }
                    inside_path = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(MapError::Xml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(MapModel {
        points,
        paths,
        name_prefixes: crate::config::MapNamePrefixes::default(),
    })
}

fn read_point_attrs(
    e: &quick_xml::events::BytesStart<'_>,
    name: &mut String,
    px: &mut f64,
    py: &mut f64,
) {
    name.clear();
    for a in e.attributes().flatten() {
        let k = String::from_utf8_lossy(a.key.as_ref());
        let v = String::from_utf8_lossy(&a.value);
        match k.as_ref() {
            "name" => *name = v.into_owned(),
            "positionX" => *px = v.parse().unwrap_or(0.0),
            "positionY" => *py = v.parse().unwrap_or(0.0),
            _ => {}
        }
    }
}

fn read_path_attrs(e: &quick_xml::events::BytesStart<'_>, pb: &mut PathBuild) {
    for a in e.attributes().flatten() {
        let k = String::from_utf8_lossy(a.key.as_ref());
        let v = String::from_utf8_lossy(&a.value);
        match k.as_ref() {
            "name" => pb.name = v.into_owned(),
            "sourcePoint" => pb.source = v.into_owned(),
            "destinationPoint" => pb.dest = v.into_owned(),
            "length" => pb.length_mm = v.parse().unwrap_or(0.0),
            "maxVelocity" => pb.max_velocity_mm_s = v.parse().unwrap_or(0.0),
            _ => {}
        }
    }
}

fn read_control_point_attrs(e: &quick_xml::events::BytesStart<'_>) -> (f64, f64) {
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    for a in e.attributes().flatten() {
        let k = String::from_utf8_lossy(a.key.as_ref());
        let v = String::from_utf8_lossy(&a.value);
        match k.as_ref() {
            "x" => cx = v.parse().unwrap_or(0.0),
            "y" => cy = v.parse().unwrap_or(0.0),
            _ => {}
        }
    }
    (cx, cy)
}

fn finish_path_polyline(points: &HashMap<String, MapPoint>, pb: &PathBuild) -> Vec<(f64, f64)> {
    let Some(s_pt) = points.get(&pb.source) else {
        return Vec::new();
    };
    let Some(d_pt) = points.get(&pb.dest) else {
        return Vec::new();
    };
    let s_world = (s_pt.x_m, s_pt.y_m);
    let d_world = (d_pt.x_m, d_pt.y_m);

    if pb.layout_cps.len() >= 2 {
        polyline_from_layout(&pb.layout_cps, s_world, d_world)
    } else {
        vec![s_world, d_world]
    }
}
