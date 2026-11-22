//! Binary STL of a scene's meshes.

use super::*;

/// The scene's meshes as binary STL, at rest: [`stl_at`] with
/// [`Placement::Rest`].
pub fn stl(scene: &Scene) -> Result<Vec<u8>, ExportError> {
    stl_at(scene, Placement::Rest)
}

/// The scene's meshes as binary STL: an 80-byte header, the triangle
/// count, then per triangle its unit normal and corners, in metres, Z up
/// (as slicers expect), placed at `at`.
pub fn stl_at(scene: &Scene, at: Placement) -> Result<Vec<u8>, ExportError> {
    let parts = meshes(scene, at)?;
    let count: usize = parts.iter().map(|p| p.2.len()).sum();
    let mut out = Vec::with_capacity(84 + 50 * count);
    let posed = if at == Placement::Pose { ", posed" } else { "" };
    let mut header =
        format!("{} - written by og-shape, metres, Z up{posed}", scene.name).into_bytes();
    header.truncate(80);
    header.resize(80, b' ');
    out.extend_from_slice(&header);
    out.extend_from_slice(&(count as u32).to_le_bytes());
    for (_, pos, tris) in &parts {
        for t in tris.iter() {
            let [a, b, c] = t.map(|v| pos[v as usize]);
            let n = face_normal(a, b, c);
            for v in [n, a, b, c] {
                for x in [v.x, v.y, v.z] {
                    push_f32(&mut out, x as f32);
                }
            }
            out.extend_from_slice(&0u16.to_le_bytes());
        }
    }
    Ok(out)
}
