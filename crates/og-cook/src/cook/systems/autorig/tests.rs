//! The rig templates and the knowledge package's `rigParts` (brief.ts)
//! cannot drift apart.

use super::*;

/// `rigParts` of @origingame/create-game (the knowledge package's
/// scripts/brief.ts, docs/art.md §2.8) carries each template's parts as
/// data between its `rig-parts` markers: every part, with the fields the
/// cook rigs it by, and the template's `sides`, equal to these templates,
/// so neither changes without the other.
#[test]
fn rig_parts_match_the_templates() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/knowledge/scripts/brief.ts");
    let text = std::fs::read_to_string(&path).unwrap();
    let block = text
        .split("// rig-parts: begin")
        .nth(1)
        .and_then(|t| t.split("// rig-parts: end").next())
        .expect("brief.ts has its rig-parts markers");
    let json = &block[block.find("= {").expect("const RIG_PARTS = {") + 2..];
    let json = json
        .trim()
        .trim_end_matches(';')
        .trim_end_matches("as const")
        .trim();
    let data: serde_json::Value = serde_json::from_str(json).unwrap();
    let mut names: Vec<&str> = data
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    names.sort();
    let mut want = template::NAMES.to_vec();
    want.sort();
    assert_eq!(names, want, "rigParts' templates");
    for t in template::ALL {
        let ts = &data[t.name];
        assert_eq!(ts["sides"], t.sides, "{}: sides", t.name);
        let parts = ts["parts"].as_array().unwrap();
        assert_eq!(
            parts.len(),
            t.parts.len(),
            "{}: the number of parts",
            t.name
        );
        for s in t.parts {
            let p = parts
                .iter()
                .find(|p| p["name"] == s.name)
                .unwrap_or_else(|| panic!("{}: rigParts has no `{}`", t.name, s.name));
            let (shape, bones) = match s.shape {
                template::Shape::Rigid => ("rigid", 1),
                template::Shape::Limb => ("limb", 1),
                template::Shape::Chain(k) => ("chain", k),
            };
            let end = match s.end {
                template::End::Top => "top",
                template::End::Bottom => "bottom",
                template::End::Centre => "centre",
                template::End::Near => "near",
            };
            let expected = serde_json::json!({
                "name": s.name,
                "required": s.required,
                "parents": s.parents,
                "end": end,
                "shape": shape,
                "bones": bones,
                "what": s.what,
            });
            assert_eq!(p, &expected, "{}: `{}` in brief.ts", t.name, s.name);
        }
    }
}
