//! The engine's built-in look assets through the cook: a look that names
//! `"@canvas/<name>"` or `"@palettes/<name>"` (itself or through a preset it
//! extends) cooks with the built-in in the game's bundle under its reserved
//! id, loadable as any other texture or palette; a built-in nobody names
//! stays out; a game's own path starting with `@` is refused.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-look-builtins")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

#[test]
fn a_look_naming_built_ins_cooks_them_into_the_bundle() {
    let root = project(
        "named",
        &[
            (
                "looks/main.ogt",
                "type look\nversion 1\n---\nStylise {\n  paintCanvas \"@canvas/paper\"\n  paintCanvasStrength 0.5\n  palette \"@palettes/handheld4\"\n}\n",
            ),
            // Through a preset: watercolor-wash names @canvas/watercolor.
            (
                "looks/wash.ogt",
                "type look\nversion 1\n---\nextends \"watercolor-wash\"\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let ids: Vec<&str> = bundle.manifest().outputs().map(|o| o.id.as_str()).collect();
    for id in [
        "@canvas/paper.png",
        "@canvas/watercolor.png",
        "@palettes/handheld4.hex",
    ] {
        assert!(ids.contains(&id), "{id} in {ids:?}");
    }
    // Only what a look names.
    assert!(!ids.contains(&"@canvas/linen.png"), "{ids:?}");
    assert!(!ids.contains(&"@palettes/1bit.hex"), "{ids:?}");

    let tex = bundle.load_texture("@canvas/paper.png").unwrap();
    assert_eq!((tex.meta.width, tex.meta.height), (256, 256));
    let bytes = bundle.read("@palettes/handheld4.hex").unwrap();
    let colours = bundle
        .palette("@palettes/handheld4.hex", bytes.as_slice())
        .unwrap();
    assert_eq!(colours.len(), 4);
    // The look stores the bundle id, which og.load.texture resolves.
    let bytes = bundle.read("looks/main.ogt").unwrap();
    let look = bundle.look("looks/main.ogt", bytes.as_slice()).unwrap();
    let text = look.to_text();
    assert!(text.contains("paintCanvas \"@canvas/paper.png\""), "{text}");
    assert!(
        text.contains("palette \"@palettes/handheld4.hex\""),
        "{text}"
    );

    // A second cook finds every output fresh.
    let again = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(again.ok(), "{:?}", again.errors());
    assert!(again.cooked_outputs().is_empty(), "nothing re-cooks");
}

#[test]
fn a_games_own_at_path_is_refused() {
    let root = project("refused", &[("@canvas/paper.hex", "ffffff\n")]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("engine's built-in assets"), "{errors}");
}
