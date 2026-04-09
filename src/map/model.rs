//! OpenTCS plant model in world coordinates (meters).

use std::collections::HashMap;

use crate::config::MapNamePrefixes;

use super::name_prefix::strip_prefix_and_parse_int;

/// A halt / waypoint in world coordinates (m).
#[derive(Debug, Clone)]
pub struct MapPoint {
    pub x_m: f64,
    pub y_m: f64,
}

/// Directed path between two points, with optional polyline in world space (m).
#[derive(Debug, Clone)]
pub struct MapPath {
    #[allow(dead_code)]
    pub name: String,
    pub source: String,
    pub dest: String,
    /// Declared OpenTCS length (mm); geometry uses point coordinates / polyline.
    #[allow(dead_code)]
    pub length_mm: f64,
    /// OpenTCS max velocity along path (typically mm/s).
    pub max_velocity_mm_s: f64,
    /// At least two points: start → end, possibly via layout-mapped control points.
    pub polyline_world_m: Vec<(f64, f64)>,
}

/// Parsed driving course.
#[derive(Debug, Clone)]
pub struct MapModel {
    pub points: HashMap<String, MapPoint>,
    pub paths: HashMap<String, MapPath>,
    /// From `[map.name_prefixes]` after load; default is no stripping.
    pub name_prefixes: MapNamePrefixes,
}

impl Default for MapModel {
    fn default() -> Self {
        Self {
            points: HashMap::new(),
            paths: HashMap::new(),
            name_prefixes: MapNamePrefixes::default(),
        }
    }
}

impl MapModel {
    fn resolve_point_key(&self, name: &str) -> Option<String> {
        if self.points.contains_key(name) {
            return Some(name.to_string());
        }
        if !self.name_prefixes.apply_stripping {
            return None;
        }
        let id = strip_prefix_and_parse_int(name, &self.name_prefixes.point_prefix);
        if id <= 0 {
            return None;
        }
        let canonical = format!("{}{}", self.name_prefixes.point_prefix, id);
        if self.points.contains_key(&canonical) {
            Some(canonical)
        } else {
            None
        }
    }

    fn resolve_path_key(&self, name: &str) -> Option<String> {
        if self.paths.contains_key(name) {
            return Some(name.to_string());
        }
        if !self.name_prefixes.apply_stripping {
            return None;
        }
        let id = strip_prefix_and_parse_int(name, &self.name_prefixes.path_prefix);
        if id <= 0 {
            return None;
        }
        let canonical = format!("{}{}", self.name_prefixes.path_prefix, id);
        if self.paths.contains_key(&canonical) {
            Some(canonical)
        } else {
            None
        }
    }

    pub fn point_world(&self, name: &str) -> Option<(f64, f64)> {
        let key = self.resolve_point_key(name)?;
        self.points.get(&key).map(|p| (p.x_m, p.y_m))
    }

    /// Resolve geometry: OpenTCS path `name` == VDA `edge_id`, else match `(source, dest)` to order nodes.
    pub fn path_for_edge(&self, edge_id: &str, start_node_id: &str, end_node_id: &str) -> Option<&MapPath> {
        if let Some(k) = self.resolve_path_key(edge_id) {
            if let Some(p) = self.paths.get(&k) {
                return Some(p);
            }
        }
        let start = self.resolve_point_key(start_node_id)?;
        let end = self.resolve_point_key(end_node_id)?;
        self.paths
            .values()
            .find(|p| p.source == start && p.dest == end)
    }
}
