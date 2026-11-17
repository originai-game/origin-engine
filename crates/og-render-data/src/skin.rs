/// Whether a bind's axes are independent, regardless of its units or
/// dequantization scale. A tiny determinant alone does not mean singular.
pub fn valid_bind(bind: glam::Mat4) -> bool {
    let axes = [bind.x_axis, bind.y_axis, bind.z_axis]
        .map(|v| v.truncate().as_dvec3().normalize_or_zero());
    bind.is_finite()
        && glam::DMat3::from_cols(axes[0], axes[1], axes[2])
            .determinant()
            .abs()
            > 1e-6
}

/// The WGSL that defines `joint_matrix` and `morph_delta` (the palette and
/// the morph deltas as read-only storage buffers), in bind group `group`.
pub fn palette_wgsl(group: u32) -> String {
    wgsl!("skin_storage.wgsl").replace("SKIN_GROUP", &group.to_string())
}
