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

/// The runtime test's timeline (`og-runtime/tests/timeline.rs`): a rail
/// camera, a transform track, events and markers, written before R10.
const BEFORE_R10: &str = r#"type timeline
version 1
---
name "T"
duration 6
path "arc" {
  kind catmullRom
  point -4 3 8
  point 0 2 7
  point 4 3 8
}
track camera {
  rail "arc"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 6 {
    at 1
    target 2 1 0
    fov 40
    ease cubicInOut
  }
}
track transform "box" {
  key 0 {
    position 0 0 0
  }
  key 3 {
    position 3 0 0
    ease quadIn
  }
  key 6 {
    position 3 2 0
    rotation 0 90 0
  }
}
track event {
  key 1.5 {
    do "mark:early"
  }
  key 3.5 {
    do "mark:late"
    retroactive true
  }
}
track marker {
  key 1 {
    name "one"
  }
  key 5 {
    name "five"
  }
}
"#;

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

/// R10 added track kinds and camera keys without bumping the blob: a
/// timeline written before them cooks to the same blob (its length, its
/// canonical text, pinned on the code before R10 with `og cook`, and the
/// type hash and version), so nothing re-cooks.
#[test]
fn a_timeline_from_before_r10_cooks_to_the_bytes_it_had() {
    use og_cook::format::BlobKind;
    let root = project("before-r10", &[("timelines/t.ogt", BEFORE_R10)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let blob = std::fs::read(root.join("cooked/timelines/t.ogt.ogb")).unwrap();
    let at = blob
        .windows(13)
        .position(|w| w == b"type timeline")
        .expect("the canonical text");
    let text = {
        let mut t = &blob[at..];
        while let [rest @ .., 0] = t {
            t = rest;
        }
        t
    };
    assert_eq!(
        (
            blob.len(),
            text.len(),
            format!("{:016x}", fnv(text)),
            format!("{:016x}", og_cook::format::Timeline::type_hash()),
            og_cook::format::Timeline::schema_version()
        ),
        (
            861,
            669,
            "5cafa6278ac2bd29".into(),
            "2028e07f19e5abae".into(),
            1
        )
    );
}

/// The new tracks' fields are checked against the engine's records at
/// cook, naming the field: a field the record lacks (with its fields), one
/// a timeline cannot drive, a value of the wrong shape, a resource given an
/// entity. Timelines that nest in a cycle, or deeper than four, are refused
/// naming the chain.
///
/// Ways this could fail: a typo cooks and silently does nothing when it
/// plays, or a cycle loops the loader.
#[test]
fn field_tracks_are_checked_against_the_records_and_nesting_cycles_are_refused() {
    let head = "type timeline\nversion 1\n---\nduration 2\n";
    let bad = |body: &str| -> String {
        let root = project(
            "fields-bad",
            &[("timelines/t.ogt", &format!("{head}{body}"))],
        );
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(!report.ok(), "{body} cooked");
        format!("{:?}", report.errors())
    };
    let e = bad("track light \"lamp\" {\n  key 0 {\n    brightness 2\n  }\n}\n");
    assert!(e.contains("'brightness'") && e.contains("intensity"), "{e}");
    let e = bad(
        "track property \"lamp\" {\n  field \"og.PointLight.intensty\"\n  key 0 {\n    value 2\n  }\n}\n",
    );
    assert!(e.contains("og.PointLight has no field 'intensty'"), "{e}");
    let e = bad(
        "track property \"lamp\" {\n  field \"og.PointLight.cookie\"\n  key 0 {\n    value 2\n  }\n}\n",
    );
    assert!(e.contains("cannot ease or step"), "{e}");
    let e = bad("track light \"lamp\" {\n  key 0 {\n    color 1 0\n  }\n}\n");
    assert!(e.contains("'color'") && e.contains("3 numbers"), "{e}");
    let e = bad(
        "track property \"lamp\" {\n  field \"og.PostEffects.bloomIntensity\"\n  key 0 {\n    value 2\n  }\n}\n",
    );
    assert!(
        e.contains("is a resource, so the track names no entity"),
        "{e}"
    );
    let e = bad("track wobble {\n}\n");
    assert!(
        e.contains("'wobble'") && e.contains(&format!("og {}", env!("CARGO_PKG_VERSION"))),
        "{e}"
    );

    // a → b → a.
    let nest =
        |to: &str| format!("{head}track timeline {{\n  key 0 {{\n    play \"{to}\"\n  }}\n}}\n");
    let root = project(
        "nest-cycle",
        &[
            ("timelines/a.ogt", &nest("timelines/b.ogt")),
            ("timelines/b.ogt", &nest("timelines/a.ogt")),
        ],
    );
    let e = format!(
        "{:?}",
        cook_project(&root, &CookOptions::default())
            .unwrap()
            .errors()
    );
    assert!(
        e.contains("timelines/a.ogt → timelines/b.ogt → timelines/a.ogt"),
        "{e}"
    );
    // Six deep: refused at the fifth level.
    let mut files: Vec<(String, String)> = (0..5)
        .map(|i| {
            (
                format!("timelines/n{i}.ogt"),
                nest(&format!("timelines/n{}.ogt", i + 1)),
            )
        })
        .collect();
    files.push((
        "timelines/n5.ogt".into(),
        format!("{head}track marker {{\n  key 1 {{\n    name \"x\"\n  }}\n}}\n"),
    ));
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let root = project("nest-deep", &refs);
    let e = format!(
        "{:?}",
        cook_project(&root, &CookOptions::default())
            .unwrap()
            .errors()
    );
    assert!(e.contains("nest at most 4 deep"), "{e}");
    // Four deep cooks.
    let root = project("nest-four", &refs[1..]);
    let r = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(r.ok(), "{:?}", r.errors());
}
