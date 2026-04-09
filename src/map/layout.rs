//! Map layout control points (OpenTCS pathLayout) into world meters using independent XY affine stretch
//! from first→last layout CP onto source→destination point coordinates.

/// Apply optional scale (layout unit × scale as position in mm space before affine) and flip Y.
pub fn preprocess_layout_xy(x: f64, y: f64, scale: f64, flip_y: bool) -> (f64, f64) {
    let y = if flip_y { -y } else { y };
    (x * scale, y * scale)
}

/// Map one layout (lx, ly) into world meters so first maps to `s_world` and last maps to `d_world`
/// using separate linear interpolation on X and Y (handles typical plant editor layouts).
pub fn layout_to_world_xy(
    lx: f64,
    ly: f64,
    l_first: (f64, f64),
    l_last: (f64, f64),
    s_world: (f64, f64),
    d_world: (f64, f64),
) -> (f64, f64) {
    let (lfx, lfy) = l_first;
    let (llx, lly) = l_last;
    let (sx, sy) = s_world;
    let (dx, dy) = d_world;

    let wx = if (llx - lfx).abs() > 1e-9 {
        sx + (lx - lfx) / (llx - lfx) * (dx - sx)
    } else {
        sx
    };
    let wy = if (lly - lfy).abs() > 1e-9 {
        sy + (ly - lfy) / (lly - lfy) * (dy - sy)
    } else {
        sy
    };
    (wx, wy)
}

/// Build world polyline: layout CPs mapped with affine; if fewer than 2 CPs, returns `[s_world, d_world]`.
pub fn polyline_from_layout(
    layout_points: &[(f64, f64)],
    s_world: (f64, f64),
    d_world: (f64, f64),
) -> Vec<(f64, f64)> {
    if layout_points.len() < 2 {
        return vec![s_world, d_world];
    }
    let first = layout_points[0];
    let last = *layout_points.last().unwrap();
    layout_points
        .iter()
        .map(|&(lx, ly)| layout_to_world_xy(lx, ly, first, last, s_world, d_world))
        .collect()
}
