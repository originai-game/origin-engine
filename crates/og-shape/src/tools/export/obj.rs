//! Wavefront OBJ of a scene's meshes.

use super::*;

/// The scene's meshes as Wavefront OBJ text, at rest: [`obj_at`] with
/// [`Placement::Rest`].
pub fn obj(scene: &Scene) -> Result<String, ExportError> {
    obj_at(scene, Placement::Rest)
}

/// The scene's meshes as Wavefront OBJ text: one `o` per part, in metres,
/// Y up (the GLB's axes), coordinates as f32, placed at `at`. No normals,
/// materials or joints.
pub fn obj_at(scene: &Scene, at: Placement) -> Result<String, ExportError> {
    let posed = if at == Placement::Pose { ", posed" } else { "" };
    let mut out = format!(
        "# {} - written by og-shape, metres, Y up{posed}\n",
        scene.name
    );
    let mut base = 1u64;
    for (name, pos, tris) in meshes(scene, at)? {
        out.push_str(&format!("o {name}\n"));
        for p in &pos {
            let q = y_up(*p);
            out.push_str(&format!("v {} {} {}\n", q.x as f32, q.y as f32, q.z as f32));
        }
        for t in tris {
            out.push_str(&format!(
                "f {} {} {}\n",
                base + u64::from(t[0]),
                base + u64::from(t[1]),
                base + u64::from(t[2])
            ));
        }
        base += pos.len() as u64;
    }
    Ok(out)
}
