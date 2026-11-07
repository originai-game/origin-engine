//! Dialogues and quests through the cook (G2-8): a valid one cooks to the
//! IR the runtime loads, a broken one is refused with its line and the
//! fix, and a key the base string table lacks, or an argument its message
//! takes that nothing passes, is warned about.

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-story")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

const SMITH: &str = "#og-profile:yarn-spinner-3.2.2\n#og-name:smith\ntitle: greet\n---\nsmith: Hello. #line:smith.hello\nsmith: Price #line:smith.price\n-> Choose #line:smith.missing\n===\n";
const QUEST: &str = "type quest\nversion 1\n---\nid rescue\ntitle \"quest.rescue\"\nstage hunt {\n  objective wolves \"quest.wolves\" {\n    target 3\n  }\n}\n";
const EN: &str = "type strings\nversion 1\n---\nlocale \"en\"\nbase true\nstring \"npc.smith\" \"Brannoc\"\nstring \"smith.hello\" \"Hello.\"\nstring \"smith.price\" \"That's {price} gold.\"\nstring \"quest.rescue\" \"Rescue\"\nstring \"quest.wolves\" \"Wolves {count}/{target}\"\n";

#[test]
fn dialogues_and_quests_cook_and_their_keys_are_checked_against_the_base_table() {
    let bad =
        "#og-profile:yarn-spinner-3.2.2\n#og-name:bad\ntitle: a\n---\n<<jump nowhere>>\n===\n";
    let root = project(
        "keys",
        &[
            ("story/smith.yarn", SMITH),
            ("story/rescue.ogt", QUEST),
            ("story/bad.yarn", bad),
            (
                "story/missing.yarn",
                &bad.replace("<<jump nowhere>>", "Missing id"),
            ),
            ("strings/en.ogt", EN),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors[0].starts_with("assets/story/bad.yarn")
            && errors[0].contains("line 5")
            && errors[0].contains("'nowhere' names no node"),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("assets/story/missing.yarn")
                && e.contains("line 5")
                && e.contains("append #line:unique_id")),
        "{errors:?}"
    );
    let w = &report.warnings;
    assert!(
        !w.iter()
            .any(|w| w.contains("'smith.missing' is not in the base string table")),
        "inline Yarn text supplies its base string: {w:?}"
    );
    assert!(
        w.iter().any(|w| w.starts_with("assets/story/smith.yarn")
            && w.contains("line 6")
            && w.contains("takes {price}")),
        "{w:?}"
    );
    // The quest's objective is passed count and target by the log: no warning.
    assert!(!w.iter().any(|w| w.contains("rescue.ogt")), "{w:?}");

    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for (id, text) in [("story/smith.yarn", SMITH), ("story/rescue.ogt", QUEST)] {
        let cooked = bundle.story_text(id, &bundle.read(id).unwrap()).unwrap();
        if id.ends_with(".yarn") {
            let ir = og_story::Dialogue::decode(&cooked).unwrap();
            assert_eq!(ir, og_story::Dialogue::parse(text).unwrap());
            assert_eq!(ir.strings["smith.missing"], "Choose");
        } else {
            assert_eq!(cooked, text);
        }
    }
    assert!(
        bundle
            .story_text("strings/en.ogt", &bundle.read("strings/en.ogt").unwrap())
            .is_err(),
        "a string table is not a story"
    );
}

#[test]
fn registered_yarn_functions_typecheck_extract_and_recook_when_the_body_changes() {
    let source = SMITH.replace("smith: Hello.", "smith: {score(3, 4)}");
    let root = project("functions", &[("story/smith.yarn", &source)]);
    let config = "type project\nversion 1\n---\nstoryFunctions {\n function score number {\n  param amount number\n  param scale number\n  body \"$amount * $scale\"\n }\n}\n";
    for (body, expected) in [
        (config.to_owned(), 12.0),
        (config.replace(" * ", " + "), 7.0),
    ] {
        std::fs::write(root.join("project.ogt"), body).unwrap();
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        let extracted = og_cook::cook::strings_extract::extract(&root).unwrap();
        assert!(extracted.errors.is_empty(), "{:?}", extracted.errors);
        let bundle = Bundle::open(&root.join("cooked")).unwrap();
        let text = bundle
            .story_text(
                "story/smith.yarn",
                &bundle.read("story/smith.yarn").unwrap(),
            )
            .unwrap();
        let dialogue = og_story::Dialogue::decode(&text).unwrap();
        let lib = og_story::Library {
            dialogues: std::collections::BTreeMap::from([(
                "smith".into(),
                std::sync::Arc::new(dialogue),
            )]),
            ..Default::default()
        };
        let mut story = og_story::Story::new(og_core::Rng::new(1).fork("story"));
        let id = story
            .start(&lib, "smith", Default::default(), Default::default(), None)
            .unwrap();
        assert_eq!(
            story.convs[&id].history[0].args[0].1,
            og_story::Val::Num(expected)
        );
    }
    std::fs::write(
        root.join("assets/story/smith.yarn"),
        source.replace("score(3, 4)", "score(true, 4)"),
    )
    .unwrap();
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(
        report
            .errors()
            .iter()
            .any(|error| error.contains("expected Number, found Bool")),
        "{:?}",
        report.errors()
    );
    assert!(
        !og_cook::cook::strings_extract::extract(&root)
            .unwrap()
            .errors
            .is_empty()
    );
    std::fs::write(
        root.join("project.ogt"),
        config.replace("$amount * $scale", "random()"),
    )
    .unwrap();
    assert!(
        cook_project(&root, &CookOptions::default())
            .unwrap_err()
            .to_string()
            .contains("pure function cannot call stateful 'random'")
    );
}
