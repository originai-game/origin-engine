//! MagicaVoxel `.vox` through the cook: its import settings' `model {
//! voxScale voxOrigin }` reach the mesh (a settings file sorts after the
//! `.vox` it names, so the conversion waits for it), an edit of them
//! re-cooks the model, and a bad one is refused by line.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn chunk(out: &mut Vec<u8>, id: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(id);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(body);
}

/// A `.vox` of one model of `size` (MagicaVoxel z up), no scene graph.
fn vox(size: [u32; 3], voxels: &[[u8; 4]]) -> Vec<u8> {
    vox_of(&[(size, voxels.to_vec())], &[])
}

/// A `.vox` of several models and then `graph`'s chunks (none: no graph).
fn vox_of(models: &[([u32; 3], Vec<[u8; 4]>)], graph: &[u8]) -> Vec<u8> {
    let mut children = Vec::new();
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

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("og-cook-vox-{}", std::process::id()))
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn settings(body: &str) -> Vec<u8> {
    format!("type import\nversion 1\n---\nmodel {{\n{body}\n}}\n").into_bytes()
}

fn bounds(root: &Path) -> ([f32; 3], [f32; 3]) {
    bounds_of(root, "models/crate.vox")
}

fn bounds_of(root: &Path, id: &str) -> ([f32; 3], [f32; 3]) {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let m = bundle.load_model(id).unwrap();
    (m.min.to_array(), m.max.to_array())
}

/// A scene graph of one model: root transform 0 → group 1 → transform 2
/// at `t` → shape 3 of model 0.
fn trivial_graph(t: [i32; 3]) -> Vec<u8> {
    let dict = |b: &mut Vec<u8>, pairs: &[(&str, String)]| {
        b.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
        for (k, v) in pairs {
            for s in [*k, v.as_str()] {
                b.extend_from_slice(&(s.len() as u32).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            }
        }
    };
    let trn = |node: u32, child: u32, frame: &[(&str, String)]| {
        let mut b = node.to_le_bytes().to_vec();
        dict(&mut b, &[]);
        b.extend_from_slice(&child.to_le_bytes());
        b.extend_from_slice(&u32::MAX.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        dict(&mut b, frame);
        b
    };
    let mut g = Vec::new();
    chunk(&mut g, b"nTRN", &trn(0, 1, &[]));
    let mut grp = 1u32.to_le_bytes().to_vec();
    dict(&mut grp, &[]);
    grp.extend_from_slice(&1u32.to_le_bytes());
    grp.extend_from_slice(&2u32.to_le_bytes());
    chunk(&mut g, b"nGRP", &grp);
    let at = format!("{} {} {}", t[0], t[1], t[2]);
    chunk(&mut g, b"nTRN", &trn(2, 3, &[("_t", at)]));
    let mut shp = 3u32.to_le_bytes().to_vec();
    dict(&mut shp, &[]);
    shp.extend_from_slice(&1u32.to_le_bytes());
    shp.extend_from_slice(&0u32.to_le_bytes());
    dict(&mut shp, &[]);
    chunk(&mut g, b"nSHP", &shp);
    g
}

/// A transform's key frame: its `_f` and `_t`.
type Key = (u32, [i32; 3]);

/// A scene graph of parts: root transform 0 → group 1 → per part a
/// transform with its key frames over a shape of its model.
fn keyed_graph(parts: &[(u32, &[Key])]) -> Vec<u8> {
    let dict = |b: &mut Vec<u8>, pairs: &[(&str, String)]| {
        b.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
        for (k, v) in pairs {
            for s in [*k, v.as_str()] {
                b.extend_from_slice(&(s.len() as u32).to_le_bytes());
                b.extend_from_slice(s.as_bytes());
            }
        }
    };
    let trn = |node: u32, child: u32, keys: &[Key]| {
        let mut b = node.to_le_bytes().to_vec();
        dict(&mut b, &[]);
        for v in [child, u32::MAX, 0, keys.len().max(1) as u32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        if keys.is_empty() {
            dict(&mut b, &[]);
        }
        for (f, t) in keys {
            let at = format!("{} {} {}", t[0], t[1], t[2]);
            dict(&mut b, &[("_f", f.to_string()), ("_t", at)]);
        }
        b
    };
    let mut g = Vec::new();
    chunk(&mut g, b"nTRN", &trn(0, 1, &[]));
    let mut grp = 1u32.to_le_bytes().to_vec();
    dict(&mut grp, &[]);
    grp.extend_from_slice(&(parts.len() as u32).to_le_bytes());
    for i in 0..parts.len() as u32 {
        grp.extend_from_slice(&(2 + 2 * i).to_le_bytes());
    }
    chunk(&mut g, b"nGRP", &grp);
    for (i, (model, keys)) in parts.iter().enumerate() {
        let node = 2 + 2 * i as u32;
        chunk(&mut g, b"nTRN", &trn(node, node + 1, keys));
        let mut shp = (node + 1).to_le_bytes().to_vec();
        dict(&mut shp, &[]);
        shp.extend_from_slice(&1u32.to_le_bytes());
        shp.extend_from_slice(&model.to_le_bytes());
        dict(&mut shp, &[]);
        chunk(&mut g, b"nSHP", &shp);
    }
    g
}

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

/// The same voxels with and without a scene graph cook to the same
/// bounds (FINDINGS #27: a graph-less file was placed by its corner, so a
/// lantern's wick hung off its cage): centred about MagicaVoxel's pivot,
/// and both on the ground with `voxOrigin bottom`.
#[test]
fn a_vox_with_and_without_a_scene_graph_cooks_to_the_same_bounds() {
    let size = [5, 4, 7];
    let voxels = solid(size, 3);
    let model = [(size, voxels)];
    let bare = vox_of(&model, &[]);
    let graphed = vox_of(&model, &trivial_graph([0, 0, 0]));
    let root = project(
        "graphs",
        &[
            ("models/bare.vox", &bare),
            ("models/graphed.vox", &graphed),
            ("models/bare-low.vox", &bare),
            (
                "models/bare-low.vox.ogt",
                &settings("voxScale 0.1\nvoxOrigin bottom"),
            ),
            ("models/graphed-low.vox", &graphed),
            (
                "models/graphed-low.vox.ogt",
                &settings("voxScale 0.1\nvoxOrigin bottom"),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bare = bounds_of(&root, "models/bare.vox");
    assert_eq!(bare, bounds_of(&root, "models/graphed.vox"));
    // 5 x 7 x 4 m about the pivot floor(size / 2) = (2, 2, 3).
    assert!(
        close(bare.0, [-2.0, -3.0, -2.0]) && close(bare.1, [3.0, 4.0, 2.0]),
        "{bare:?}"
    );
    let low = bounds_of(&root, "models/bare-low.vox");
    assert_eq!(low, bounds_of(&root, "models/graphed-low.vox"));
    assert!(
        close(low.0, [-0.2, 0.0, -0.2]) && close(low.1, [0.3, 0.7, 0.2]),
        "{low:?}"
    );
}

/// An animated `.vox` (three frames without a scene graph, MagicaVoxel
/// 0.98's animation) cooks to a model with a `#anim` blob: `voxFps` and
/// `voxClip` give its clips, whose visibility channels show one frame's
/// mesh node at a time. A folded frame is hidden, not a joint scaled to
/// 0: every joint rests at scale 1, no clip has a scale channel, and the
/// frames after 0 are hidden at rest. Compressed (`animation { compress
/// true }`, the loosest tolerance), the flags are the same. Before the
/// visibility track, each frame was a STEP scale of 0 or 1 on its joint,
/// and every folded frame still cost its draw.
#[test]
fn an_animated_vox_cooks_its_frames_into_clips() {
    let frames = vec![
        ([2, 2, 2], solid([2, 2, 2], 1)),
        ([2, 2, 3], solid([2, 2, 3], 2)),
        ([2, 2, 4], solid([2, 2, 4], 3)),
    ];
    let bytes = vox_of(&frames, &[]);
    let root = project(
        "frames",
        &[
            ("models/blob.vox", &bytes),
            (
                "models/blob.vox.ogt",
                &settings("voxScale 0.5\nvoxFps 4\nvoxClip idle 0 0\nvoxClip grow 0 2"),
            ),
            ("models/plain.vox", &bytes),
            ("models/packed.vox", &bytes),
            (
                "models/packed.vox.ogt",
                b"type import\nversion 1\n---\nanimation {\n  compress true\n  tolerance 0.1\n}\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let anim = bundle.load_animation("models/blob.vox").unwrap();
    let clips: Vec<(&str, f32)> = anim
        .clips
        .iter()
        .map(|c| (c.name.as_str(), c.duration))
        .collect();
    assert_eq!(clips, vec![("idle", 0.25), ("grow", 0.75)]);
    assert_eq!(anim.skeleton.skins.len(), 1);
    assert_eq!(anim.skeleton.skins[0].joints.len(), 3);
    let sk = &anim.skeleton;
    let joints = &sk.skins[0].joints;
    assert!(
        joints
            .iter()
            .all(|&j| sk.nodes[j as usize].rest.scale == glam::Vec3::ONE),
        "no joint is scaled to 0"
    );
    // Frame k's mesh node is the node written before its joint.
    let mesh_nodes: Vec<u32> = joints.iter().map(|j| j - 1).collect();
    assert!(
        mesh_nodes
            .iter()
            .all(|&n| sk.nodes[n as usize].mesh != og_anim::NONE)
    );
    assert_eq!(sk.hidden, mesh_nodes[1..], "frames 1 and 2 hidden at rest");
    assert!(
        anim.clips
            .iter()
            .flat_map(|c| &c.channels)
            .all(|ch| ch.path == og_anim::ChannelPath::Visibility),
        "visibility channels only"
    );
    let grow = anim.clip("grow").unwrap();
    assert_eq!(grow.rate, 4.0);
    assert_eq!(grow.channels.len(), 3);
    for (k, ch) in grow.channels.iter().enumerate() {
        assert!(ch.step, "a STEP channel");
        assert_eq!(ch.node, mesh_nodes[k], "on frame {k}'s mesh node");
        let mut want = vec![0.0; 4];
        want[k] = 1.0;
        want[3] = want[2];
        assert_eq!(ch.samples, want, "frame {k}");
    }
    let packed = bundle.load_animation("models/packed.vox").unwrap();
    let plain_frames = bundle.load_animation("models/plain.vox").unwrap();
    assert_eq!(packed.clips, plain_frames.clips, "compressed, still exact");
    // Every frame's primitive is skinned whole to its joint.
    assert_eq!(anim.skin_streams.len(), 3);
    for (k, s) in anim.skin_streams.iter().enumerate() {
        assert!(s.joints.iter().all(|j| j[0] as usize == k));
        assert!(s.weights.iter().all(|w| (w[0] - 1.0).abs() < 1e-4));
    }
    // Without settings: one clip `frames` of every frame at 10 per second.
    let plain = bundle.load_animation("models/plain.vox").unwrap();
    let clips: Vec<(&str, f32)> = plain
        .clips
        .iter()
        .map(|c| (c.name.as_str(), c.duration))
        .collect();
    assert_eq!(clips, vec![("frames", 0.3)]);
    // The bind pose draws frame 0 alone: the model's bounds are its 1 m x
    // 1 m x 1 m (2 x 2 x 2 at 0.5 m), the other frames folded to a point.
    let (lo, hi) = bounds_of(&root, "models/blob.vox");
    assert!(
        close(lo, [-0.5, -0.5, -0.5]) && close(hi, [0.5, 0.5, 0.5]),
        "{lo:?} {hi:?}"
    );
}

/// A `.vox` whose transform is keyed (MagicaVoxel's `nTRN` frames, owed
/// row 63): a block keyed to move 4 voxels along x and 2 up at frame 2
/// over a static base cooks to a model with a `#anim` blob whose clip
/// moves the block's joint (a STEP translation channel, at `voxFps`) and
/// keeps it shown; the base stays unskinned. Before the keys were read
/// the first key placed the block throughout: a static model, no `#anim`.
#[test]
fn a_keyed_transform_cooks_to_a_moving_joint() {
    let models = [
        ([6, 6, 1], solid([6, 6, 1], 3)),
        ([2, 2, 2], solid([2, 2, 2], 1)),
    ];
    let bytes = vox_of(
        &models,
        &keyed_graph(&[
            (0, &[(0, [0, 0, 0])]),
            (1, &[(0, [-2, 0, 2]), (2, [2, 0, 4])]),
        ]),
    );
    let root = project(
        "keyed",
        &[
            ("models/lift.vox", &bytes),
            ("models/lift.vox.ogt", &settings("voxScale 0.5\nvoxFps 4")),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let anim = bundle
        .load_animation("models/lift.vox")
        .expect("a keyed file is animated");
    assert_eq!(anim.skeleton.skins.len(), 1);
    assert_eq!(anim.skeleton.skins[0].joints.len(), 1, "the block alone");
    let clip = anim.clip("frames").unwrap();
    assert_eq!((clip.rate, clip.duration), (4.0, 0.75));
    let paths: Vec<og_anim::ChannelPath> = clip.channels.iter().map(|c| c.path).collect();
    assert_eq!(
        paths,
        vec![
            og_anim::ChannelPath::Translation,
            og_anim::ChannelPath::Visibility
        ]
    );
    assert!(clip.channels.iter().all(|c| c.step), "STEP channels");
    assert_eq!(
        clip.channels[1].samples,
        vec![1.0; 4],
        "shown on every frame"
    );
    // Frames 0, 1, 2 and the end key: (x, z, -y) of the keys at 0.5 m.
    let places: Vec<[f32; 3]> = clip.channels[0]
        .samples
        .chunks(3)
        .map(|s| [s[0], s[1], s[2]])
        .collect();
    let want = [
        [-1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, 2.0, 0.0],
        [1.0, 2.0, 0.0],
    ];
    assert!(
        places.iter().zip(want).all(|(a, b)| close(*a, b)),
        "{places:?}"
    );
    // The translation drives the block's joint node, the visibility its
    // mesh node.
    let joint = anim.skeleton.skins[0].joints[0];
    let binding = og_anim::bind(&anim.skeleton, clip);
    assert_eq!(binding, vec![joint, joint - 1]);
    // The bind pose is frame 0: the block at its first key beside the base.
    let (lo, hi) = bounds_of(&root, "models/lift.vox");
    assert!(
        close(lo, [-1.5, 0.0, -1.5]) && close(hi, [1.5, 1.5, 1.5]),
        "{lo:?} {hi:?}"
    );
}

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
}

#[test]
fn import_settings_scale_and_ground_a_vox_and_an_edit_recooks_it() {
    // A 4 x 2 x 6 block (6 voxels tall in MagicaVoxel z, y in glTF).
    let mut voxels = Vec::new();
    for z in 0..6u8 {
        for y in 0..2u8 {
            for x in 0..4u8 {
                voxels.push([x, y, z, 9]);
            }
        }
    }
    let block = vox([4, 2, 6], &voxels);
    let root = project(
        "crate",
        &[
            ("models/crate.vox", &block),
            (
                "models/crate.vox.ogt",
                &settings("voxScale 0.25\nvoxOrigin bottom"),
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let (lo, hi) = bounds(&root);
    // 1 m x 1.5 m x 0.5 m about its pivot (a file without a scene graph
    // is centred, as MagicaVoxel shows it), on the ground.
    assert!(
        close(lo, [-0.5, 0.0, -0.25]) && close(hi, [0.5, 1.5, 0.25]),
        "{lo:?} {hi:?}"
    );

    // Unchanged: nothing cooks. The settings edited: the model again.
    let again = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(!again.cooked_outputs().iter().any(|o| o.cooked));
    std::fs::write(
        root.join("assets/models/crate.vox.ogt"),
        settings("voxScale 1"),
    )
    .unwrap();
    let third = cook_project(&root, &CookOptions::default()).unwrap();
    let cooked: Vec<&str> = third
        .cooked_outputs()
        .iter()
        .filter(|o| o.cooked)
        .map(|o| o.id.as_str())
        .collect();
    assert_eq!(cooked, vec!["models/crate.vox"]);
    let (lo, hi) = bounds(&root);
    assert!(
        close(lo, [-2.0, -3.0, -1.0]) && close(hi, [2.0, 3.0, 1.0]),
        "one metre a voxel, centred: {lo:?} {hi:?}"
    );

    // A bad setting fails the model, naming the line.
    std::fs::write(
        root.join("assets/models/crate.vox.ogt"),
        settings("voxOrigin top"),
    )
    .unwrap();
    let bad = cook_project(&root, &CookOptions::default()).unwrap();
    let e = bad.errors().join("\n");
    assert!(
        e.contains("models/crate.vox") && e.contains("line 5") && e.contains("voxOrigin"),
        "{e}"
    );
}
