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
/// `voxClip` give its clips, whose STEP scale channels show one frame's
/// joint at a time.
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
    let grow = anim.clip("grow").unwrap();
    assert_eq!(grow.rate, 4.0);
    assert_eq!(grow.channels.len(), 3);
    for (k, ch) in grow.channels.iter().enumerate() {
        assert!(ch.step, "a STEP channel");
        assert_eq!(ch.path, og_anim::ChannelPath::Scale);
        let shown: Vec<f32> = ch.samples.chunks(3).map(|s| s[0]).collect();
        let mut want = vec![0.0; 4];
        want[k] = 1.0;
        want[3] = want[2];
        assert_eq!(shown, want, "frame {k}'s joint");
    }
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
