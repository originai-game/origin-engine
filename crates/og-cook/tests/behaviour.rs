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
  selector {
    sequence flee {
      check Foe.hp below Tuning.impFleeHp
      flee {
        distance 6
      }
    }
    sequence bite {
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
