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
pub(super) use meshing::*;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A `.vox` of one model: `size`, its voxels, an optional palette.
    pub fn write(size: [u32; 3], voxels: &[[u8; 4]], palette: Option<&[[u8; 4]]>) -> Vec<u8> {
        let mut children = Vec::new();
        let chunk = |out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]| {
            out.extend_from_slice(id);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(body);
        };
        let mut size_body = Vec::new();
        for s in size {
            size_body.extend_from_slice(&s.to_le_bytes());
        }
        chunk(&mut children, b"SIZE", &size_body);
        let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
        for v in voxels {
            xyzi.extend_from_slice(v);
        }
        chunk(&mut children, b"XYZI", &xyzi);
        if let Some(p) = palette {
            let mut body = Vec::new();
            for i in 0..256 {
                body.extend_from_slice(&p.get(i).copied().unwrap_or([0; 4]));
            }
            chunk(&mut children, b"RGBA", &body);
        }
        let mut out = b"VOX ".to_vec();
        out.extend_from_slice(&150u32.to_le_bytes());
        out.extend_from_slice(b"MAIN");
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(children.len() as u32).to_le_bytes());
        out.extend_from_slice(&children);
        out
    }

    #[test]
    fn a_solid_block_of_one_colour_is_six_quads() {
        let mut voxels = Vec::new();
        for z in 0..4u8 {
            for y in 0..3u8 {
                for x in 0..5u8 {
                    voxels.push([x, y, z, 7]);
                }
            }
        }
        let bytes = write([5, 3, 4], &voxels, None);
        assert_eq!(quad_count(&bytes).unwrap(), (6, 12));
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        assert_eq!(&glb[0..4], b"glTF");
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let mesh = g.document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();
        assert!(prim.get(&gltf::Semantic::Colors(0)).is_some());
        assert_eq!(prim.get(&gltf::Semantic::Positions).unwrap().count(), 24);
    }

    #[test]
    fn two_colours_split_a_face_and_hidden_faces_are_dropped() {
        // Two voxels side by side along x of different colours: 10 faces
        // (the shared face is hidden), the +z face is two quads.
        let bytes = write([2, 1, 1], &[[0, 0, 0, 1], [1, 0, 0, 2]], None);
        assert_eq!(quad_count(&bytes).unwrap().0, 10);
        // The same colour: one block, six quads.
        let bytes = write([2, 1, 1], &[[0, 0, 0, 1], [1, 0, 0, 1]], None);
        assert_eq!(quad_count(&bytes).unwrap().0, 6);
    }

    #[test]
    fn the_palette_colours_the_vertices_and_the_frame_is_y_up() {
        let mut palette = [[0u8; 4]; 256];
        palette[1] = [255, 0, 0, 255];
        let bytes = write(
            [1, 1, 3],
            &[[0, 0, 0, 1], [0, 0, 1, 1], [0, 0, 2, 1]],
            Some(&palette[1..]),
        );
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let mesh = g.document.meshes().next().unwrap();
        let prim = mesh.primitives().next().unwrap();
        let reader = prim.reader(|_| Some(&blob[..]));
        let pos: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        // A column of three voxels stands along y (glTF up), 3 m tall.
        let (lo, hi) = pos
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), p| (l.min(p[1]), h.max(p[1])));
        assert_eq!((lo, hi), (-1.0, 2.0));
        let colors: Vec<[f32; 4]> = reader.read_colors(0).unwrap().into_rgba_f32().collect();
        assert!(
            colors
                .iter()
                .all(|c| c[0] > 0.99 && c[1] < 0.01 && c[2] < 0.01)
        );
    }

    /// A `.vox` of one model placed by a scene graph: root transform 0 →
    /// group 1 → per placement a transform (`_t`, `_r`) over a shape of
    /// model 0.
    fn write_placed(
        size: [u32; 3],
        voxels: &[[u8; 4]],
        placements: &[([i32; 3], Option<u8>)],
    ) -> Vec<u8> {
        write_placed_with(size, voxels, None, placements)
    }

    /// [`write_placed`] with a palette (`RGBA`, entry 0 for index 1).
    fn write_placed_with(
        size: [u32; 3],
        voxels: &[[u8; 4]],
        palette: Option<&[[u8; 4]]>,
        placements: &[([i32; 3], Option<u8>)],
    ) -> Vec<u8> {
        let mut out = write(size, voxels, palette);
        let mut extra = Vec::new();
        let chunk = |out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]| {
            out.extend_from_slice(id);
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(body);
        };
        let string = |b: &mut Vec<u8>, s: &str| {
            b.extend_from_slice(&(s.len() as u32).to_le_bytes());
            b.extend_from_slice(s.as_bytes());
        };
        let transform = |node: u32, child: u32, frame: &[(&str, String)]| {
            let mut b = Vec::new();
            b.extend_from_slice(&node.to_le_bytes());
            b.extend_from_slice(&0u32.to_le_bytes()); // attributes
            b.extend_from_slice(&child.to_le_bytes());
            b.extend_from_slice(&(-1i32).to_le_bytes()); // reserved
            b.extend_from_slice(&0u32.to_le_bytes()); // layer
            b.extend_from_slice(&1u32.to_le_bytes()); // one frame
            b.extend_from_slice(&(frame.len() as u32).to_le_bytes());
            for (k, v) in frame {
                string(&mut b, k);
                string(&mut b, v);
            }
            b
        };
        chunk(&mut extra, b"nTRN", &transform(0, 1, &[]));
        let mut group = Vec::new();
        group.extend_from_slice(&1u32.to_le_bytes());
        group.extend_from_slice(&0u32.to_le_bytes());
        group.extend_from_slice(&(placements.len() as u32).to_le_bytes());
        for i in 0..placements.len() as u32 {
            group.extend_from_slice(&(2 + 2 * i).to_le_bytes());
        }
        chunk(&mut extra, b"nGRP", &group);
        for (i, (t, r)) in placements.iter().enumerate() {
            let node = 2 + 2 * i as u32;
            let mut frame = vec![("_t", format!("{} {} {}", t[0], t[1], t[2]))];
            if let Some(r) = r {
                frame.push(("_r", r.to_string()));
            }
            chunk(&mut extra, b"nTRN", &transform(node, node + 1, &frame));
            let mut shape = Vec::new();
            shape.extend_from_slice(&(node + 1).to_le_bytes());
            shape.extend_from_slice(&0u32.to_le_bytes());
            shape.extend_from_slice(&1u32.to_le_bytes());
            shape.extend_from_slice(&0u32.to_le_bytes()); // model 0
            shape.extend_from_slice(&0u32.to_le_bytes()); // its attributes
            chunk(&mut extra, b"nSHP", &shape);
        }
        out.extend_from_slice(&extra);
        // MAIN's children size.
        let children = (out.len() - 20) as u32;
        out[16..20].copy_from_slice(&children.to_le_bytes());
        out
    }

    /// A primitive's positions, normals and triangles.
    type Primitive = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>);

    /// Per primitive of a glTF binary: its positions, normals and triangles.
    fn primitives(glb: &[u8]) -> Vec<Primitive> {
        let g = gltf::Gltf::from_slice(glb).unwrap();
        let blob = g.blob.clone().unwrap();
        g.document
            .meshes()
            .flat_map(|m| m.primitives().collect::<Vec<_>>())
            .map(|p| {
                let r = p.reader(|_| Some(&blob[..]));
                let pos: Vec<[f32; 3]> = r.read_positions().unwrap().collect();
                let nrm: Vec<[f32; 3]> = r.read_normals().unwrap().collect();
                let idx: Vec<u32> = r.read_indices().unwrap().into_u32().collect();
                let tris = idx.chunks(3).map(|t| [t[0], t[1], t[2]]).collect();
                (pos, nrm, tris)
            })
            .collect()
    }

    /// Every triangle's winding (counter-clockwise) faces along its normal.
    fn faces_out(glb: &[u8]) -> bool {
        primitives(glb).iter().all(|(pos, nrm, tris)| {
            tris.iter().all(|t| {
                let v = |i: u32| glam::Vec3::from(pos[i as usize]);
                let n = glam::Vec3::from(nrm[t[0] as usize]);
                (v(t[1]) - v(t[0])).cross(v(t[2]) - v(t[0])).dot(n) > 0.0
            })
        })
    }

    #[test]
    fn determinants_of_signed_axis_permutations() {
        // Identity, a swap of x and y, one sign, two signs, a cycle.
        assert_eq!(determinant(IDENTITY), 1);
        assert_eq!(determinant([(1, 1), (0, 1), (2, 1)]), -1);
        assert_eq!(determinant([(0, -1), (1, 1), (2, 1)]), -1);
        assert_eq!(determinant([(0, -1), (1, -1), (2, 1)]), 1);
        assert_eq!(determinant([(1, 1), (2, 1), (0, 1)]), 1);
        // The `_r` byte 4 is the identity; 20 flips x; 1 swaps x and y.
        assert_eq!(rotation_of(4), IDENTITY);
        assert_eq!(determinant(rotation_of(20)), -1);
        assert_eq!(rotation_of(1)[0..2], [(1, 1), (0, 1)]);
        assert_eq!(determinant(rotation_of(1)), -1);
        // Not a rotation: an axis twice.
        assert_eq!(rotation_of(0), IDENTITY);
    }

    /// Every one of the 48 signed axis permutations a frame's `_r` can
    /// hold keeps every face's winding counter-clockwise from outside: the
    /// 24 mirrors flip it back (without the flip, they faced inwards).
    #[test]
    fn mirrors_flip_the_winding_and_every_frame_faces_out() {
        let voxels = [[0, 0, 0, 1], [1, 0, 0, 2], [0, 1, 0, 3], [0, 0, 1, 4]];
        let mut mirrors = 0;
        let mut seen = std::collections::BTreeSet::new();
        for r in 0u8..128 {
            let rot = rotation_of(r);
            if r & 3 == (r >> 2) & 3 || r & 3 == 3 || (r >> 2) & 3 == 3 || !seen.insert(rot) {
                continue;
            }
            let bytes = write_placed([2, 2, 2], &voxels, &[([3, -2, 5], Some(r))]);
            let file = parse(&bytes).unwrap();
            assert_eq!(file.placements[0].rotation, rot);
            mirrors += (determinant(rot) < 0) as usize;
            let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
            assert!(faces_out(&glb), "_r {r} ({rot:?}) turns faces inwards");
        }
        assert_eq!((seen.len(), mirrors), (48, 24));
    }

    /// The pivot fixture (`tests/fixtures/vox/pivot.vox`, made by
    /// [`pivot_fixture`]): one 3 x 2 x 5 model (odd in x and z) of five
    /// isolated voxels, each its own colour, placed five times: as is, turned
    /// 90 degrees about z (`_r` 17), mirrored in x (20), turned 180 degrees
    /// about y (84) and the axes cycled (9), each at its own translation.
    const PIVOT_VOXELS: [[u8; 4]; 5] = [
        [0, 0, 0, 1],
        [2, 0, 0, 2],
        [0, 1, 2, 3],
        [1, 0, 3, 4],
        [2, 1, 4, 5],
    ];
    const PIVOT_PLACEMENTS: [([i32; 3], Option<u8>); 5] = [
        ([0, 0, 0], None),
        ([10, 0, 0], Some(17)),
        ([0, 10, 0], Some(20)),
        ([0, 0, 10], Some(84)),
        ([-10, 3, -7], Some(9)),
    ];

    fn pivot_fixture() -> Vec<u8> {
        let palette: Vec<[u8; 4]> = (1..=5u8)
            .map(|i| [i * 40, 255 - i * 40, i * 20, 255])
            .collect();
        write_placed_with([3, 2, 5], &PIVOT_VOXELS, Some(&palette), &PIVOT_PLACEMENTS)
    }

    /// MagicaVoxel's placement of a voxel (ogt_vox's documented pivot, which
    /// vengi's importer matches): a model's pivot is `floor(size / 2)`, and
    /// voxel `v` of a model turned by `R` at translation `T` occupies the
    /// world cell whose lowest corner is `floor(T + R (v + 1/2 - pivot))`.
    /// The pivot is a whole voxel and `R` a signed axis permutation, so the
    /// centre lands on a half and the cell is `T + R (v + 1/2 - pivot) - 1/2`.
    fn magicavoxel_cell(size: [u32; 3], v: [u8; 3], t: [i32; 3], r: [(usize, i32); 3]) -> [i32; 3] {
        let centre = |i: usize| 2 * v[i] as i32 + 1 - 2 * (size[i] / 2) as i32;
        std::array::from_fn(|i| {
            let turned = centre(r[i].0) * r[i].1;
            (2 * t[i] + turned).div_euclid(2)
        })
    }

    /// Every voxel of every turned or mirrored placement of an odd-sized
    /// model lands where MagicaVoxel puts it: the committed fixture (made
    /// by the test writer) is meshed and each colour's box compared with
    /// [`magicavoxel_cell`] taken to the glTF frame.
    #[test]
    fn turned_and_mirrored_odd_models_keep_magicavoxels_pivot() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vox/pivot.vox");
        let made = pivot_fixture();
        if std::env::var_os("OG_WRITE_FIXTURES").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &made).unwrap();
        }
        let bytes = std::fs::read(&path).expect("the committed pivot fixture");
        assert_eq!(bytes, made, "the fixture is the test writer's");
        let file = parse(&bytes).unwrap();
        assert_eq!(file.placements.len(), PIVOT_PLACEMENTS.len());
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let nodes: Vec<_> = g.document.nodes().collect();
        assert_eq!(nodes.len(), PIVOT_PLACEMENTS.len());
        let palette = &file.palette;
        for (node, (placement, (t, r))) in nodes
            .iter()
            .zip(file.placements.iter().zip(PIVOT_PLACEMENTS))
        {
            let rot = r.map_or(IDENTITY, rotation_of);
            assert_eq!(placement.rotation, rot);
            let at = glam::Vec3::from(node.transform().decomposed().0);
            let prim = node.mesh().unwrap().primitives().next().unwrap();
            let reader = prim.reader(|_| Some(&blob[..]));
            let pos: Vec<glam::Vec3> = reader
                .read_positions()
                .unwrap()
                .map(glam::Vec3::from)
                .collect();
            let col: Vec<[f32; 4]> = reader.read_colors(0).unwrap().into_rgba_f32().collect();
            for v in PIVOT_VOXELS {
                let c = palette[v[3] as usize];
                let want = [linear(c[0]), linear(c[1]), linear(c[2])];
                let (mut lo, mut hi) = (glam::Vec3::MAX, glam::Vec3::MIN);
                for (p, k) in pos.iter().zip(&col) {
                    if (0..3).all(|i| (k[i] - want[i]).abs() < 1e-6) {
                        lo = lo.min(at + *p);
                        hi = hi.max(at + *p);
                    }
                }
                let m = magicavoxel_cell([3, 2, 5], [v[0], v[1], v[2]], t, rot);
                // MagicaVoxel's cell [m, m + 1] in glTF's frame: (x, z, -y).
                let cell_lo = glam::Vec3::new(m[0] as f32, m[2] as f32, -(m[1] as f32) - 1.0);
                assert_eq!(
                    (lo, hi),
                    (cell_lo, cell_lo + glam::Vec3::ONE),
                    "voxel {v:?} of the placement at {t:?} turned by _r {r:?}"
                );
            }
        }
    }

    #[test]
    fn vox_scale_and_bottom_origin_place_the_mesh() {
        // A column of three voxels, placed 10 voxels up (MagicaVoxel z).
        let column = [[0, 0, 0, 1], [0, 0, 1, 1], [0, 0, 2, 1]];
        let bytes = write_placed([1, 1, 3], &column, &[([0, 0, 10], None)]);
        let node_y = |glb: &[u8]| {
            let g = gltf::Gltf::from_slice(glb).unwrap();
            let n = g.document.nodes().next().unwrap();
            n.transform().decomposed().0[1]
        };
        let height = |glb: &[u8]| {
            let p = &primitives(glb)[0].0;
            p.iter()
                .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v[1]), h.max(v[1])))
        };
        let editor = to_glb(&bytes, &VoxSettings::default()).unwrap();
        assert_eq!(
            height(&editor),
            (-1.0, 2.0),
            "one metre a voxel, the pivot floored"
        );
        assert_eq!(node_y(&editor), 10.0);
        let tenth = VoxSettings {
            scale: 0.1,
            origin: VoxOrigin::Center,
            ..VoxSettings::default()
        };
        let small = to_glb(&bytes, &tenth).unwrap();
        let (lo, hi) = height(&small);
        assert!(
            (lo + 0.1).abs() < 1e-6 && (hi - 0.2).abs() < 1e-6,
            "{lo} {hi}"
        );
        assert!((node_y(&small) - 1.0).abs() < 1e-6);
        let bottom = VoxSettings {
            scale: 0.1,
            origin: VoxOrigin::Bottom,
            ..VoxSettings::default()
        };
        let grounded = to_glb(&bytes, &bottom).unwrap();
        let (lo, hi) = height(&grounded);
        let y = node_y(&grounded);
        assert!(
            (y + lo).abs() < 1e-6,
            "the lowest voxel on y = 0: {}",
            y + lo
        );
        assert!((y + hi - 0.3).abs() < 1e-6, "0.3 m tall: {}", y + hi);
        // Two models: the lowest of the two decides, both lift together.
        let two = write_placed(
            [1, 1, 3],
            &column,
            &[([0, 0, 10], None), ([5, 0, -4], None)],
        );
        let g = to_glb(
            &two,
            &VoxSettings {
                scale: 1.0,
                origin: VoxOrigin::Bottom,
                ..VoxSettings::default()
            },
        )
        .unwrap();
        let doc = gltf::Gltf::from_slice(&g).unwrap();
        let ys: Vec<f32> = doc
            .document
            .nodes()
            .map(|n| n.transform().decomposed().0[1])
            .collect();
        assert_eq!(ys, vec![15.0, 1.0], "-4 - 1 is the floor: both up by 5");
    }

    #[test]
    fn vox_settings_are_read_and_bad_ones_refused() {
        let block = |text: &str| {
            og_core::ogt::Document::parse(&format!(
                "type import\nversion 1\n---\nmodel {{\n{text}\n}}\n"
            ))
            .unwrap()
            .body
            .get("model")
            .unwrap()
            .block
            .clone()
            .unwrap()
        };
        assert_eq!(VoxSettings::parse(None).unwrap(), VoxSettings::default());
        let b = block("voxScale 0.0625\nvoxOrigin bottom");
        assert_eq!(
            VoxSettings::parse(Some(&b)).unwrap(),
            VoxSettings {
                scale: 0.0625,
                origin: VoxOrigin::Bottom,
                ..VoxSettings::default()
            }
        );
        for (text, says) in [
            ("voxScale 0", "above 0"),
            ("voxScale -1", "above 0"),
            ("voxOrigin top", "center"),
            ("voxPivot 1", "unknown model setting 'voxPivot'"),
            ("voxFps 0", "1 to 120"),
            ("voxClip walk 3 1", "first and last frame"),
            ("voxClip walk 0", "first and last frame"),
            ("voxClip walk 0 1.5", "first and last frame"),
            (
                "voxClip walk 0 1\nvoxClip walk 2 3",
                "line 6: a second voxClip 'walk'",
            ),
        ] {
            let e = VoxSettings::parse(Some(&block(text))).unwrap_err();
            assert!(e.contains(says) && e.contains("line "), "{text}: {e}");
            if !says.starts_with("line") {
                assert!(e.contains("line 5"), "{text}: {e}");
            }
        }
    }

    #[test]
    fn frame_rate_and_clips_are_read() {
        let block = og_core::ogt::Document::parse(
            "type import\nversion 1\n---\nmodel {\nvoxFps 4\nvoxClip idle 0 1\nvoxClip \"jump high\" 2 5\n}\n",
        )
        .unwrap()
        .body
        .get("model")
        .unwrap()
        .block
        .clone()
        .unwrap();
        let v = VoxSettings::parse(Some(&block)).unwrap();
        assert_eq!(v.fps, 4.0);
        let clips: Vec<(&str, u32, u32)> = v
            .clips
            .iter()
            .map(|c| (c.name.as_str(), c.first, c.last))
            .collect();
        assert_eq!(clips, vec![("idle", 0, 1), ("jump high", 2, 5)]);
    }

    fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
        out.extend_from_slice(id);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(body);
    }

    fn dict_of(b: &mut Vec<u8>, pairs: &[(&str, String)]) {
        b.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
        for (k, v) in pairs {
            for s in [*k, v.as_str()] {
                b.extend_from_slice(&(s.len() as u32).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            }
        }
    }

    /// A `.vox` of several models (each `SIZE`/`XYZI`) and then `graph`'s
    /// chunks (none: no scene graph).
    fn write_models(models: &[([u32; 3], Vec<[u8; 4]>)], graph: &[u8]) -> Vec<u8> {
        let mut children = Vec::new();
        let mut pack = Vec::new();
        pack.extend_from_slice(&(models.len() as u32).to_le_bytes());
        chunk(&mut children, b"PACK", &pack);
        for (size, voxels) in models {
            let body: Vec<u8> = size.iter().flat_map(|s| s.to_le_bytes()).collect();
            chunk(&mut children, b"SIZE", &body);
            let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
            for v in voxels {
                xyzi.extend_from_slice(v);
            }
            chunk(&mut children, b"XYZI", &xyzi);
        }
        children.extend_from_slice(graph);
        let mut out = b"VOX ".to_vec();
        out.extend_from_slice(&150u32.to_le_bytes());
        out.extend_from_slice(b"MAIN");
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(children.len() as u32).to_le_bytes());
        out.extend_from_slice(&children);
        out
    }

    fn trn(out: &mut Vec<u8>, node: u32, child: u32, t: [i32; 3]) {
        let mut b = Vec::new();
        for v in [node, 0, child, u32::MAX, 0, 1] {
            if v == 0 && b.len() == 4 {
                dict_of(&mut b, &[]);
            } else {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        dict_of(&mut b, &[("_t", format!("{} {} {}", t[0], t[1], t[2]))]);
        chunk(out, b"nTRN", &b);
    }

    /// An `nTRN` of several frames: per key its `_f`, `_t` and `_r`.
    fn trn_keys(out: &mut Vec<u8>, node: u32, child: u32, keys: &[(u32, [i32; 3], Option<u8>)]) {
        trn_keys_with(out, node, &[], child, keys);
    }

    /// [`trn_keys`] with the node's attributes (`_loop`).
    fn trn_keys_with(
        out: &mut Vec<u8>,
        node: u32,
        attrs: &[(&str, String)],
        child: u32,
        keys: &[(u32, [i32; 3], Option<u8>)],
    ) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, attrs);
        for v in [child, u32::MAX, 0, keys.len() as u32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for (f, t, r) in keys {
            let mut frame = vec![
                ("_f", f.to_string()),
                ("_t", format!("{} {} {}", t[0], t[1], t[2])),
            ];
            if let Some(r) = r {
                frame.push(("_r", r.to_string()));
            }
            dict_of(&mut b, &frame);
        }
        chunk(out, b"nTRN", &b);
    }

    fn grp(out: &mut Vec<u8>, node: u32, children: &[u32]) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, &[]);
        b.extend_from_slice(&(children.len() as u32).to_le_bytes());
        for c in children {
            b.extend_from_slice(&c.to_le_bytes());
        }
        chunk(out, b"nGRP", &b);
    }

    fn shp(out: &mut Vec<u8>, node: u32, models: &[(u32, Option<u32>)]) {
        shp_with(out, node, &[], models);
    }

    /// [`shp`] with the node's attributes (`_loop`).
    fn shp_with(
        out: &mut Vec<u8>,
        node: u32,
        attrs: &[(&str, String)],
        models: &[(u32, Option<u32>)],
    ) {
        let mut b = node.to_le_bytes().to_vec();
        dict_of(&mut b, attrs);
        b.extend_from_slice(&(models.len() as u32).to_le_bytes());
        for (m, f) in models {
            b.extend_from_slice(&m.to_le_bytes());
            match f {
                Some(f) => dict_of(&mut b, &[("_f", f.to_string())]),
                None => dict_of(&mut b, &[]),
            }
        }
        chunk(out, b"nSHP", &b);
    }

    /// A solid box of `size` in colour `c`.
    fn solid(size: [u32; 3], c: u8) -> Vec<[u8; 4]> {
        let mut v = Vec::new();
        for z in 0..size[2] as u8 {
            for y in 0..size[1] as u8 {
                for x in 0..size[0] as u8 {
                    v.push([x, y, z, c]);
                }
            }
        }
        v
    }

    /// The world-space bounds of a glTF binary's default scene (node
    /// translations and scales applied, as the model cook computes them).
    fn world_bounds(glb: &[u8]) -> (glam::Vec3, glam::Vec3) {
        let g = gltf::Gltf::from_slice(glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let (mut lo, mut hi) = (glam::Vec3::MAX, glam::Vec3::MIN);
        for n in g.document.nodes() {
            let Some(mesh) = n.mesh() else { continue };
            let m = glam::Mat4::from_cols_array_2d(&n.transform().matrix());
            for p in mesh.primitives() {
                let r = p.reader(|_| Some(&blob[..]));
                for v in r.read_positions().unwrap() {
                    let w = m.transform_point3(glam::Vec3::from(v));
                    lo = lo.min(w);
                    hi = hi.max(w);
                }
            }
        }
        (lo, hi)
    }

    /// A file without a scene graph lands exactly where the same voxels
    /// under a graph of one transform at the origin land (MagicaVoxel
    /// centres a lone model; the old reading put its corner at the
    /// origin), even and odd sizes, with either origin.
    #[test]
    fn a_file_without_a_graph_lands_where_a_trivial_graph_puts_it() {
        for size in [[4, 2, 6], [3, 2, 5], [1, 1, 1]] {
            let voxels = solid(size, 9);
            let bare = write(size, &voxels, None);
            let graphed = write_placed(size, &voxels, &[([0, 0, 0], None)]);
            for origin in [VoxOrigin::Center, VoxOrigin::Bottom] {
                let settings = VoxSettings {
                    scale: 0.5,
                    origin,
                    ..VoxSettings::default()
                };
                let a = to_glb(&bare, &settings).unwrap();
                let b = to_glb(&graphed, &settings).unwrap();
                assert_eq!(world_bounds(&a), world_bounds(&b), "{size:?} {origin:?}");
                assert_eq!(a, b, "{size:?} {origin:?}: the same glTF");
            }
            // Centred: MagicaVoxel's pivot floor(size / 2) at the origin.
            let (lo, hi) = world_bounds(&to_glb(&bare, &VoxSettings::default()).unwrap());
            let p = size.map(|s| (s / 2) as f32);
            let want_lo = glam::Vec3::new(-p[0], -p[2], -(size[1] as f32 - p[1]));
            assert_eq!(lo, want_lo, "{size:?}");
            assert_eq!(
                hi - lo,
                glam::Vec3::new(size[0] as f32, size[2] as f32, size[1] as f32)
            );
        }
    }

    /// The visibility tracks of a glTF's animation `i`
    /// (`OG_animation_visibility`): per track its node (a mesh node: what
    /// it hides is that node's primitives), its key times and its values
    /// (STEP, 0 or 1).
    fn tracks(glb: &[u8], i: usize) -> Vec<(usize, Vec<f32>, Vec<f32>)> {
        let g = gltf::Gltf::from_slice(glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json: serde_json::Value = serde_json::from_slice(&glb[20..20 + len]).unwrap();
        let anim = g.document.animations().nth(i).expect("the clip");
        let nodes: Vec<_> = g.document.nodes().collect();
        let floats = |a: gltf::Accessor| -> Vec<f32> {
            gltf::accessor::Iter::<f32>::new(a, |_| Some(&blob[..]))
                .unwrap()
                .collect()
        };
        json["animations"][i]["extensions"][VISIBILITY]["channels"]
            .as_array()
            .expect("visibility tracks")
            .iter()
            .map(|c| {
                let node = c["node"].as_u64().unwrap() as usize;
                assert!(nodes[node].mesh().is_some());
                let s = anim
                    .samplers()
                    .nth(c["sampler"].as_u64().unwrap() as usize)
                    .unwrap();
                assert_eq!(s.interpolation(), gltf::animation::Interpolation::Step);
                (node, floats(s.input()), floats(s.output()))
            })
            .collect()
    }

    /// Several models without a graph are MagicaVoxel 0.98's animation: one
    /// frame each, all at the origin, cooked as plain mesh nodes whose clip
    /// `frames` shows each frame's node on its frame and hides it elsewhere
    /// (a visibility track). No frame moves, so none has a joint: the file
    /// has no skin, and every node stays at scale 1.
    #[test]
    fn a_pack_without_a_graph_is_the_frames_of_one_clip() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 4], solid([2, 2, 4], 2)),
            ([4, 2, 2], solid([4, 2, 2], 3)),
        ];
        let bytes = write_models(&models, &[]);
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 3);
        let frames: Vec<_> = file.placements.iter().map(|p| p.frames.clone()).collect();
        assert_eq!(frames, vec![Some(0..1), Some(1..2), Some(2..3)]);
        assert!(file.placements.iter().all(|p| p.translation == [0; 3]));
        let settings = VoxSettings {
            fps: 5.0,
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        assert_eq!(g.document.skins().count(), 0, "no joint: no frame moves");
        // One node per frame, at scale 1; those after frame 0 are hidden
        // at rest (the model drawn without a skeleton shows frame 0).
        let scales: Vec<[f32; 3]> = g
            .document
            .nodes()
            .map(|n| n.transform().decomposed().2)
            .collect();
        assert_eq!(scales, vec![[1.0; 3]; 3]);
        let len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
        let json: serde_json::Value = serde_json::from_slice(&glb[20..20 + len]).unwrap();
        let hidden: Vec<bool> = json["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["extensions"]["KHR_node_visibility"]["visible"] == false)
            .collect();
        assert_eq!(hidden, [false, true, true]);
        assert!(
            g.document
                .nodes()
                .all(|n| n.mesh().is_some() && n.skin().is_none())
        );
        let anims: Vec<_> = g.document.animations().collect();
        assert_eq!(anims.len(), 1);
        assert_eq!(anims[0].name(), Some(DEFAULT_CLIP));
        assert_eq!(anims[0].channels().count(), 0, "no transform is keyed");
        let tracks = tracks(&glb, 0);
        assert_eq!(tracks.len(), 3);
        for (k, (node, times, shown)) in tracks.into_iter().enumerate() {
            assert_eq!(node, k, "on frame {k}'s mesh node");
            assert_eq!(times, vec![0.0, 0.2, 0.4, 0.6]);
            // Frame k only; the end key holds the last frame.
            let mut want = vec![0.0; 4];
            want[k] = 1.0;
            want[3] = want[2];
            assert_eq!(shown, want, "frame {k}");
        }
        // Every frame meshes into its own primitive, faces out.
        assert_eq!(primitives(&glb).len(), 3);
        assert!(faces_out(&glb));
    }

    /// An animated shape (an `nSHP` of several models with `_f` key frames)
    /// shows each model from its key to the next, beside a static part;
    /// `voxClip` cuts the frames into named clips.
    #[test]
    fn an_animated_shape_keys_its_models_and_clips_cut_its_frames() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 3], solid([2, 2, 3], 2)),
            ([6, 6, 1], solid([6, 6, 1], 3)),
        ];
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4]);
        trn(&mut graph, 2, 3, [0, 0, 4]);
        shp(&mut graph, 3, &[(1, Some(4)), (0, Some(0))]);
        trn(&mut graph, 4, 5, [0, 0, 0]);
        shp(&mut graph, 5, &[(2, None)]);
        let bytes = write_models(&models, &graph);
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 5);
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| (p.model, p.translation, p.frames.clone()))
            .collect();
        assert_eq!(
            placed,
            vec![
                (0, [0, 0, 4], Some(0..4)),
                (1, [0, 0, 4], Some(4..5)),
                (2, [0, 0, 0], None)
            ]
        );
        let settings = VoxSettings {
            clips: vec![
                VoxClip {
                    name: "idle".into(),
                    first: 0,
                    last: 3,
                },
                VoxClip {
                    name: "pop".into(),
                    first: 3,
                    last: 4,
                },
            ],
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        // Nothing moves, so nothing is skinned: the frames are swapped by
        // their visibility tracks alone, beside the static part.
        assert_eq!(g.document.skins().count(), 0);
        assert!(g.document.nodes().all(|n| n.skin().is_none()));
        let anims: Vec<_> = g.document.animations().collect();
        assert_eq!(anims[0].name(), Some("idle"));
        assert_eq!(
            shown_frames(&glb, 0),
            vec![vec![1.0; 5], vec![0.0; 5]],
            "idle: model 0 throughout"
        );
        assert_eq!(anims[1].name(), Some("pop"));
        assert_eq!(
            shown_frames(&glb, 1),
            vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 1.0]]
        );
        // A clip past the last frame, or on a static file, is refused.
        let past = VoxSettings {
            clips: vec![VoxClip {
                name: "far".into(),
                first: 2,
                last: 5,
            }],
            ..VoxSettings::default()
        };
        let e = to_glb(&bytes, &past).unwrap_err();
        assert!(e.contains("'far'") && e.contains("5 frame(s)"), "{e}");
        let still = write([1, 1, 1], &[[0, 0, 0, 1]], None);
        let e = to_glb(&still, &past).unwrap_err();
        assert!(e.contains("no animation frames"), "{e}");
        // A static file stays unskinned and without clips.
        let g = gltf::Gltf::from_slice(&to_glb(&still, &VoxSettings::default()).unwrap()).unwrap();
        assert_eq!(
            (g.document.skins().count(), g.document.animations().count()),
            (0, 0)
        );
    }

    /// A file whose transforms carry key frames (owed row 63): a cube
    /// keyed to move between frames 0 and 2, a bar keyed to turn at frame 1
    /// and move at frame 2, and a static base. Before the keys were read,
    /// each transform's first frame placed its model throughout: one
    /// static file, no clip.
    fn keyed_file() -> Vec<u8> {
        let models = vec![
            ([6, 6, 1], solid([6, 6, 1], 3)),
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([3, 1, 1], solid([3, 1, 1], 2)),
        ];
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4, 6]);
        trn(&mut graph, 2, 3, [0, 0, 0]);
        shp(&mut graph, 3, &[(0, None)]);
        // Listed out of order: the keys sort by `_f`.
        trn_keys(
            &mut graph,
            4,
            5,
            &[(2, [2, 0, 4], None), (0, [-2, 0, 2], None)],
        );
        shp(&mut graph, 5, &[(1, None)]);
        trn_keys(
            &mut graph,
            6,
            7,
            &[
                (0, [0, 3, 1], None),
                (1, [0, 3, 1], Some(17)),
                (2, [1, 3, 1], Some(17)),
            ],
        );
        shp(&mut graph, 7, &[(2, None)]);
        write_models(&models, &graph)
    }

    #[test]
    fn keyed_transforms_move_and_turn_what_they_hold() {
        let bytes = keyed_file();
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 3, "the last key, frame 2, plus one");
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| {
                (
                    p.model,
                    p.translation,
                    p.rotation,
                    p.frames.clone(),
                    p.track.clone(),
                )
            })
            .collect();
        let turned = rotation_of(17);
        assert_eq!(
            placed,
            vec![
                (0, [0, 0, 0], IDENTITY, None, vec![]),
                (
                    1,
                    [-2, 0, 2],
                    IDENTITY,
                    Some(0..3),
                    vec![[-2, 0, 2], [-2, 0, 2], [2, 0, 4]]
                ),
                (2, [0, 3, 1], IDENTITY, Some(0..1), vec![]),
                (2, [0, 3, 1], turned, Some(1..3), vec![[0, 3, 1], [1, 3, 1]]),
            ],
            "the cube moves, the bar is a mesh per turn and moves once turned"
        );
        let settings = VoxSettings {
            clips: vec![
                VoxClip {
                    name: "all".into(),
                    first: 0,
                    last: 2,
                },
                VoxClip {
                    name: "late".into(),
                    first: 1,
                    last: 2,
                },
            ],
            ..VoxSettings::default()
        };
        let glb = to_glb(&bytes, &settings).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let blob = g.blob.clone().unwrap();
        let skin = g.document.skins().next().expect("a skin");
        let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
        assert_eq!(
            joints.len(),
            2,
            "the cube and the turned bar; the bar before its turn never moves"
        );
        // Per model (0 the cube, 1 the bar, 2 the turned bar: glTF nodes
        // `vox1` to `vox3`): its visibility track (on its mesh node) and,
        // when it moves, its joint's (`vox<n>-frames`) translation channel.
        let model_of = |node: gltf::Node| -> usize {
            let name = node.name().unwrap();
            let n: usize = name
                .trim_start_matches("vox")
                .trim_end_matches("-frames")
                .parse()
                .unwrap();
            n - 1
        };
        let nodes: Vec<_> = g.document.nodes().collect();
        type Read = (usize, &'static str, Vec<Vec<f32>>);
        let read = |i: usize, anim: &gltf::Animation| -> Vec<Read> {
            let mut out: Vec<Read> = tracks(&glb, i)
                .into_iter()
                .map(|(node, _, v)| {
                    (
                        model_of(nodes[node].clone()),
                        "visible",
                        v.into_iter().map(|x| vec![x]).collect(),
                    )
                })
                .collect();
            out.extend(anim.channels().map(|ch| {
                assert_eq!(
                    ch.sampler().interpolation(),
                    gltf::animation::Interpolation::Step
                );
                let node = ch.target().node();
                assert!(joints.contains(&node.index()), "on a joint");
                let r = ch.reader(|_| Some(&blob[..]));
                match r.read_outputs().unwrap() {
                    gltf::animation::util::ReadOutputs::Translations(t) => (
                        model_of(node),
                        "translation",
                        t.map(|v| v.to_vec()).collect(),
                    ),
                    _ => panic!("translation only: no joint is scaled"),
                }
            }));
            out.sort_by_key(|r| (r.0, r.1));
            out
        };
        let anims: Vec<_> = g.document.animations().collect();
        let v = |x: f32, y: f32, z: f32| vec![x, y, z];
        // glTF's frame: MagicaVoxel (x, y, z) is (x, z, -y).
        assert_eq!(
            read(0, &anims[0]),
            vec![
                (
                    0,
                    "translation",
                    vec![
                        v(-2.0, 2.0, 0.0),
                        v(-2.0, 2.0, 0.0),
                        v(2.0, 4.0, 0.0),
                        v(2.0, 4.0, 0.0)
                    ]
                ),
                (0, "visible", vec![vec![1.0]; 4]),
                (
                    1,
                    "visible",
                    vec![vec![1.0], vec![0.0], vec![0.0], vec![0.0]]
                ),
                // Folded on frame 0, it waits where frame 1 shows it.
                (
                    2,
                    "translation",
                    vec![
                        v(0.0, 1.0, -3.0),
                        v(0.0, 1.0, -3.0),
                        v(1.0, 1.0, -3.0),
                        v(1.0, 1.0, -3.0)
                    ]
                ),
                (
                    2,
                    "visible",
                    vec![vec![0.0], vec![1.0], vec![1.0], vec![1.0]]
                ),
            ]
        );
        assert_eq!(
            read(1, &anims[1]),
            vec![
                (
                    0,
                    "translation",
                    vec![v(-2.0, 2.0, 0.0), v(2.0, 4.0, 0.0), v(2.0, 4.0, 0.0)]
                ),
                (0, "visible", vec![vec![1.0]; 3]),
                (1, "visible", vec![vec![0.0]; 3]),
                (
                    2,
                    "translation",
                    vec![v(0.0, 1.0, -3.0), v(1.0, 1.0, -3.0), v(1.0, 1.0, -3.0)]
                ),
                (2, "visible", vec![vec![1.0]; 3]),
            ]
        );
        // The rest pose is frame 0: the cube's joint and mesh node where
        // its first key puts it; the base and the unturned bar are not
        // skinned.
        assert!(nodes[0].skin().is_none() && nodes[0].mesh().is_some());
        let bar = nodes.iter().find(|n| n.name() == Some("vox2")).unwrap();
        assert!(bar.skin().is_none() && bar.mesh().is_some());
        assert_eq!(bar.transform().decomposed().2, [1.0; 3], "at scale 1");
        assert_eq!(nodes[1].transform().decomposed().0, [-2.0, 2.0, 0.0]);
        assert_eq!(
            nodes[joints[0]].transform().decomposed().0,
            [-2.0, 2.0, 0.0]
        );
        assert!(faces_out(&glb));
        // `bottom` lifts by the lowest voxel over every frame and lifts the
        // track with the nodes: the base's underside, 0 here, stays on 0.
        let grounded = to_glb(
            &bytes,
            &VoxSettings {
                origin: VoxOrigin::Bottom,
                ..VoxSettings::default()
            },
        )
        .unwrap();
        assert_eq!(world_bounds(&grounded).0.y, 0.0);
    }

    /// Keys that change nothing (the same place on every frame) leave a
    /// static file: no skin, no clip, the same glTF as one key.
    #[test]
    fn keys_that_do_not_move_leave_the_file_static() {
        let models = vec![([2, 2, 2], solid([2, 2, 2], 1))];
        let graph = |keys: &[(u32, [i32; 3], Option<u8>)]| {
            let mut g = Vec::new();
            trn(&mut g, 0, 1, [0, 0, 0]);
            grp(&mut g, 1, &[2]);
            trn_keys(&mut g, 2, 3, keys);
            shp(&mut g, 3, &[(0, None)]);
            write_models(&models, &g)
        };
        let still = graph(&[(0, [1, 2, 3], None), (4, [1, 2, 3], None)]);
        let one = graph(&[(0, [1, 2, 3], None)]);
        let file = parse(&still).unwrap();
        assert_eq!(file.frame_count, 0);
        let a = to_glb(&still, &VoxSettings::default()).unwrap();
        assert_eq!(a, to_glb(&one, &VoxSettings::default()).unwrap());
    }

    /// A `META` chunk of the scene's attributes (`_anim_range`).
    fn meta(out: &mut Vec<u8>, attrs: &[(&str, String)]) {
        let mut b = Vec::new();
        dict_of(&mut b, attrs);
        chunk(out, b"META", &b);
    }

    /// Each visibility track of a glTF's animation `i` (1 where the
    /// model shows): one list per animated model.
    fn shown_frames(glb: &[u8], i: usize) -> Vec<Vec<f32>> {
        tracks(glb, i).into_iter().map(|t| t.2).collect()
    }

    /// `_loop` on a shape and on a keyed transform: their keys repeat from
    /// the first to the last over the file's frames instead of holding the
    /// last (MagicaVoxel's and ogt_vox's `compute_looped_frame_index`),
    /// a frame before the first key counting back from the last.
    #[test]
    fn looped_nodes_repeat_their_keys() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 3], solid([2, 2, 3], 2)),
            ([1, 1, 1], solid([1, 1, 1], 3)),
            ([3, 1, 1], solid([3, 1, 1], 4)),
        ];
        let lp = [("_loop", "1".to_string())];
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4, 6]);
        // A looped shape of two models, keyed at frames 1 and 2.
        trn(&mut graph, 2, 3, [0, 0, 0]);
        shp_with(&mut graph, 3, &lp, &[(0, Some(1)), (1, Some(2))]);
        // A looped transform keyed at frames 0 and 1.
        trn_keys_with(
            &mut graph,
            4,
            &lp,
            5,
            &[(0, [5, 0, 0], None), (1, [6, 0, 0], None)],
        );
        shp(&mut graph, 5, &[(2, None)]);
        // A transform keyed to frame 5, not looped: the file's frames.
        trn_keys(
            &mut graph,
            6,
            7,
            &[(0, [9, 0, 0], None), (5, [10, 0, 0], None)],
        );
        shp(&mut graph, 7, &[(3, None)]);
        let bytes = write_models(&models, &graph);
        let file = parse(&bytes).unwrap();
        assert_eq!(file.frame_count, 6);
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| (p.model, p.frames.clone(), p.track.clone()))
            .collect();
        let x = |v: i32| [v, 0, 0];
        assert_eq!(
            placed,
            vec![
                // Frame 0 is before the first key: looped back from the
                // last, model 1; then 0, 1, 0, 1, 0.
                (1, Some(0..1), vec![]),
                (0, Some(1..2), vec![]),
                (1, Some(2..3), vec![]),
                (0, Some(3..4), vec![]),
                (1, Some(4..5), vec![]),
                (0, Some(5..6), vec![]),
                (2, Some(0..6), vec![x(5), x(6), x(5), x(6), x(5), x(6)]),
                (3, Some(0..6), vec![x(9), x(9), x(9), x(9), x(9), x(10)]),
            ],
            "the shape and the looped transform repeat; the other holds"
        );
        // The same file without `_loop` holds each node's last key.
        let mut graph = Vec::new();
        trn(&mut graph, 0, 1, [0, 0, 0]);
        grp(&mut graph, 1, &[2, 4]);
        trn(&mut graph, 2, 3, [0, 0, 0]);
        shp(&mut graph, 3, &[(0, Some(1)), (1, Some(2))]);
        trn_keys(
            &mut graph,
            4,
            5,
            &[(0, [9, 0, 0], None), (5, [10, 0, 0], None)],
        );
        shp(&mut graph, 5, &[(3, None)]);
        let file = parse(&write_models(&models, &graph)).unwrap();
        let shape: Vec<_> = file
            .placements
            .iter()
            .filter(|p| p.model < 2)
            .map(|p| (p.model, p.frames.clone()))
            .collect();
        assert_eq!(shape, vec![(0, Some(1..2)), (1, Some(2..6))]);
        // In the cooked clip the looped shape's models alternate.
        let glb = to_glb(&bytes, &VoxSettings::default()).unwrap();
        let shown = shown_frames(&glb, 0);
        // Seven frames' keys: six frames and the clip's end holding the
        // last; the shape's six runs, each its own joint.
        assert_eq!(shown[0], vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(shown[5], vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]);
    }

    /// The scene's playback range (`META`'s `_anim_range`, MagicaVoxel
    /// 0.99.7.2): the frames walked run to its end, holding the last keys
    /// past them, and the default clip plays its frames alone. A static
    /// file with a range (the editor writes one into every file) stays
    /// static.
    #[test]
    fn the_playback_range_is_the_default_clip() {
        let models = vec![
            ([2, 2, 2], solid([2, 2, 2], 1)),
            ([2, 2, 3], solid([2, 2, 3], 2)),
        ];
        let file_with = |range: Option<&str>| {
            let mut graph = Vec::new();
            if let Some(r) = range {
                meta(&mut graph, &[("_anim_range", r.to_string())]);
            }
            trn(&mut graph, 0, 1, [0, 0, 0]);
            grp(&mut graph, 1, &[2]);
            trn(&mut graph, 2, 3, [0, 0, 0]);
            shp(&mut graph, 3, &[(0, Some(0)), (1, Some(2))]);
            write_models(&models, &graph)
        };
        let ranged = file_with(Some("1 4"));
        let file = parse(&ranged).unwrap();
        assert_eq!(file.frame_count, 5, "the range's end, frame 4, plus one");
        assert_eq!(file.playback, Some((1, 4)));
        let placed: Vec<_> = file
            .placements
            .iter()
            .map(|p| (p.model, p.frames.clone()))
            .collect();
        assert_eq!(placed, vec![(0, Some(0..2)), (1, Some(2..5))]);
        // The default clip: frames 1 to 4 (four keys and the end's).
        let glb = to_glb(&ranged, &VoxSettings::default()).unwrap();
        let g = gltf::Gltf::from_slice(&glb).unwrap();
        let clip = g.document.animations().next().unwrap();
        assert_eq!(clip.name(), Some(DEFAULT_CLIP));
        assert_eq!(
            shown_frames(&glb, 0),
            vec![vec![1.0, 0.0, 0.0, 0.0, 0.0], vec![0.0, 1.0, 1.0, 1.0, 1.0]]
        );
        // `voxClip` still names any frame of the file, before the range too.
        let early = VoxSettings {
            clips: vec![VoxClip {
                name: "early".into(),
                first: 0,
                last: 1,
            }],
            ..VoxSettings::default()
        };
        assert_eq!(
            shown_frames(&to_glb(&ranged, &early).unwrap(), 0),
            vec![vec![1.0, 1.0, 1.0], vec![0.0, 0.0, 0.0]]
        );
        // A range ending before the last key cuts the frames there: frames
        // 0 and 1 show model 0 alone, so the file plays as a static one.
        let short = parse(&file_with(Some("0 1"))).unwrap();
        assert_eq!((short.frame_count, short.playback), (0, None));
        let cut = parse(&file_with(Some("0 2"))).unwrap();
        assert_eq!((cut.frame_count, cut.playback), (3, Some((0, 2))));
        // A range that ends before it starts is ignored, as ogt_vox does.
        let bad = parse(&file_with(Some("3 1"))).unwrap();
        assert_eq!((bad.frame_count, bad.playback), (3, None));
        // A static file with the editor's default range stays static,
        // byte for byte.
        let still = |range: Option<&str>| {
            let mut graph = Vec::new();
            if let Some(r) = range {
                meta(&mut graph, &[("_anim_range", r.to_string())]);
            }
            trn(&mut graph, 0, 1, [0, 0, 0]);
            grp(&mut graph, 1, &[2]);
            trn(&mut graph, 2, 3, [1, 2, 3]);
            shp(&mut graph, 3, &[(0, None)]);
            write_models(&models[..1], &graph)
        };
        let f = parse(&still(Some("0 30"))).unwrap();
        assert_eq!((f.frame_count, f.playback), (0, None));
        assert_eq!(
            to_glb(&still(Some("0 30")), &VoxSettings::default()).unwrap(),
            to_glb(&still(None), &VoxSettings::default()).unwrap()
        );
    }

    #[test]
    fn a_file_without_the_header_is_refused() {
        assert!(parse(b"not a vox").is_err());
        assert!(parse(b"VOX \x96\0\0\0MAIN\0\0\0\0\0\0\0\0").is_err());
    }
}
