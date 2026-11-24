//! MagicaVoxel `.vox` → glTF: a voxel model becomes a flat-shaded, greedy-
//! meshed glTF binary in memory, which the model cook then treats like any
//! `.glb` (meshoptimizer, LODs, the node graph), so a voxel asset is a
//! model asset with vertex colours and one material.
//!
//! What is read: `SIZE`/`XYZI` model pairs (every model of a `PACK`), the
//! `RGBA` palette (MagicaVoxel's default palette when the file has none),
//! and the scene graph (`nTRN` translations, `nGRP`, `nSHP`) so a multi-part
//! file keeps each part where the editor put it; a file without a graph
//! places each model at the origin about its pivot, exactly where a graph
//! of one identity transform puts it (ogt_vox's reading; MagicaVoxel
//! centres a lone model). Rotations in `nTRN` (the `_r`
//! frame byte) are applied as the signed axis permutation they encode; one
//! with a negative determinant mirrors the model, so its triangles' winding
//! is flipped to stay counter-clockwise from outside.
//!
//! Axes: MagicaVoxel is z-up with y away from the viewer; the mesh is y-up
//! (glTF): `(x, y, z)` → `(x, z, -y)`, the model's origin at its `nTRN`
//! translation. The pivot is MagicaVoxel's: `floor(size / 2)` (ogt_vox's
//! documented convention, which vengi's importer matches), so voxel `v` of a
//! model turned by `R` at translation `T` fills the cell whose lowest corner
//! is `floor(T + R (v + 1/2 - floor(size / 2)))`; with a whole-voxel pivot
//! and a signed axis permutation that is the box `T + R ([v, v + 1] -
//! floor(size / 2))` the mesh is built from, turned or mirrored, odd sizes
//! included (`tests/fixtures/vox/pivot.vox`: five turns and mirrors of a
//! 3 x 2 x 5 model, checked voxel by voxel). The import settings' `model { voxScale
//! voxOrigin }` ([`VoxSettings`]) give the metres per voxel (default 1) and
//! the origin: `center` (the default) where the editor puts it, `bottom`
//! the whole file lifted so its lowest voxel stands on y = 0.
//!
//! Animation: MagicaVoxel's frames are models swapped in time, an `nSHP`
//! listing several models each with its `_f` key frame (shown from that
//! frame to the next key), or, in a file without a graph, several models
//! (0.98's `PACK` animation: model `i` is frame `i`). Such a file cooks to
//! a glTF of mesh swaps: each animated model a mesh node of its own, and
//! per clip a visibility track per model's mesh node (the engine's
//! `OG_animation_visibility`, 1 on the frames its model shows, 0
//! elsewhere; the mesh nodes not shown on frame 0 hidden at rest with
//! `KHR_node_visibility`), so `og.AnimationPlayer` plays it as any clip
//! and a frame folded away is not drawn at all. A frame that never moves
//! has no joint (no palette matrix; drawn as a static instance when
//! shown); only a model a keyed transform moves is skinned. `model { voxFps 8 voxClip walk 0 3 }` gives the frame
//! rate (default 10) and named frame ranges (default one clip `frames` of
//! every frame). `nTRN` key frames (an `_f` per frame of a transform, each
//! held until the next, as ogt_vox samples them) move what is under the
//! transform: a model whose place changes across frames is skinned whole
//! to a joint of its own with a STEP translation channel as well, and a key
//! that turns it starts a new mesh (a turn is a different set of voxel
//! faces), swapped in like a frame's model. A node's `_loop` (on an
//! `nTRN` or an `nSHP`) repeats its keys from the first to the last on
//! every frame outside them, as the editor and ogt_vox
//! (`compute_looped_frame_index`) play it; the scene's playback range
//! (`META`'s `_anim_range`, MagicaVoxel 0.99.7.2) sets the frames walked
//! (to its last) and the default clip (its frames alone). The editor's
//! frame rate is not in the file (no chunk of the format carries one; the
//! playback range is the one animation setting the editor writes, and
//! ogt_vox reads no rate either): `voxFps` gives it.
//!
//! Greedy meshing: per face direction, per slice, runs of voxels of one
//! colour merge into rectangles, so a 32³ block of one colour is 12
//! triangles. Each rectangle is four unwelded vertices with the face normal
//! and the palette colour (`COLOR_0`, linear), so the mesh is flat-shaded
//! whatever the material says. The one material is white, metalness 0,
//! roughness 0.9: `MaterialStyle` or the game's material replaces it.

use std::collections::BTreeMap;

mod export;
mod meshing;
mod parser;
mod settings;

pub use export::*;
pub(crate) use meshing::*;
pub use parser::*;
pub use settings::*;

/// The visibility tracks' glTF extension (`super::anim::VISIBILITY_EXTENSION`).
const VISIBILITY: &str = super::anim::VISIBILITY_EXTENSION;

/// sRGB byte to linear.
fn linear(c: u8) -> f32 {
    let v = c as f32 / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// The quads a `.vox` meshes to, for tests and reports: (quads, triangles).
pub fn quad_count(bytes: &[u8]) -> Result<(usize, usize), String> {
    let file = parse(bytes)?;
    let q: usize = file
        .placements
        .iter()
        .map(|p| mesh_model(&file.models[p.model]).len())
        .sum();
    Ok((q, q * 2))
}
