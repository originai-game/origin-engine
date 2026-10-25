//! Animation sample compression (`animation { compress true tolerance t }`,
//! `og_cook::anim_codec`) and sparse morph targets, on every committed
//! animated model (fx-showcase's walker, trickster, strider with its
//! retargeted library clips, stout, puppet2d and the library itself; the
//! game template, third-person and og-tests carry copies of the same
//! files).
//!
//! Each model is cooked twice, f32 (the default) and compressed at the
//! default tolerance. Asserted: every frame of every channel is within the
//! tolerance of the f32 cook; every node's model-space position within
//! 0.5 mm per metre of the skeleton's extent; the compressed samples are
//! smaller; and the trickster posed mid-clip by either cook, two metres
//! tall and filling the frame, renders within og-tests' default frame
//! tolerance of the other (RMSE 0.01, 0.5 % outliers; measured 0.0028 and
//! 0.008 %: silhouette edges moved by a fraction of a pixel). Evidence in `$OG_EVIDENCE_DIR/P4/cook/`:
//! `anim-compress.json` (sizes, errors, the frame comparison),
//! `anim-compress-f32.png`, `anim-compress-quantized.png` and the diff.

use og_anim::{ClipAsset, Pose, bind, sample};
use og_cook::cook::{CookOptions, cook_project};
use og_cook::runtime::{Bundle, CookedAnimation};
use og_render::{RenderScene, SkinData, SkinnedInstance};
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn evidence_dir() -> PathBuf {
    let base = std::env::var_os("OG_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("evidence"));
    let dir = base.join("P4/cook");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The animated models and what they read, as fx-showcase ships them.
const FILES: &[&str] = &[
    "models/walker.glb",
    "models/trickster.glb",
    "models/strider.glb",
    "models/strider.glb.ogt",
    "models/stout.glb",
    "models/puppet2d.glb",
    "anims/humanoid-clips.glb",
    "rigs/strider.ogt",
];

const MODELS: &[&str] = &[
    "models/walker.glb",
    "models/trickster.glb",
    "models/strider.glb",
    "models/stout.glb",
    "models/puppet2d.glb",
    "anims/humanoid-clips.glb",
];

/// Copies the models into a fresh project; with `compress`, every glTF's
/// import settings gain `compress true` (the strider's keep their rig and
/// library).
fn stage(name: &str, compress: bool) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-anim-compress")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    let assets = root.join("assets");
    let src = repo().join("templates/fx-showcase/assets");
    for f in FILES {
        let to = assets.join(f);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(src.join(f), &to).unwrap();
    }
    if compress {
        for m in MODELS {
            let path = assets.join(format!("{m}.ogt"));
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t.replacen("animation {\n", "animation {\n  compress true\n", 1),
                Err(_) => "type import\nversion 1\n---\nanimation {\n  compress true\n}\n".into(),
            };
            std::fs::write(&path, text).unwrap();
        }
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    root
}

fn anim_bytes(root: &Path, id: &str) -> usize {
    std::fs::metadata(root.join("cooked").join(format!("{id}.anim.ogb")))
        .unwrap()
        .len() as usize
}

/// Bytes of the blob's sample sections (`samples` and `packed`).
fn sample_bytes(root: &Path, id: &str) -> usize {
    let bytes = std::fs::read(root.join("cooked").join(format!("{id}.anim.ogb"))).unwrap();
    let blob = og_blob::Blob::parse(&bytes, id).unwrap();
    ["samples", "packed"]
        .iter()
        .filter_map(|s| blob.section(s))
        .map(|s| s.size as usize)
        .sum()
}

/// The largest model-space node distance between the clip `a` and `b` at
/// every frame and halfway between frames.
fn model_space_error(anim: &CookedAnimation, a: &ClipAsset, b: &ClipAsset) -> f32 {
    let sk = &anim.skeleton;
    let (ba, bb) = (bind(sk, a), bind(sk, b));
    let mut worst = 0.0f32;
    for i in 0..a.frames * 2 - 1 {
        let t = (i as f32 * 0.5 / a.rate).min(a.duration);
        let (mut pa, mut pb) = (Pose::rest(sk), Pose::rest(sk));
        sample(sk, a, &ba, t, og_anim::NONE, &mut pa);
        sample(sk, b, &bb, t, og_anim::NONE, &mut pb);
        let (ma, mb) = (sk.model_matrices(&pa.nodes), sk.model_matrices(&pb.nodes));
        for (x, y) in ma.iter().zip(&mb) {
            worst = worst.max(x.w_axis.truncate().distance(y.w_axis.truncate()));
        }
    }
    worst
}

/// The skeleton's extent at rest (the largest distance between two nodes'
/// model-space positions along an axis).
fn extent(anim: &CookedAnimation) -> f32 {
    let m = anim.skeleton.model_matrices(&anim.skeleton.rest_pose());
    let (lo, hi) = m.iter().map(|m| m.w_axis.truncate()).fold(
        (
            og_core::glam::Vec3::splat(f32::MAX),
            og_core::glam::Vec3::splat(f32::MIN),
        ),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    );
    (hi - lo).max_element().max(1e-6)
}

/// The trickster posed by `clip` at `t`, skinned, in og-tests'
/// gltf-reference camera and light.
fn posed_trickster(root: &Path, clip: &str, t: f32) -> og_render::RgbaImage {
    let b = Bundle::open(&root.join("cooked")).unwrap();
    let (model, textures) = b.load_model_with_textures("models/trickster.glb").unwrap();
    let anim = b.load_animation("models/trickster.glb").unwrap();
    let clip = anim.clip(clip).unwrap();
    let sk = &anim.skeleton;
    let mut pose = Pose::rest(sk);
    sample(sk, clip, &bind(sk, clip), t, og_anim::NONE, &mut pose);
    let mut gfx = og_tests::scenes::build("gltf-reference").unwrap();
    let src = &gfx.scene;
    // A fresh scene: the model's meshes are its primitives from handle 0.
    let mut scene = RenderScene::new();
    let parts = model.add_to(&mut scene, &textures);
    let mut palette = Vec::new();
    sk.palette(0, &pose.nodes, &mut palette);
    scene
        .joint_palettes
        .push(og_render::JointPalette { matrices: palette });
    // Two metres tall on the origin, filling the frame.
    use og_core::glam::{Mat4, Vec3};
    let scale = 2.0 / (model.max - model.min).max_element().max(1e-3);
    let place = Mat4::from_translation(Vec3::Y * (-model.min.y * scale))
        * Mat4::from_scale(Vec3::splat(scale));
    for &(mesh, material, local) in &parts.parts {
        let p = mesh.index();
        match anim.skin_streams.iter().find(|s| s.primitive == p) {
            Some(s) => {
                let skinned = scene.add_skinned_mesh(
                    model.primitives[p].mesh(0),
                    SkinData {
                        joints: s.joints.clone(),
                        weights: s.weights.clone(),
                    },
                );
                scene
                    .skinned_instances
                    .push(SkinnedInstance::new(skinned, material, place, 0));
            }
            None => scene.add_instance(mesh, material, place * local),
        }
    }
    scene.camera = og_render::Camera {
        position: Vec3::new(0.4, 1.3, 2.8),
        target: Vec3::new(0.0, 1.0, 0.0),
        ..src.camera
    };
    scene.light = src.light;
    scene.lights = src.lights.clone();
    scene.environment = src.environment;
    scene.post = src.post;
    gfx.scene = scene;
    og_tests::gfx::render_built(&mut gfx).unwrap().image
}

#[test]
fn compressed_clips_stay_within_tolerance_and_shrink() {
    let f32_root = stage("f32", false);
    let q_root = stage("quantized", true);
    let tolerance = og_cook::cook::anim::DEFAULT_TOLERANCE;
    let (fb, qb) = (
        Bundle::open(&f32_root.join("cooked")).unwrap(),
        Bundle::open(&q_root.join("cooked")).unwrap(),
    );
    let mut rows = Vec::new();
    let (mut total_f32, mut total_q) = (0usize, 0usize);
    for id in MODELS {
        let (fa, qa) = (
            fb.load_animation(id).unwrap(),
            qb.load_animation(id).unwrap(),
        );
        assert_eq!(fa.skeleton, qa.skeleton, "{id}: skeletons");
        assert_eq!(fa.skin_streams, qa.skin_streams, "{id}: skin streams");
        assert_eq!(fa.morphs, qa.morphs, "{id}: morph targets");
        assert_eq!(fa.clips.len(), qa.clips.len());
        let (mut lane_err, mut model_err) = (0.0f32, 0.0f32);
        for (a, b) in fa.clips.iter().zip(&qa.clips) {
            assert_eq!(
                (&a.name, a.frames, &a.events),
                (&b.name, b.frames, &b.events)
            );
            for (ca, cb) in a.channels.iter().zip(&b.channels) {
                assert_eq!((ca.node, ca.path, ca.lanes), (cb.node, cb.path, cb.lanes));
                let lanes = ca.lanes as usize;
                let (sa, sb) = (ca.decoded(a.frames), cb.decoded(b.frames));
                for (x, y) in sa.chunks(lanes).zip(sb.chunks(lanes)) {
                    // A rotation and its negation are one rotation.
                    let flip = ca.path == og_anim::ChannelPath::Rotation
                        && x.iter().zip(y).map(|(p, q)| p * q).sum::<f32>() < 0.0;
                    for (p, q) in x.iter().zip(y) {
                        let d = (p - if flip { -q } else { *q }).abs();
                        lane_err = lane_err.max(d);
                    }
                }
            }
            model_err = model_err.max(model_space_error(&fa, a, b));
        }
        assert!(
            lane_err <= tolerance * 1.0001,
            "{id}: a lane is {lane_err} off (tolerance {tolerance})"
        );
        let ext = extent(&fa);
        assert!(
            model_err <= 5e-4 * ext.max(1.0),
            "{id}: a node moved {model_err} (extent {ext})"
        );
        let (sf, sq) = (sample_bytes(&f32_root, id), sample_bytes(&q_root, id));
        let (bf, bq) = (anim_bytes(&f32_root, id), anim_bytes(&q_root, id));
        if sf > 0 {
            assert!(sq < sf, "{id}: compressed samples {sq} B >= f32 {sf} B");
        }
        total_f32 += bf;
        total_q += bq;
        rows.push(format!(
            "    {{\"id\": \"{id}\", \"clips\": {}, \"blob_f32\": {bf}, \"blob_quantized\": {bq}, \"samples_f32\": {sf}, \"samples_quantized\": {sq}, \"max_lane_error\": {lane_err:.3e}, \"max_node_error_m\": {model_err:.3e}, \"extent\": {ext:.4}}}",
            fa.clips.len()
        ));
    }

    // A frame: the trickster mid-clip from each cook.
    let clip = "die";
    let (img_f, img_q) = (
        posed_trickster(&f32_root, clip, 0.5),
        posed_trickster(&q_root, clip, 0.5),
    );
    let cmp = og_tests::compare::compare(
        &img_q.pixels,
        &img_f.pixels,
        img_f.width,
        img_f.height,
        &og_tests::compare::Tolerance::default(),
    )
    .unwrap();
    let ev = evidence_dir();
    og_tests::gfx::save_png(&ev.join("anim-compress-f32.png"), &img_f).unwrap();
    og_tests::gfx::save_png(&ev.join("anim-compress-quantized.png"), &img_q).unwrap();
    og_tests::gfx::save_png(
        &ev.join("anim-compress.diff.png"),
        &og_render::RgbaImage {
            width: img_f.width,
            height: img_f.height,
            pixels: cmp.diff_image.clone(),
        },
    )
    .unwrap();
    std::fs::write(
        ev.join("anim-compress.json"),
        format!(
            "{{\n  \"tolerance\": {tolerance},\n  \"models\": [\n{}\n  ],\n  \"total_f32\": {total_f32},\n  \"total_quantized\": {total_q},\n  \"frame\": {{\"model\": \"models/trickster.glb\", \"clip\": \"{clip}\", \"time\": 0.5, \"rmse\": {:.6}, \"max_channel_diff\": {}, \"outliers\": {}, \"pixels\": {}}}\n}}\n",
            rows.join(",\n"),
            cmp.rmse,
            cmp.max_channel_diff,
            cmp.outliers,
            cmp.pixels
        ),
    )
    .unwrap();
    // og-tests' default frame tolerance (RMSE 0.01, 0.5 % of pixels past
    // 32 levels): the edges of a 2 m figure move by a fraction of a pixel.
    cmp.check(&og_tests::compare::Tolerance::default())
        .unwrap_or_else(|e| panic!("the posed frames differ: {e}"));
    assert!(total_q < total_f32);
}

/// Frame `j` of every lane of a quantised channel, written out from the
/// format's description (`og_cook::anim_codec`): the key on a kept frame,
/// else a hold (STEP), a lerp per lane or a normalised lerp on the shorter
/// arc between the kept frames around it. Independent of og-anim's
/// sampler, which reads the keys where they lie.
fn written_out(ch: &og_anim::Channel, frames: u32) -> Vec<f32> {
    let og_anim::Samples::Quantized(q) = &ch.samples else {
        return ch.decoded(frames);
    };
    let rotation = ch.path == og_anim::ChannelPath::Rotation;
    let lanes = ch.lanes as usize;
    let per = if rotation { 3 } else { lanes };
    let key = |k: usize| -> Vec<f32> {
        let w = &q.words[k * per..(k + 1) * per];
        if rotation {
            og_anim::unpack_quat([w[0], w[1], w[2]]).to_vec()
        } else {
            (0..lanes)
                .map(|l| og_anim::dequantize(q.range[2 * l], q.range[2 * l + 1], w[l]))
                .collect()
        }
    };
    let kept: Vec<usize> = if q.frames.is_empty() {
        (0..frames as usize).collect()
    } else {
        q.frames.iter().map(|&f| f as usize).collect()
    };
    let mut out = Vec::new();
    for j in 0..frames as usize {
        let k = kept.iter().rposition(|&f| f <= j).unwrap();
        let (fa, a) = (kept[k], key(k));
        if fa == j || ch.step {
            out.extend(a);
            continue;
        }
        let (fb, b) = (kept[k + 1], key(k + 1));
        let u = (j - fa) as f32 / (fb - fa) as f32;
        if rotation {
            let a4 = [a[0], a[1], a[2], a[3]];
            out.extend(og_anim::nlerp4(a4, [b[0], b[1], b[2], b[3]], u));
        } else {
            out.extend(a.iter().zip(&b).map(|(&x, &y)| x + (y - x) * u));
        }
    }
    out
}

/// A world playing every clip of `anim` (one entity each, at speed 1, at
/// 0.37 for times between frames, and backwards), with `clips` registered.
fn playing(anim: &CookedAnimation, clips: &[ClipAsset]) -> og_ecs::World {
    let mut w = og_ecs::World::new(&og_ecs::WorldDesc {
        tick_rate: 60,
        seed: 1,
    })
    .unwrap();
    let sys = og_anim::install(&mut w).unwrap();
    sys.add_skeleton(1, anim.skeleton.clone()).unwrap();
    for (i, c) in clips.iter().enumerate() {
        let handle = 100 + i as u64;
        sys.add_clip(handle, c.clone()).unwrap();
        for speed in [1.0, 0.37, -1.0] {
            w.spawn((
                og_anim::Transform::default(),
                og_anim::Skeleton {
                    asset: 1,
                    ..og_anim::Skeleton::default()
                },
                og_anim::AnimationPlayer {
                    speed,
                    ..og_anim::AnimationPlayer::new(handle)
                },
            ))
            .unwrap();
        }
    }
    w
}

#[test]
fn resident_quantised_clips_cost_their_cooked_bytes_and_pose_as_their_frames() {
    let root = stage("resident", true);
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let mut quantised_channels = 0;
    for id in MODELS {
        let anim = bundle.load_animation(id).unwrap();
        // Resident: the clips' frames take what the blob's sample sections hold.
        let resident: usize = anim.clips.iter().map(|c| c.sample_bytes()).sum();
        assert_eq!(
            resident,
            sample_bytes(&root, id),
            "{id}: resident sample bytes"
        );
        // The same clips with every frame written out as f32 (what the
        // loader built before og-anim sampled the keys).
        let written: Vec<ClipAsset> = anim
            .clips
            .iter()
            .map(|c| ClipAsset {
                channels: c
                    .channels
                    .iter()
                    .map(|ch| {
                        if let og_anim::Samples::Quantized(q) = &ch.samples
                            && !q.frames.is_empty()
                        {
                            quantised_channels += 1;
                        }
                        og_anim::Channel {
                            samples: og_anim::Samples::F32(written_out(ch, c.frames)),
                            ..ch.clone()
                        }
                    })
                    .collect(),
                ..c.clone()
            })
            .collect();
        let f32_bytes: usize = written.iter().map(|c| c.sample_bytes()).sum();
        // (stout carries a skeleton and no clips.)
        assert!(
            resident < f32_bytes || resident == 0,
            "{id}: {resident} B resident against {f32_bytes} B as f32"
        );
        // Every clip's frames read the same bits, frame by frame.
        for (a, b) in anim.clips.iter().zip(&written) {
            for (ca, cb) in a.channels.iter().zip(&b.channels) {
                let (da, db) = (ca.decoded(a.frames), cb.decoded(b.frames));
                assert!(
                    da.iter().zip(&db).all(|(x, y)| x.to_bits() == y.to_bits()),
                    "{id} '{}': node {} {:?} frames differ",
                    a.name,
                    ca.node,
                    ca.path
                );
            }
        }
        // And og.anim poses them to the same world, tick by tick.
        let mut qa = playing(&anim, &anim.clips);
        let mut fa = playing(&anim, &written);
        for tick in 0..150 {
            qa.step().unwrap();
            fa.step().unwrap();
            assert_eq!(qa.world_hash(), fa.world_hash(), "{id}: tick {tick}");
        }
    }
    assert!(quantised_channels > 0, "some channels keep reduced keys");
}

#[test]
fn compress_and_tolerance_are_import_settings() {
    use og_cook::cook::anim::AnimSettings;
    let doc = |body: &str| {
        og_core::ogt::Document::parse(&format!(
            "type import\nversion 1\n---\nanimation {{\n{body}\n}}\n"
        ))
        .unwrap()
    };
    let parse = |body: &str| {
        let d = doc(body);
        AnimSettings::parse(d.body.get("animation").and_then(|e| e.block.as_ref()))
    };
    let a = parse("  compress true\n  tolerance 0.001").unwrap();
    assert!(a.compress);
    assert_eq!(a.tolerance, 0.001);
    let d = parse("  rate 30").unwrap();
    assert!(!d.compress, "f32 is the default");
    assert_eq!(d.tolerance, og_cook::cook::anim::DEFAULT_TOLERANCE);
    assert!(parse("  tolerance 0").unwrap_err().contains("line 5"));
    assert!(
        parse("  compress maybe")
            .unwrap_err()
            .contains("true or false")
    );
    assert_ne!(
        a.describe(),
        d.describe(),
        "the setting is part of the stamp"
    );
}
