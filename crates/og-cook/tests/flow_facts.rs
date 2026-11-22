//! `og flow list` and `og flow routes` read a project's flows from its
//! sources: a choice through a condition reaches every branch, a held
//! segment's outcome is a step, an ending nothing reaches is named, and a
//! missing segment or placeholder footage is a problem.

use og_cook::cook::flow::{list, routes};
use serde_json::Value;
use std::path::PathBuf;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-flow")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    root
}

const STORY: &str = "#og-profile:yarn-spinner-3.2.2
#og-name:film
title: start
og-flow: segments/a.ogt
---
<<declare $lucky = false>>
-> Go #line:go
    <<jump fork>>
-> Stay #line:stay
    <<jump held>>
===
title: fork
---
<<if $lucky>>
    <<jump good>>
<<else>>
    <<jump bad>>
<<endif>>
===
title: held
og-flow: segments/a.ogt
---
-> won #line:won #outcome:won
    <<jump good>>
===
title: good
og-flow: segments/live.ogt
og-flow-ending: good
---
===
title: bad
og-flow: segments/missing.ogt
og-flow-ending: bad
---
===
title: secret
og-flow: segments/a.ogt
og-flow-ending: secret
---
===
";

const A: &str = "type timeline\nversion 1\n---\nduration 2\ntrack video {\n  key 0 {\n    video \"clips/a.ogt\"\n  }\n}\ntrack marker {\n  key 1 {\n    name \"flow.open\"\n  }\n  key 1.5 {\n    name \"flow.lock\"\n  }\n  key 1.8 {\n    name \"flow.cut\"\n  }\n}\n";
const LIVE: &str = "type timeline\nversion 1\n---\nduration 2\ntrack scene {\n  key 0 {\n    scene \"scenes/end.ogt\"\n  }\n}\n";
const CLIP: &str = "type video\nversion 1\n---\nsource \"a.mp4\"\nstream true\nplaceholder true\n";

fn film(name: &str) -> PathBuf {
    project(
        name,
        &[
            (
                "project.ogt",
                "type project\nversion 1\n---\nname \"film\"\nflow \"story/film.yarn\"\n",
            ),
            ("assets/story/film.yarn", STORY),
            ("assets/segments/a.ogt", A),
            ("assets/segments/live.ogt", LIVE),
            ("assets/clips/a.ogt", CLIP),
            ("assets/clips/a.mp4", ""),
            (
                "assets/scenes/end.ogt",
                "type scene\nversion 1\n---\nname \"End\"\n",
            ),
        ],
    )
}

fn node<'a>(flow: &'a Value, title: &str) -> &'a Value {
    flow["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["node"] == title)
        .unwrap_or_else(|| panic!("no node {title}: {flow:#}"))
}

#[test]
fn flow_list_reads_segments_footage_and_where_each_choice_leads() {
    let out = list(&film("list")).unwrap();
    let flow = &out["flows"][0];
    assert_eq!(flow["file"], "story/film.yarn");
    assert_eq!(flow["declared"], true);
    // The logic node is no flow node; its choice reaches both branches.
    assert!(
        flow["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|n| n["node"] != "fork")
    );
    let start = node(flow, "start");
    assert_eq!(
        start["choices"][0]["to"],
        serde_json::json!(["bad", "good"])
    );
    assert_eq!(start["choices"][1]["to"], serde_json::json!(["held"]));
    assert_eq!(start["segment"]["kind"], "footage");
    assert_eq!(start["segment"]["footage"]["placeholder"], true);
    assert!(
        start["problems"][0]
            .as_str()
            .unwrap()
            .contains("placeholder"),
        "{start:#}"
    );
    let good = node(flow, "good");
    assert_eq!(good["segment"]["kind"], "live");
    assert_eq!(good["segment"]["scenes"][0]["exists"], true);
    assert_eq!(good["problems"], serde_json::json!([]));
    assert_eq!(node(flow, "held")["choices"][0]["outcome"], "won");
    let bad = node(flow, "bad");
    assert!(
        bad["problems"][0]
            .as_str()
            .unwrap()
            .contains("segments/missing.ogt"),
        "{bad:#}"
    );
}

#[test]
fn flow_routes_walk_every_way_to_an_ending_and_name_the_unreached() {
    let out = routes(&film("routes")).unwrap();
    let flow = &out["flows"][0];
    let ends: Vec<(&str, usize)> = flow["routes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["end"].as_str().unwrap(),
                r["steps"].as_array().unwrap().len(),
            )
        })
        .collect();
    assert_eq!(ends, [("bad", 1), ("good", 1), ("good", 2)], "{flow:#}");
    let first = &flow["routes"][0]["steps"][0];
    assert_eq!(first["choose"], 0);
    assert_eq!(first["condition"], true);
    let held = &flow["routes"][2]["steps"];
    assert_eq!(held[0]["choose"], 1);
    assert!(held[0].get("condition").is_none());
    assert_eq!(held[1]["advance"], "won");
    assert_eq!(flow["unreached"], serde_json::json!(["secret"]));
    assert_eq!(flow["truncated"], false);
}
