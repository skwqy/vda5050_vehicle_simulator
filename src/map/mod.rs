mod layout;
mod model;
mod name_prefix;
mod opentcs_xml;

#[allow(unused_imports)]
pub use model::{MapModel, MapPoint};
pub use opentcs_xml::parse_opentcs_model_xml;

use std::fs;
use std::sync::Arc;

use thiserror::Error;

use crate::config::MapConfig;

#[derive(Debug, Error)]
pub enum MapError {
    #[error("XML: {0}")]
    Xml(String),
    #[error("IO: {0}")]
    Io(String),
}

/// Load driving course from configured path (relative to cwd unless absolute).
pub fn load_map_from_config(cfg: &MapConfig) -> Result<MapModel, MapError> {
    validate_map_name_prefixes(&cfg.name_prefixes)?;
    let text = fs::read_to_string(&cfg.xml_path).map_err(|e| {
        MapError::Io(format!("{}: {e}", cfg.xml_path))
    })?;
    let mut model = parse_opentcs_model_xml(&text, cfg.layout_scale_mm, cfg.layout_flip_y)?;
    model.name_prefixes = cfg.name_prefixes.clone();
    Ok(model)
}

fn validate_map_name_prefixes(p: &crate::config::MapNamePrefixes) -> Result<(), MapError> {
    if !p.apply_stripping {
        return Ok(());
    }
    if !p.point_prefix.ends_with('_') {
        return Err(MapError::Io(format!(
            "map.name_prefixes.point_prefix must end with '_', got: {}",
            p.point_prefix
        )));
    }
    if !p.path_prefix.ends_with('_') {
        return Err(MapError::Io(format!(
            "map.name_prefixes.path_prefix must end with '_', got: {}",
            p.path_prefix
        )));
    }
    Ok(())
}

/// Convenience for main: `Arc` only on success.
pub fn load_map_arc(cfg: &MapConfig) -> Result<Arc<MapModel>, MapError> {
    load_map_from_config(cfg).map(Arc::new)
}
