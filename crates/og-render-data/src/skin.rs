/// The WGSL that defines `joint_matrix` and `morph_delta` (the palette and
/// the morph deltas as read-only storage buffers), in bind group `group`.
pub fn palette_wgsl(group: u32) -> String {
    wgsl!("skin_storage.wgsl").replace("SKIN_GROUP", &group.to_string())
}
