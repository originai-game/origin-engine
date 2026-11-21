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

/// og.flow's dialogues: a flow node's segment is a timeline with one video
/// track of its own and its markers in order, at most one choice is
/// `#default`, a flow node says no Yarn lines, only flow nodes offer
/// choices, a `#hotspot` names a hotspot of its segment, a `#qte` is one of
/// its forms and an `og-flow-dissolve` is 0 to 5 s. Each is an error of the
/// dialogue; a flow that keeps them cooks.
#[test]
fn a_flow_dialogue_is_refused_for_each_segment_default_line_and_choice_it_gets_wrong() {
    let segment = |tracks: &str| format!("type timeline\nversion 1\n---\nduration 4\n{tracks}");
    let video = "track video {\n  key 0 {\n    video \"clips/a.ogt\"\n  }\n}\n";
    let spot = "track hotspot {\n  key 1 {\n    id \"knife\"\n    rect 0.1 0.1 0.2 0.2\n  }\n}\n";
    let late = "track marker {\n  key 3 {\n    name \"flow.open\"\n  }\n  key 2 {\n    name \"flow.lock\"\n  }\n}\n";
    let head = "#og-profile:yarn-spinner-3.2.2\n#og-name:";
    let good = format!(
        "{head}good\ntitle: a\nog-flow: segments/spot.ogt\n---\n-> Go #line:good.go #default\n    <<jump b>>\n-> Stay #line:good.stay #hotspot:knife\n    <<jump b>>\n-> Run #line:good.run #qte:hold:run:500\n    <<jump b>>\n===\ntitle: b\nog-flow: segments/ok.ogt\nog-flow-dissolve: 0.5\n---\n===\n"
    );
    let bad = format!(
        "{head}bad\ntitle: a\nog-flow: segments/missing.ogt\n---\n<<jump b>>\n===\ntitle: b\nog-flow: segments/two.ogt\n---\n<<jump c>>\n===\ntitle: c\nog-flow: segments/late.ogt\n---\n<<jump d>>\n===\ntitle: d\nog-flow: segments/ok.ogt\n---\nMae: Hello. #line:bad.hello\n-> One #line:bad.one #default\n    <<jump e>>\n-> Two #line:bad.two #default\n    <<jump e>>\n===\ntitle: e\n---\n-> Left #line:bad.left\n    <<jump d>>\n===\ntitle: g\nog-flow: segments/spot.ogt\nog-flow-dissolve: 9\n---\n-> X #line:bad.x #hotspot:fork\n    <<jump d>>\n-> Y #line:bad.y #qte:tap:jump\n    <<jump d>>\n===\n"
    );
    let root = project(
        "film",
        &[
            ("story/good.yarn", &good),
            ("story/bad.yarn", &bad),
            ("segments/ok.ogt", &segment(video)),
            ("segments/spot.ogt", &segment(&format!("{video}{spot}"))),
            ("segments/two.ogt", &segment(&format!("{video}{video}"))),
            ("segments/late.ogt", &segment(&format!("{video}{late}"))),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    let of = |rel: &str| -> Vec<&String> { errors.iter().filter(|e| e.starts_with(rel)).collect() };
    assert!(of("assets/story/good.yarn").is_empty(), "{errors:?}");
    let bad = of("assets/story/bad.yarn");
    assert_eq!(bad.len(), 1, "{errors:?}");
    let e = bad[0];
    for want in [
        "node 'a': its segment 'segments/missing.ogt' is no timeline",
        "node 'b': segment segments/two.ogt: it has 2 video tracks",
        "node 'c': segment segments/late.ogt: its markers are out of order",
        "node 'd': 2 choices are #default",
        "node 'd' says Yarn lines (line 21)",
        "node 'e' offers choices but has no og-flow segment",
        "node 'g': og-flow-dissolve: 9 is not a dissolve's seconds",
        "node 'g': choice 0 is #hotspot:fork, which its segment segments/spot.ogt has no `track hotspot` key for",
        "node 'g': choice 1: #qte:tap:jump is not one of",
    ] {
        assert!(e.contains(want), "{want}: {e}");
    }
}
