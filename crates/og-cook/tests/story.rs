//! Dialogues and quests through the cook (G2-8): a valid one cooks to the
//! text the runtime parses, a broken one is refused with its line and the
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

const SMITH: &str = "type dialogue\nversion 1\n---\nname smith\nspeaker smith {\n  name \"npc.smith\"\n}\nglobal gold 3\nnode greet {\n  say smith \"smith.hello\"\n  say smith \"smith.price\"\n  choice \"smith.missing\"\n}\n";
const QUEST: &str = "type quest\nversion 1\n---\nid rescue\ntitle \"quest.rescue\"\nstage hunt {\n  objective wolves \"quest.wolves\" {\n    target 3\n  }\n}\n";
const EN: &str = "type strings\nversion 1\n---\nlocale \"en\"\nbase true\nstring \"npc.smith\" \"Brannoc\"\nstring \"smith.hello\" \"Hello.\"\nstring \"smith.price\" \"That's {price} gold.\"\nstring \"quest.rescue\" \"Rescue\"\nstring \"quest.wolves\" \"Wolves {count}/{target}\"\n";

#[test]
fn dialogues_and_quests_cook_and_their_keys_are_checked_against_the_base_table() {
    let bad = "type dialogue\nversion 1\n---\nname bad\nnode a {\n  goto nowhere\n}\n";
    let root = project(
        "keys",
        &[
            ("story/smith.ogt", SMITH),
            ("story/rescue.ogt", QUEST),
            ("story/bad.ogt", bad),
            ("strings/en.ogt", EN),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].starts_with("assets/story/bad.ogt")
            && errors[0].contains("line 6")
            && errors[0].contains("'nowhere' names no node"),
        "{errors:?}"
    );
    let w = &report.warnings;
    assert!(
        w.iter().any(|w| w.starts_with("assets/story/smith.ogt")
            && w.contains("line 12")
            && w.contains("'smith.missing' is not in the base string table")),
        "{w:?}"
    );
    assert!(
        w.iter().any(|w| w.starts_with("assets/story/smith.ogt")
            && w.contains("line 11")
            && w.contains("takes {price}")),
        "{w:?}"
    );
    // The quest's objective is passed count and target by the log: no warning.
    assert!(!w.iter().any(|w| w.contains("rescue.ogt")), "{w:?}");

    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for (id, text) in [("story/smith.ogt", SMITH), ("story/rescue.ogt", QUEST)] {
        let cooked = bundle.story_text(id, &bundle.read(id).unwrap()).unwrap();
        assert_eq!(cooked, text);
    }
    assert!(
        bundle
            .story_text("strings/en.ogt", &bundle.read("strings/en.ogt").unwrap())
            .is_err(),
        "a string table is not a story"
    );
}
