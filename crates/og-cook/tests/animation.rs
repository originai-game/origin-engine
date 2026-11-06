//! glTF skins, clips and morph targets through the cook into the `#anim`
//! blob and back out of the runtime loader as og-anim assets, on the
//! template's two rigged models: `walker.glb` (self-made: LINEAR, STEP,
//! CUBICSPLINE and weights channels, markers, morph targets) and
//! `trickster.glb` (a Blender export: LINEAR and STEP).
//!
//! Failure modes covered: resampling that disagrees with the glTF sampler
//! (lerp instead of slerp, a wrong Hermite basis or tangent scale, STEP one
//! key late, the last frame missing, times off by a sample), channels bound
//! to the wrong node after the topological reorder, inverse bind matrices
//! transposed, joints and weights or morph deltas attached to the wrong
//! cooked vertex after welding and reordering, weights not summing to 1,
//! markers lost or out of their clip, import settings ignored.
//!
//! Evidence: `$OG_EVIDENCE_DIR/anim-cook-resample.json` (default
//! `<target tmp>/evidence/`): per clip, rates, frames and the largest error.

use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::{Bundle, CookedAnimation};
use std::path::PathBuf;

use crate::support;

/// The two models cooked with no import settings, once for every test
/// process that only reads them (`support::shared`).
fn cooked_default() -> Bundle {
    let sources = [
        support::template_model("walker.glb"),
        support::template_model("trickster.glb"),
    ];
    let cook = support::shared("anim-default", &sources, |root| {
        std::fs::create_dir_all(root.join("assets/models")).unwrap();
        for m in ["walker.glb", "trickster.glb"] {
            std::fs::copy(
                support::template_model(m),
                root.join("assets/models").join(m),
            )
            .unwrap();
        }
    });
    Bundle::open(&cook.root.join("cooked")).unwrap()
}

fn source(name: &str) -> (gltf::Document, Vec<gltf::buffer::Data>) {
    let (doc, buffers, _) = gltf::import(support::template_model(name)).unwrap();
    (doc, buffers)
}

fn anim(b: &Bundle, model: &str) -> CookedAnimation {
    b.load_animation(model).unwrap()
}

/// The source vertices bucketed by position, so the vertex a cooked vertex
/// came from is found among its neighbours instead of among all of them.
/// The match minimises `|source - p| - 1e-3 * (source normal . n)`, so it
/// lies within `nearest + 2e-3` of `p`: [`SourceGrid::near`] returns every
/// vertex in the 27 cells around `p` (cells of `CELL`), all vertices within
/// `CELL` of it, when the nearest is within `CELL - 2.5e-3` (a margin over
/// 2e-3 for rounding), and otherwise every
/// vertex. Candidates come in index order, so ties resolve as a full scan
/// would.
struct SourceGrid {
    cells: std::collections::HashMap<[i32; 3], Vec<usize>>,
    positions: Vec<og_core::glam::Vec3>,
}

const CELL: f32 = 4e-3;

impl SourceGrid {
    fn new(pos: &[[f32; 3]]) -> Self {
        let positions: Vec<og_core::glam::Vec3> =
            pos.iter().map(|&p| og_core::glam::Vec3::from(p)).collect();
        let mut cells = std::collections::HashMap::<[i32; 3], Vec<usize>>::new();
        for (i, p) in positions.iter().enumerate() {
            cells.entry(Self::cell(*p)).or_default().push(i);
        }
        SourceGrid { cells, positions }
    }

    fn cell(p: og_core::glam::Vec3) -> [i32; 3] {
        let c = (p / CELL).floor();
        [c.x as i32, c.y as i32, c.z as i32]
    }

    fn near(&self, p: og_core::glam::Vec3) -> Vec<usize> {
        let [x, y, z] = Self::cell(p);
        let mut out = Vec::new();
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(v) = self.cells.get(&[x + dx, y + dy, z + dz]) {
                        out.extend_from_slice(v);
                    }
                }
            }
        }
        let nearest = out
            .iter()
            .map(|&i| (self.positions[i] - p).length())
            .fold(f32::INFINITY, f32::min);
        if nearest > CELL - 2.5e-3 {
            return (0..self.positions.len()).collect();
        }
        out.sort_unstable();
        out
    }
}

#[test]
fn skins_and_morphs_follow_the_cooked_vertices() {
    let b = cooked_default();
    for model in ["walker.glb", "trickster.glb"] {
        let id = format!("models/{model}");
        let a = anim(&b, &id);
        let m = b.load_model(&id).unwrap();
        let (doc, buffers) = source(model);
        // The skeleton is the model's node graph, the skin the glTF's.
        assert_eq!(a.skeleton.nodes.len(), m.nodes.len());
        let gskin = doc.skins().next().unwrap();
        let skin = &a.skeleton.skins[0];
        assert_eq!(skin.joints.len(), gskin.joints().count());
        let ibms: Vec<[[f32; 4]; 4]> = gskin
            .reader(|bf| Some(&buffers[bf.index()].0[..]))
            .read_inverse_bind_matrices()
            .unwrap()
            .collect();
        for (j, (node, ibm)) in gskin.joints().zip(&ibms).enumerate() {
            assert_eq!(
                a.skeleton.nodes[skin.joints[j] as usize].name,
                node.name().unwrap_or("")
            );
            assert_eq!(skin.inverse_bind[j].to_cols_array_2d(), *ibm, "joint {j}");
        }
        // The bind pose's palette is the identity (to the source's precision).
        let mut palette = Vec::new();
        a.skeleton.palette(0, &a.skeleton.rest_pose(), &mut palette);
        for (j, p) in palette.iter().enumerate() {
            let err = p
                .to_cols_array()
                .iter()
                .zip(og_core::glam::Mat4::IDENTITY.to_cols_array())
                .map(|(x, y)| (x - y).abs())
                .fold(0.0, f32::max);
            assert!(err < 1e-4, "{model} joint {j}: bind palette off by {err}");
        }
        // Per cooked vertex: the joints, weights and morph deltas of the
        // source vertex at that position and normal.
        let prim = doc.meshes().next().unwrap().primitives().next().unwrap();
        let r = prim.reader(|bf| Some(&buffers[bf.index()].0[..]));
        let pos: Vec<[f32; 3]> = r.read_positions().unwrap().collect();
        let nrm: Vec<[f32; 3]> = r.read_normals().unwrap().collect();
        let joints: Vec<[u16; 4]> = r.read_joints(0).unwrap().into_u16().collect();
        let weights: Vec<[f32; 4]> = r.read_weights(0).unwrap().into_f32().collect();
        let morphs: Vec<Vec<[f32; 3]>> = r
            .read_morph_targets()
            .map(|(p, _, _)| p.unwrap().collect())
            .collect();
        let stream = &a.skin_streams[0];
        let cooked = &m.primitives[stream.primitive];
        assert_eq!(stream.joints.len(), cooked.vertices.len());
        let extent = (cooked.max - cooked.min).max_element();
        let grid = SourceGrid::new(&pos);
        for (v, vert) in cooked.vertices.iter().enumerate() {
            let p = og_core::glam::Vec3::from(vert.position);
            let n = og_core::glam::Vec3::from(vert.normal);
            let d = |i: usize| {
                (og_core::glam::Vec3::from(pos[i]) - p).length()
                    - 1e-3 * og_core::glam::Vec3::from(nrm[i]).dot(n)
            };
            let s = grid
                .near(p)
                .into_iter()
                .min_by(|&x, &y| d(x).total_cmp(&d(y)))
                .unwrap();
            assert!((og_core::glam::Vec3::from(pos[s]) - p).length() < extent * 1e-4);
            let sum: f32 = stream.weights[v].iter().sum();
            assert!((sum - 1.0).abs() < 1e-6, "weights sum {sum}");
            for k in 0..4 {
                if weights[s][k] > 0.0 {
                    assert_eq!(stream.joints[v][k], joints[s][k], "{model} vertex {v}");
                }
                let total: f32 = weights[s].iter().sum();
                assert!((stream.weights[v][k] - weights[s][k] / total).abs() < 2.0 / 65535.0);
            }
            for (t, target) in morphs.iter().enumerate() {
                let got = a.morphs[0].positions[t][v];
                assert!((got - og_core::glam::Vec3::from(target[s])).length() < 1e-6);
            }
        }
        if model == "walker.glb" {
            let mt = &a.morphs[0];
            assert_eq!(mt.names, ["blink", "squash"]);
            assert_eq!(mt.default_weights, [0.0, 0.0]);
            assert!(mt.positions[0].iter().any(|d| d.length() > 0.01));
        } else {
            assert!(a.morphs.is_empty());
        }
    }
}

#[test]
fn skin_diagnostics_warn_on_stretch_not_foreign_weights_and_survive_cached_cooks() {
    use serde_json::json;
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("skin-diagnostics");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    // Four three-joint limbs; arms leave different spine joints. Joint order
    // below is reversed in the skin to exercise JOINTS_0 rather than node IDs.
    let mut nodes = json!([
        {"name":"Hips", "children":[1,9,12]},
        {"name":"Spine", "children":[2,3]},
        {"name":"Chest", "children":[6]},
        {"name":"LeftArm", "translation":[0,1,0], "children":[4]},
        {"name":"LeftForeArm", "children":[5]}, {"name":"LeftHand"},
        {"name":"RightArm", "translation":[0,1,0], "children":[7]},
        {"name":"RightForeArm", "children":[8]}, {"name":"RightHand"},
        {"name":"LeftUpLeg", "children":[10]},
        {"name":"LeftLeg", "children":[11]}, {"name":"LeftFoot"},
        {"name":"RightUpLeg", "children":[13]},
        {"name":"RightLeg", "children":[14]}, {"name":"RightFoot"},
        {"mesh":0,"skin":0}
    ]);
    let mut bin = Vec::<u8>::new();
    // Triangle 0 has uniform cross-limb weights: harmless. Triangle 1 tears.
    // Triangle 2 grows by >2.5x but stays below 8cm; triangle 3 grows beyond
    // 8cm but not by 2.5x. Only source triangle 1 should be warned about.
    let positions: [[f32; 3]; 12] = [
        [0., 0., 0.],
        [0.01, 0., 0.],
        [0., 0.01, 0.],
        [0., 0., 0.],
        [0.01, 0., 0.],
        [0., 0.01, 0.],
        [0., 0.9, 0.],
        [0.001, 0.9, 0.],
        [0., 0.901, 0.],
        [0., 0., 0.],
        [0.8, 0., 0.],
        [0., 0.8, 0.],
    ];
    for p in positions {
        for f in p {
            bin.extend(f.to_le_bytes());
        }
    }
    for _ in &positions {
        for j in [11u16, 8, 0, 0] {
            bin.extend(j.to_le_bytes());
        }
    }
    for i in 0..12 {
        let weights: [f32; 4] = if i >= 3 && i % 3 == 0 {
            [1., 0., 0., 0.]
        } else {
            [0.8, 0.2, 0., 0.]
        };
        for w in weights {
            bin.extend(w.to_le_bytes());
        }
    }
    for j in (0..15).rev() {
        let m = glam::Mat4::from_translation(if (3..9).contains(&j) {
            glam::Vec3::NEG_Y
        } else {
            glam::Vec3::ZERO
        });
        for f in m.to_cols_array() {
            bin.extend(f.to_le_bytes());
        }
    }
    std::fs::write(root.join("assets/skin.bin"), &bin).unwrap();
    let mut doc = json!({
        "asset":{"version":"2.0"}, "scene":0, "scenes":[{"nodes":[0,15]}],
        "nodes":nodes, "skins":[{"joints":(0..15).rev().collect::<Vec<_>>(),"inverseBindMatrices":3}],
        "buffers":[{"uri":"skin.bin","byteLength":bin.len()}],
        "bufferViews":[
            {"buffer":0,"byteOffset":0,"byteLength":144},
            {"buffer":0,"byteOffset":144,"byteLength":96},
            {"buffer":0,"byteOffset":240,"byteLength":192},
            {"buffer":0,"byteOffset":432,"byteLength":960}
        ],
        "accessors":[
            {"bufferView":0,"componentType":5126,"count":12,"type":"VEC3","min":[0,0,0],"max":[0.8,0.901,0]},
            {"bufferView":1,"componentType":5123,"count":12,"type":"VEC4"},
            {"bufferView":2,"componentType":5126,"count":12,"type":"VEC4"},
            {"bufferView":3,"componentType":5126,"count":15,"type":"MAT4"}
        ],
        "meshes":[{"primitives":[{"attributes":{"POSITION":0,"JOINTS_0":1,"WEIGHTS_0":2}}]}]
    });
    std::fs::write(root.join("assets/skin.gltf"), doc.to_string()).unwrap();
    let first = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(first.ok(), "{:?}", first.errors());
    let warnings = first.warnings.join("\n");
    assert!(
        warnings.contains("rightArmForward: 1 stretched triangles"),
        "{warnings}"
    );
    assert!(
        first
            .warnings
            .iter()
            .all(|w| w.contains("source triangle IDs [1]")),
        "{warnings}"
    );
    assert!(
        !warnings.contains("cross-limb"),
        "counts alone must not warn"
    );
    let notes = first
        .assets
        .iter()
        .flat_map(|a| &a.notes)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        notes.contains("9 cross-limb vertices (information only)"),
        "{notes}"
    );
    let second = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(second.cooked_outputs().is_empty());
    assert_eq!(first.warnings, second.warnings);
    let stats =
        og_cook::cook::stats::cook_report_json(&second, &root.join("cooked"), &Default::default());
    assert!(stats["assets"].as_array().unwrap().iter().any(|a| {
        a["skinDiagnostics"].is_array()
            && a["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w["kind"] == "skinStretch")
    }));
    // The same unnamed rig takes the topology fallback, including the
    // continuing spine child at its fork, with no humanoid-name dependency.
    for n in nodes.as_array_mut().unwrap() {
        n.as_object_mut().unwrap().remove("name");
    }
    doc["nodes"] = nodes;
    std::fs::write(root.join("assets/skin.gltf"), doc.to_string()).unwrap();
    let fallback = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(fallback.ok(), "{:?}", fallback.errors());
    assert!(
        fallback
            .warnings
            .iter()
            .any(|w| w.contains("joint11Forward") && w.contains("source triangle IDs [1]")),
        "{:?}",
        fallback.warnings
    );
    assert!(
        fallback
            .warnings
            .iter()
            .any(|w| w.contains("joint12Forward"))
    );
    // A custom retarget map is authoritative too. Default node transforms
    // deliberately disagree with bind matrices; probes must use the latter.
    for (i, n) in doc["nodes"].as_array_mut().unwrap().iter_mut().enumerate() {
        n["name"] = json!(format!("n{i}"));
    }
    doc["nodes"][3]["translation"] = json!([0, 3, 0]);
    std::fs::write(root.join("assets/skin.gltf"), doc.to_string()).unwrap();
    std::fs::write(
        root.join("assets/skin.gltf.ogt"),
        "type import\nversion 1\n---\nanimation {\n  rig \"custom.ogt\"\n}\n",
    )
    .unwrap();
    std::fs::write(root.join("assets/custom.ogt"), "type rig\nversion 1\n---\nleftUpperArm \"n3\"\nleftLowerArm \"n4\"\nrightUpperArm \"n6\"\nrightLowerArm \"n7\"\nleftUpperLeg \"n9\"\nleftLowerLeg \"n10\"\nrightUpperLeg \"n12\"\n").unwrap();
    let custom = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(custom.ok(), "{:?}", custom.errors());
    assert_eq!(custom.warnings, first.warnings);
    // Limbs sharing an attach joint are exempt from the informational count.
    doc["nodes"][1]["children"] = json!([2, 3, 6]);
    doc["nodes"][2].as_object_mut().unwrap().remove("children");
    std::fs::write(root.join("assets/skin.gltf"), doc.to_string()).unwrap();
    let shared = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(shared.ok(), "{:?}", shared.errors());
    assert!(
        shared
            .assets
            .iter()
            .flat_map(|a| &a.notes)
            .any(|n| n.contains("0 cross-limb vertices"))
    );
}
