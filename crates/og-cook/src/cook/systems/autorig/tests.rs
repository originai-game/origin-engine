//! PSD art refused for what its layers cannot show, naming the layer and
//! the file; group parts merged; hidden and `_` layers left out.

use super::super::psd::layers::tests::{Out, psd};
use super::template::HUMANOID_SIDE;
use super::*;

/// Every humanoid-side part as a small block, touching its parent, on a
/// 40 x 80 canvas; `edit` changes the layers before they are written.
fn figure(edit: impl Fn(&mut Vec<Out>)) -> Vec<u8> {
    let c = [200, 100, 50, 255];
    let mut ls = vec![
        Out::new("pelvis", [40, 15, 48, 25], c),
        Out::new("torso", [20, 15, 42, 25], c),
        Out::new("head", [8, 14, 21, 26], c),
        Out::new("arm-upper-l", [21, 18, 32, 22], c),
        Out::new("arm-lower-l", [31, 18, 40, 22], c),
        Out::new("hand-l", [39, 18, 43, 22], c),
        Out::new("arm-upper-r", [21, 18, 32, 22], c),
        Out::new("arm-lower-r", [31, 18, 40, 22], c),
        Out::new("hand-r", [39, 18, 43, 22], c),
        Out::new("leg-upper-l", [46, 16, 60, 20], c),
        Out::new("leg-lower-l", [59, 16, 72, 20], c),
        Out::new("foot-l", [71, 16, 75, 24], c),
        Out::new("leg-upper-r", [46, 20, 60, 24], c),
        Out::new("leg-lower-r", [59, 20, 72, 24], c),
        Out::new("foot-r", [71, 20, 75, 28], c),
    ];
    edit(&mut ls);
    psd(40, 80, &ls)
}

fn read_psd(bytes: &[u8]) -> Result<parts::Art, String> {
    let dir = super::super::test_dir::TempDir::new("og-autorig");
    std::fs::create_dir_all(dir.join("chars")).unwrap();
    let name = format!("chars/{:016x}.psd", super::super::hash_bytes(bytes));
    std::fs::write(dir.join(&name), bytes).unwrap();
    parts::read(&dir, &name, &HUMANOID_SIDE, &mut Vec::new())
        .map_err(|e| e.replace(&name, "chars/hero.psd"))
}

#[test]
fn the_figure_rigs() {
    let art = read_psd(&figure(|_| {})).unwrap();
    let packed = atlas::pack(&art.parts).unwrap();
    let rig = skeleton::rig(&art, &HUMANOID_SIDE, &[], &packed.rects).unwrap();
    assert_eq!(rig.slots.len(), 15);
}

#[test]
fn a_layer_with_effects_or_another_blend_mode_is_refused_by_name() {
    let e = read_psd(&figure(|ls| ls[2].blocks.push((*b"lfx2", vec![0; 8])))).unwrap_err();
    assert!(
        e.contains("chars/hero.psd: layer `head` has layer effects") && e.contains("Rasterize"),
        "{e}"
    );
    let e = read_psd(&figure(|ls| ls[1].blend = *b"mul ")).unwrap_err();
    assert!(
        e.contains("layer `torso` blends with the mode `mul`"),
        "{e}"
    );
}

#[test]
fn a_group_named_for_a_part_is_that_part_and_hidden_or_underscored_layers_are_left_out() {
    let art = read_psd(&figure(|ls| {
        // The head as a group of two layers with any names.
        let head = ls.remove(2);
        let mut eye = Out::new("eye", [12, 22, 14, 24], [0, 0, 0, 255]);
        eye.rle = true;
        ls.extend([
            Out::section("</Layer group>", 3),
            Out::new("face", head.rect, [200, 100, 50, 255]),
            eye,
            Out::section("Head", 1),
        ]);
        let mut sketch = Out::new("Sketch", [0, 0, 80, 40], [9, 9, 9, 255]);
        sketch.flags = 2;
        ls.push(sketch);
        ls.push(Out::new("_guide", [0, 0, 2, 2], [9, 9, 9, 255]));
    }))
    .unwrap();
    let head = art.part("head").unwrap();
    assert_eq!(
        (head.left, head.top, head.width, head.height),
        (14, 8, 12, 13)
    );
    // The eye is composited over the face.
    let px = |x: i32, y: i32| {
        &head.rgba[(((y - head.top) * head.width as i32 + x - head.left) * 4) as usize..][..4]
    };
    assert_eq!(px(22, 12), &[0, 0, 0, 255]);
    assert_eq!(px(16, 10), &[200, 100, 50, 255]);
    assert_eq!(art.parts.len(), 15);
}

#[test]
fn two_layers_of_one_part_are_refused() {
    let e = read_psd(&figure(|ls| {
        ls.push(Out::new("head", [0, 0, 2, 2], [1, 1, 1, 255]))
    }))
    .unwrap_err();
    assert!(
        e.contains("layer `head` is a second `head`") && e.contains("one group named `head`"),
        "{e}"
    );
}

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
