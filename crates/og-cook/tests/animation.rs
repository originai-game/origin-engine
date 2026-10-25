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

mod support;

fn project(name: &str, settings: Option<&str>) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-anim")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("assets/models")).unwrap();
    for m in ["walker.glb", "trickster.glb"] {
        std::fs::copy(
            support::template_model(m),
            dir.join("assets/models").join(m),
        )
        .unwrap();
    }
    if let Some(s) = settings {
        std::fs::write(dir.join("assets/models/walker.glb.ogt"), s).unwrap();
    }
    dir
}

fn cooked(name: &str, settings: Option<&str>) -> (PathBuf, Bundle) {
    let root = project(name, settings);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    (root, b)
}

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

/// The glTF sampler, written from the specification (§3.11 and Appendix C),
/// in f64. A key within a microsecond of `t` counts as reached.
fn reference(
    interp: gltf::animation::Interpolation,
    times: &[f32],
    values: &[f32],
    lanes: usize,
    rotation: bool,
    t: f64,
) -> Vec<f64> {
    use gltf::animation::Interpolation::*;
    let per = if interp == CubicSpline { 3 } else { 1 };
    let v = |k: usize| -> Vec<f64> {
        let base = (k * per + if per == 3 { 1 } else { 0 }) * lanes;
        values[base..base + lanes]
            .iter()
            .map(|&x| x as f64)
            .collect()
    };
    let norm = |mut q: Vec<f64>| {
        if rotation {
            let l = q.iter().map(|x| x * x).sum::<f64>().sqrt();
            q.iter_mut().for_each(|x| *x /= l);
        }
        q
    };
    let n = times.len();
    let k = match times.iter().rposition(|&x| x as f64 <= t + 1e-6) {
        None => return norm(v(0)),
        Some(k) if k == n - 1 => return norm(v(n - 1)),
        Some(k) => k,
    };
    let (t0, t1) = (times[k] as f64, times[k + 1] as f64);
    let s = (t - t0) / (t1 - t0);
    let (a, b) = (v(k), v(k + 1));
    norm(match interp {
        Step => a,
        Linear if rotation => {
            let mut dot: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
            let b: Vec<f64> = if dot < 0.0 {
                dot = -dot;
                b.iter().map(|x| -x).collect()
            } else {
                b
            };
            if dot > 1.0 - 1e-9 {
                a.iter().zip(&b).map(|(x, y)| x + (y - x) * s).collect()
            } else {
                let th = dot.acos();
                a.iter()
                    .zip(&b)
                    .map(|(x, y)| (((1.0 - s) * th).sin() * x + (s * th).sin() * y) / th.sin())
                    .collect()
            }
        }
        Linear => a.iter().zip(&b).map(|(x, y)| x + (y - x) * s).collect(),
        CubicSpline => {
            let dt = t1 - t0;
            let out_k = &values[(k * 3 + 2) * lanes..(k * 3 + 3) * lanes];
            let in_k1 = &values[((k + 1) * 3) * lanes..((k + 1) * 3 + 1) * lanes];
            let (s2, s3) = (s * s, s * s * s);
            (0..lanes)
                .map(|i| {
                    (2.0 * s3 - 3.0 * s2 + 1.0) * a[i]
                        + (s3 - 2.0 * s2 + s) * dt * out_k[i] as f64
                        + (-2.0 * s3 + 3.0 * s2) * b[i]
                        + (s3 - s2) * dt * in_k1[i] as f64
                })
                .collect()
        }
    })
}

fn anim(b: &Bundle, model: &str) -> CookedAnimation {
    b.load_animation(model).unwrap()
}

/// Compares every sample of every clip of `models` in `b` with the reference.
fn check(b: &Bundle, models: &[&str], tag: &str, report: &mut Vec<String>) {
    for &model in models {
        let a = anim(b, &format!("models/{model}"));
        let (doc, buffers) = source(model);
        assert_eq!(a.clips.len(), doc.animations().count());
        for ga in doc.animations() {
            let clip = a.clip(ga.name().unwrap()).expect("clip by name");
            let mut worst = 0.0f64;
            let mut checked = 0usize;
            for (ci, ch) in ga.channels().enumerate() {
                let r = ch.reader(|bf| Some(&buffers[bf.index()].0[..]));
                let times: Vec<f32> = r.read_inputs().unwrap().collect();
                use gltf::animation::util::ReadOutputs::*;
                let (lanes, rotation, values): (usize, bool, Vec<f32>) =
                    match r.read_outputs().unwrap() {
                        Translations(v) => (3, false, v.flatten().collect()),
                        Scales(v) => (3, false, v.flatten().collect()),
                        Rotations(v) => (4, true, v.into_f32().flatten().collect()),
                        MorphTargetWeights(v) => {
                            let w: Vec<f32> = v.into_f32().collect();
                            let per = if ch.sampler().interpolation()
                                == gltf::animation::Interpolation::CubicSpline
                            {
                                3
                            } else {
                                1
                            };
                            (w.len() / times.len() / per, false, w)
                        }
                    };
                let cc = &clip.channels[ci];
                // Bound to the right node: by name.
                assert_eq!(cc.node_name, ch.target().node().name().unwrap_or(""));
                assert_eq!(a.skeleton.nodes[cc.node as usize].name, cc.node_name);
                for i in 0..clip.frames {
                    let t32 = (i as f32 / clip.rate).min(clip.duration);
                    let t = (i as f64 / clip.rate as f64).min(clip.duration as f64);
                    let want = reference(
                        ch.sampler().interpolation(),
                        &times,
                        &values,
                        lanes,
                        rotation,
                        t,
                    );
                    let p = clip.frame_pos(t32);
                    let got: Vec<f64> = match cc.path {
                        og_anim::ChannelPath::Rotation => og_anim::ClipAsset::quat(cc, p)
                            .to_array()
                            .iter()
                            .map(|&x| x as f64)
                            .collect(),
                        og_anim::ChannelPath::Weights => (0..lanes)
                            .map(|k| og_anim::ClipAsset::weight(cc, p, k) as f64)
                            .collect(),
                        _ => og_anim::ClipAsset::vec3(cc, p)
                            .to_array()
                            .iter()
                            .map(|&x| x as f64)
                            .collect(),
                    };
                    let sign = if rotation
                        && got.iter().zip(&want).map(|(a, b)| a * b).sum::<f64>() < 0.0
                    {
                        -1.0
                    } else {
                        1.0
                    };
                    for (g, w) in got.iter().zip(&want) {
                        let e = (g * sign - w).abs();
                        assert!(
                            e < 1e-5,
                            "{model} '{}' channel {ci} frame {i} (t {t}): {got:?} vs {want:?}",
                            clip.name
                        );
                        worst = worst.max(e);
                    }
                    checked += 1;
                }
            }
            report.push(format!(
                "{{\"cook\":\"{tag}\",\"model\":\"{model}\",\"clip\":\"{}\",\"duration\":{},\"source_rate\":{},\"rate\":{},\"frames\":{},\"channels\":{},\"samples_checked\":{checked},\"max_error\":{worst:e}}}",
                clip.name, clip.duration, clip.source_rate, clip.rate, clip.frames, clip.channels.len()
            ));
        }
    }
}

#[test]
fn cooked_clips_resample_to_the_gltf_sampler_within_1e_5() {
    let b = cooked_default();
    let mut report = Vec::new();
    check(
        &b,
        &["walker.glb", "trickster.glb"],
        "source rate",
        &mut report,
    );
    // At 60 Hz most samples fall between keys: slerp, Hermite and STEP
    // holds are evaluated, not copied.
    let (_, b60) = cooked(
        "resample-60",
        Some("type import\nversion 1\n---\nanimation {\n  rate 60\n}\n"),
    );
    check(&b60, &["walker.glb"], "rate 60", &mut report);
    let walker = anim(&b, "models/walker.glb");
    let rates: Vec<(String, f32)> = walker
        .clips
        .iter()
        .map(|c| (c.name.clone(), c.rate))
        .collect();
    assert_eq!(
        rates,
        [
            ("idle".into(), 30.0),
            ("walk".into(), 24.0),
            ("run".into(), 10.0),
            ("wave".into(), 4.0),
            ("turn".into(), 1.0),
            ("blink".into(), 30.0)
        ]
    );
    assert!((walker.clip("walk").unwrap().source_rate - 24.0).abs() < 1e-3);
    assert!(walker.clip("wave").unwrap().channels[0].step);
    let dir = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("anim-cook-resample.json"),
        format!(
            "{{\n  \"tolerance\": 1e-5,\n  \"clips\": [\n    {}\n  ]\n}}\n",
            report.join(",\n    ")
        ),
    )
    .unwrap();
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
fn markers_come_from_extras_and_import_settings() {
    let (_, b) = cooked(
        "settings",
        Some(
            "type import\nversion 1\n---\nanimation {\n  rate 60\n  event \"idle\" \"breath\" 1\n}\n",
        ),
    );
    let a = anim(&b, "models/walker.glb");
    let names = |c: &str| -> Vec<(String, f32)> {
        a.clip(c)
            .unwrap()
            .events
            .iter()
            .map(|e| (e.name.clone(), e.time))
            .collect()
    };
    assert_eq!(
        names("walk"),
        [("footL".to_string(), 0.26), ("footR".to_string(), 0.76)]
    );
    assert_eq!(names("idle"), [("breath".to_string(), 1.0)]);
    assert!(a.clips.iter().all(|c| c.rate == 60.0));
    assert_eq!(a.clip("walk").unwrap().frames, 61);

    let root = project(
        "bad-settings",
        Some("type import\nversion 1\n---\nanimation {\n  event \"jump\" \"land\" 0.5\n}\n"),
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(
        errors.contains("event 'land' names clip 'jump'"),
        "{errors}"
    );
}

#[test]
fn a_cooked_walker_plays_in_og_anim() {
    use og_anim::*;
    let b = cooked_default();
    let a = anim(&b, "models/walker.glb");
    let mut world = og_ecs::World::new(&og_ecs::WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let sys = install(&mut world).unwrap();
    sys.add_skeleton(1, a.skeleton.clone()).unwrap();
    for (i, c) in a.clips.iter().enumerate() {
        sys.add_clip(10 + i as u64, c.clone()).unwrap();
    }
    let walk = 10 + a.clips.iter().position(|c| c.name == "walk").unwrap() as u64;
    let e = world
        .spawn((
            Transform::default(),
            Skeleton {
                asset: 1,
                ..Skeleton::default()
            },
            AnimationPlayer::new(walk),
        ))
        .unwrap();
    let mut fired = Vec::new();
    for _ in 0..60 {
        let tick = world.time().tick;
        world.step().unwrap();
        for ev in world.pending_events::<AnimationEvent>().unwrap() {
            fired.push((tick, ev.name.as_str().to_string()));
        }
    }
    assert_eq!(
        fired,
        [(15, "footL".to_string()), (45, "footR".to_string())]
    );
    let out = sys.output(e).unwrap();
    assert_eq!(out.palette.len(), 12);
    // The thigh swings: its palette entry is not the identity.
    assert!(
        out.palette[3]
            .to_cols_array()
            .iter()
            .zip(og_core::glam::Mat4::IDENTITY.to_cols_array())
            .any(|(x, y)| (x - y).abs() > 1e-3)
    );
}

#[test]
fn a_clip_is_mirrored_at_cook_time() {
    let (_, b) = cooked(
        "mirror",
        Some(
            "type import\nversion 1\n---\nanimation {\n  mirror \"walk\" \"walkMirrored\"\n  mirror \"idle\"\n}\n",
        ),
    );
    let a = anim(&b, "models/walker.glb");
    let walk = a.clip("walk").unwrap();
    let mut want = og_anim::mirror(&a.skeleton, walk).unwrap();
    want.name = "walkMirrored".into();
    // The cooked clip is the one og.anim.mirror makes at run time.
    assert_eq!(a.clip("walkMirrored").unwrap(), &want);
    assert_ne!(want.channels, walk.channels);
    assert!(a.clip("idle.mirrored").is_some());
    assert!(a.clip("walk").is_some() && a.clip("idle").is_some());

    for (settings, says) in [
        ("  mirror \"jump\"\n", "mirror 'jump'"),
        ("  mirror \"walk\" \"idle\"\n", "already called 'idle'"),
        (
            "  mirror \"walk\" \"a\"\n  mirror \"idle\" \"a\"\n",
            "line 6",
        ),
    ] {
        let root = project(
            "bad-mirror",
            Some(&format!(
                "type import\nversion 1\n---\nanimation {{\n{settings}}}\n"
            )),
        );
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        let errors = report.errors().join("\n");
        assert!(errors.contains(says), "{settings}: {errors}");
    }
}

/// A glTF (text, one data-URI buffer) whose only animation hides and shows
/// node `flag` through `KHR_animation_pointer` onto
/// `KHR_node_visibility/visible` (an unsigned-byte STEP sampler, keys at 0,
/// 0.5 and 1 s: shown, hidden, shown), as Blender and three.js write it,
/// with a second pointer channel onto `extra` when given.
fn pointer_gltf(extra: Option<&str>, required: bool) -> String {
    // Times f32 ×3 (12 bytes), then the flags u8 ×3 and a byte of padding.
    let mut buf: Vec<u8> = [0.0f32, 0.5, 1.0]
        .iter()
        .flat_map(|t| t.to_le_bytes())
        .collect();
    buf.extend_from_slice(&[1, 0, 1, 0]);
    let b64 = {
        const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::new();
        for c in buf.chunks(3) {
            let n = (c[0] as u32) << 16
                | (*c.get(1).unwrap_or(&0) as u32) << 8
                | *c.get(2).unwrap_or(&0) as u32;
            for k in 0..4 {
                s.push(if k <= c.len() {
                    A[(n >> (18 - 6 * k) & 63) as usize] as char
                } else {
                    '='
                });
            }
        }
        s
    };
    let pointer = |p: &str| serde_json::json!({ "sampler": 0, "target": { "path": "pointer", "extensions": { "KHR_animation_pointer": { "pointer": p } } } });
    let mut channels = vec![pointer("/nodes/1/extensions/KHR_node_visibility/visible")];
    if let Some(e) = extra {
        channels.push(pointer(e));
    }
    let mut json = serde_json::json!({
        "asset": { "version": "2.0" },
        "extensionsUsed": ["KHR_animation_pointer", "KHR_node_visibility"],
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [
            { "name": "root", "children": [1] },
            { "name": "flag", "translation": [0, 1, 0], "extensions": { "KHR_node_visibility": { "visible": true } } }
        ],
        "buffers": [{ "byteLength": 16, "uri": format!("data:application/octet-stream;base64,{b64}") }],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": 12 },
            { "buffer": 0, "byteOffset": 12, "byteLength": 3 }
        ],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 3, "type": "SCALAR", "min": [0.0], "max": [1.0] },
            { "bufferView": 1, "componentType": 5121, "count": 3, "type": "SCALAR" }
        ],
        "animations": [{
            "name": "blink",
            "samplers": [{ "input": 0, "output": 1, "interpolation": "STEP" }],
            "channels": channels
        }]
    });
    if required {
        json["extensionsRequired"] = serde_json::json!(["KHR_animation_pointer"]);
    }
    json.to_string()
}

fn cook_pointer(name: &str, gltf: &str) -> (og_cook::cook::Report, PathBuf) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-anim-pointer")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/models")).unwrap();
    std::fs::write(root.join("assets/models/blink.gltf"), gltf).unwrap();
    (cook_project(&root, &CookOptions::default()).unwrap(), root)
}

#[test]
fn khr_animation_pointer_onto_node_visibility_cooks_to_a_visibility_track() {
    // Required, and every pointer channel the engine's: the file loads (the
    // gltf crate alone refuses the `pointer` path) and the track arrives.
    let (report, root) = cook_pointer("visibility", &pointer_gltf(None, true));
    assert!(report.ok(), "{:?}", report.errors());
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let a = anim(&b, "models/blink.gltf");
    let clip = a.clip("blink").expect("the clip");
    let flag = a.skeleton.node("flag").unwrap();
    let ch = clip
        .channels
        .iter()
        .find(|c| c.path == og_anim::ChannelPath::Visibility)
        .expect("a visibility channel");
    assert_eq!(ch.node, flag);
    let at = |t: f32| og_anim::ClipAsset::visible(ch, clip.frame_pos(t));
    assert_eq!(
        [at(0.0), at(0.25), at(0.5), at(0.75), at(1.0)],
        [true, true, false, false, true],
        "shown, hidden from 0.5 s, shown again at 1 s"
    );
    // A required pointer onto something the engine does not animate is
    // refused, naming it; the same file not requiring the extension cooks
    // with that channel dropped and said so.
    let colour = "/materials/0/pbrMetallicRoughness/baseColorFactor";
    let (report, _) = cook_pointer("refused", &pointer_gltf(Some(colour), true));
    let errors = report.errors().join("\n");
    assert!(!report.ok() && errors.contains(colour), "{errors}");
    let (report, root) = cook_pointer("dropped", &pointer_gltf(Some(colour), false));
    assert!(report.ok(), "{:?}", report.errors());
    assert!(format!("{report:?}").contains("dropped"), "{report:?}");
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let clip = anim(&b, "models/blink.gltf").clip("blink").unwrap().clone();
    assert_eq!(clip.channels.len(), 1, "the visibility track alone");
}
