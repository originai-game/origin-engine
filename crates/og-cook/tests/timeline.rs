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

use og_cook::{CookOptions, cook_project};
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

fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
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

const SHOTS: &str = r#"type timeline
version 1
---
duration 4
track shot {
  key 2 {
    none
  }
  key 0.5 {
    camera "wide"
    blend 0.75
    ease power2.out
  }
}
"#;

/// R10 batch 3's shot track cooks to its canonical text (keys in time
/// order, a key's blend only when it has one), pinned, in the same blob
/// version; a key naming neither a camera nor `none`, and a negative blend,
/// are refused naming the fix.
///
/// Ways this could fail: the cook drops the track (a cutscene cooks and
/// never cuts), writes keys out of order, or bumps the blob so every
/// timeline re-cooks.
#[test]
fn a_shot_track_cooks_to_its_pinned_text_and_a_key_without_a_camera_is_refused() {
    let root = project("shots", &[("timelines/t.ogt", SHOTS)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let blob = std::fs::read(root.join("cooked/timelines/t.ogt.ogb")).unwrap();
    let at = blob
        .windows(13)
        .position(|w| w == b"type timeline")
        .expect("the canonical text");
    let mut text = &blob[at..];
    while let [rest @ .., 0] = text {
        text = rest;
    }
    let text = std::str::from_utf8(text).unwrap();
    let wide = text.find("camera \"wide\"").expect(text);
    let none = text.find("none").expect(text);
    assert!(wide < none, "keys in time order: {text}");
    assert!(
        text.contains("blend 0.75") && text.contains("ease power2.out"),
        "{text}"
    );
    assert_eq!(
        (blob.len(), format!("{:016x}", fnv(text.as_bytes())), {
            use og_cook::format::BlobKind;
            og_cook::format::Timeline::schema_version()
        }),
        (338, "eb323fe751ccfc50".to_string(), 1),
        "{text}"
    );
    let bad = |body: &str| -> String {
        let root = project(
            "shots-bad",
            &[(
                "timelines/t.ogt",
                &format!("type timeline\nversion 1\n---\nduration 2\ntrack shot {{\n{body}}}\n"),
            )],
        );
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(!report.ok(), "{body} cooked");
        format!("{:?}", report.errors())
    };
    let e = bad("  key 0 {\n    blend 1\n  }\n");
    assert!(e.contains("names its camera") && e.contains("none"), "{e}");
    let e = bad("  key 0 {\n    camera \"wide\"\n    blend -1\n  }\n");
    assert!(e.contains("blend is 0"), "{e}");
}
