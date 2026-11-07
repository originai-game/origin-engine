//! Behaviours through the cook: a check compared with a tuning field.
//!
//! Ways this could fail: the cook refuses `check <operand> <op>
//! <Resource.field>` (Kestrel Stair FINDINGS #3: thresholds had to be copied
//! into the .ogt as literals), or accepts a malformed comparison without its
//! line; the cooked text is not the text the runtime parses.

use og_ai::{Behaviour, Kind};
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-behaviour")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

const IMP: &str = r#"type behaviour
version 1
---
name "imp"
key reach 1
tree {
  ReactiveFallback {
    ReactiveSequence flee {
      check Foe.hp below Tuning.impFleeHp
      flee {
        distance 6
      }
    }
    ReactiveSequence bite {
      check targetDistance atMost reach
      command bite
    }
    wait 1
  }
}
"#;

#[test]
fn a_check_against_a_tuning_field_cooks_and_a_malformed_one_is_refused_with_its_line() {
    let bad = "type behaviour\nversion 1\n---\ntree {\n  check Foe.hp below 1 2\n}\n";
    let root = project("refs", &[("ai/imp.ogt", IMP), ("ai/bad.ogt", bad)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].starts_with("assets/ai/bad.ogt") && errors[0].contains("line 5"),
        "{errors:?}"
    );
    assert!(
        errors[0].contains("Tuning.fleeHp"),
        "names the fix: {errors:?}"
    );
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let text = bundle
        .behaviour_text("ai/imp.ogt", &bundle.read("ai/imp.ogt").unwrap())
        .unwrap();
    assert_eq!(text, IMP);
    let b = Behaviour::parse(&text).unwrap();
    let against: Vec<_> = b
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            Kind::Check(c) => c.against.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(against, vec!["Tuning.impFleeHp", "reach"]);
}

const PATROL: &str = "type behaviour\nversion 1\n---\nname \"patrol\"\nkey lap 0\nroute beat {\n  point 0 0 0\n  point 4 0 0\n}\ntree {\n  patrol beat\n}\n";

fn guard(sub: &str) -> String {
    format!(
        "type behaviour\nversion 1\n---\nname \"guard\"\ntree {{\n  ReactiveFallback {{\n    ReactiveSequence fight {{\n      check targetDistance atMost 2\n      command bite\n    }}\n    use \"{sub}\"\n  }}\n}}\n"
    )
}

/// `use` through the cook: the subtree is read from the assets root, the
/// stamp covers it (an edited subtree re-validates the tree using it), and
/// a cycle through files is refused naming the chain.
#[test]
fn use_reads_subtrees_from_files_recooks_on_their_edit_and_refuses_a_cycle() {
    let guard_text = guard("ai/patrol.ogt");
    let root = project(
        "use",
        &[("ai/patrol.ogt", PATROL), ("ai/guard.ogt", &guard_text)],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.errors().is_empty(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let text = bundle
        .behaviour_text("ai/guard.ogt", &bundle.read("ai/guard.ogt").unwrap())
        .unwrap();
    let mut read = |p: &str| {
        bundle
            .behaviour_text(p, &bundle.read(p).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    let b = Behaviour::parse_with(&text, "ai/guard.ogt", &mut read).unwrap();
    assert_eq!(b.subtrees, ["ai/patrol.ogt"]);
    assert!(b.keys.iter().any(|(k, _)| k == "lap"), "the subtree's keys");

    let cooked = |r: &og_cook::Report, src: &str| {
        r.assets
            .iter()
            .find(|a| a.source.ends_with(src))
            .map(|a| a.outputs.iter().any(|o| o.cooked))
    };
    // Unchanged: nothing re-cooks.
    let again = cook_project(&root, &CookOptions::default()).unwrap();
    assert_eq!(cooked(&again, "ai/guard.ogt"), Some(false));
    // The subtree becomes a machine: the guard re-cooks and is refused.
    let machine = "type behaviour\nversion 1\n---\nmachine {\n  initial idle\n  state idle {\n    wait 1\n  }\n}\n";
    std::fs::write(root.join("assets/ai/patrol.ogt"), machine).unwrap();
    let edited = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = edited.errors();
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("assets/ai/guard.ogt") && e.contains("is a machine")),
        "{errors:?}"
    );

    // a → b → a.
    let a = "type behaviour\nversion 1\n---\ntree {\n  use \"ai/b.ogt\"\n}\n";
    let b = "type behaviour\nversion 1\n---\ntree {\n  use \"ai/a.ogt\"\n}\n";
    let root = project("use-cycle", &[("ai/a.ogt", a), ("ai/b.ogt", b)]);
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors();
    assert!(
        errors.iter().any(|e| e.starts_with("assets/ai/a.ogt")
            && e.contains("use makes a cycle: ai/a.ogt → ai/b.ogt → ai/a.ogt")),
        "{errors:?}"
    );
}
