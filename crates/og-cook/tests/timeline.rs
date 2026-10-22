//! Timelines through the cook (depth S2): the caption file a track names is
//! imported into caption keys and is part of the timeline's stamp; the
//! sounds a timeline names must be in the bundle; a scene's named paths
//! reach the scene the runtime loads.
//!
//! Ways this could fail: the cooked text still names the `.srt` (the
//! runtime cannot read it) or drops its cues; an edit to the `.srt` alone
//! leaves the stale timeline in `cooked/` (the stamp misses it); the `.srt`
//! is reported as an ignored file; a timeline naming a sound that was never
//! cooked cooks anyway and fails only when it plays; a scene's `path` is
//! refused, or lost between the cook and `Bundle::scene`.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_timeline::{PathKind, Track};
use std::path::{Path, PathBuf};

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-timeline")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        write(&root, path, text);
    }
    root
}

fn write(root: &Path, path: &str, text: &str) {
    let p = root.join("assets").join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

const CLICK: &str = "type sfx\nversion 1\n---\npreset \"click\"\nseed 0\n";

const INTRO: &str = r#"type timeline
version 1
---
name "Intro"
duration 6
track caption {
  captions "lines.srt"
  style xl
}
track audio {
  key 1 {
    sound "audio/click.ogt"
    volume 0.5
  }
}
"#;

const SRT: &str =
    "1\n00:00:00,500 --> 00:00:02,000\nThe yard.\n\n2\n00:00:02,500 --> 00:00:04,000\nThe guard.\n";

fn captions(root: &Path) -> Vec<(f32, f32, String)> {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "timelines/intro.ogt";
    let t = bundle.timeline(id, &bundle.read(id).unwrap()).unwrap();
    assert_eq!(t.duration, 6.0);
    let c = t
        .tracks
        .iter()
        .find_map(|t| match t {
            Track::Caption(c) => Some(c.clone()),
            _ => None,
        })
        .expect("a caption track");
    assert_eq!(c.style, "xl");
    c.keys
        .into_iter()
        .map(|k| (k.time, k.end, k.text))
        .collect()
}

#[test]
fn an_srt_is_imported_into_caption_keys_and_its_edit_recooks_the_timeline() {
    let root = project(
        "srt",
        &[
            ("timelines/intro.ogt", INTRO),
            ("timelines/lines.srt", SRT),
            ("audio/click.ogt", CLICK),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let ignored = report.to_json();
    assert!(
        !ignored.contains("lines.srt"),
        "the caption file belongs to the timeline, not the ignored list: {ignored}"
    );
    assert_eq!(
        captions(&root),
        vec![
            (0.5, 2.0, "The yard.".to_string()),
            (2.5, 4.0, "The guard.".to_string())
        ]
    );
    // Only the .srt changes: the timeline is cooked again.
    write(
        &root,
        "timelines/lines.srt",
        &SRT.replace("The guard.", "The wall."),
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    assert_eq!(captions(&root)[1].2, "The wall.");
}

#[test]
fn a_sound_the_bundle_lacks_is_refused_by_name() {
    let root = project(
        "missing",
        &[
            (
                "timelines/intro.ogt",
                &INTRO.replace("audio/click.ogt", "audio/boom.wav"),
            ),
            ("timelines/lines.srt", SRT),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].starts_with("assets/timelines/intro.ogt")
            && errors[0].contains("sound 'audio/boom.wav'"),
        "{errors:?}"
    );
}

#[test]
fn a_scenes_named_paths_reach_the_loaded_scene() {
    let scene = r#"type scene
version 1
---
name "Yard"
path "patrol" {
  kind linear
  closed true
  point 0 0 0
  point 4 0 0
  point 4 0 4
}
entity "post" {
  Transform {
    position 1 0 1
  }
}
"#;
    let bad = "type scene\nversion 1\n---\npath \"p\" {\n  point 0 0 0\n}\n";
    let root = project(
        "scene",
        &[("scenes/yard.ogt", scene), ("scenes/bad.ogt", bad)],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains("path 'p'") && errors[0].contains("at least 2 points"),
        "{errors:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "scenes/yard.ogt";
    let s = bundle.scene(id, &bundle.read(id).unwrap()).unwrap();
    assert_eq!(s.paths.len(), 1);
    let (name, p) = &s.paths[0];
    assert_eq!(name, "patrol");
    assert!(p.closed && p.kind == PathKind::Linear && p.points.len() == 3);
    assert_eq!(s.entities.len(), 1);
}
