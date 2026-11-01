//! Art slots (docs/ui.md §13): every kind's defaults are a sound spec, a
//! wrong spec is refused with its fix, and the art brief
//! (packages/knowledge/scripts/brief.ts, `UI_KINDS`) asks for the same
//! aspects, focal points, safe areas and fits og-ui's kinds declare.
//!
//! Ways this could fail: a kind whose own defaults its validation refuses;
//! a validation that lets a portrait through at 16:9 or an item drawn at 1:1
//! for a 2×3 cell; a focal point under the text it is covered by; the brief
//! and the engine drifting apart, so generated art lands in a slot of
//! another shape.

use og_ui::slot::{Aspect, Fit, ITEM_CELLS, Mask, SlotAspect, SlotKind, SlotSpec};

#[test]
fn every_kinds_defaults_are_a_sound_spec() {
    for k in SlotKind::ALL {
        let s = SlotSpec::new("art", *k);
        assert_eq!(s.validate(), Vec::<String>::new(), "{}", k.name());
    }
}

#[test]
fn a_wrong_slot_is_refused_with_its_fix() {
    let mut bust = SlotSpec::new("hero", SlotKind::Bust);
    bust.aspect = SlotAspect::One(Aspect::new(16, 9));
    let e = bust.validate().join("\n");
    assert!(e.contains("drawn at 1:1, 3:4, not 16:9"), "{e}");

    // A full body takes anything from 1:2 to 9:16, nothing wider.
    let mut body = SlotSpec::new("hero", SlotKind::FullBody);
    body.aspect = SlotAspect::One(Aspect::new(5, 9));
    assert!(body.validate().is_empty(), "{:?}", body.validate());
    body.aspect = SlotAspect::One(Aspect::new(3, 4));
    assert!(!body.validate().is_empty());

    // Key art keeps its subject out of the side the title sits on.
    let mut key = SlotSpec::new("title", SlotKind::KeyArt);
    key.focal = [0.2, 0.5];
    let e = key.validate().join("\n");
    assert!(e.contains("focal point lies in the safe rectangle"), "{e}");

    // A 2×3 item is drawn 2:3, and only the inventory's cell shapes exist.
    let mut item = SlotSpec::new("sword", SlotKind::ItemIcon);
    item.cells = Some([2, 3]);
    let e = item.validate().join("\n");
    assert!(e.contains("2×3 item is drawn at 2:3, not 1:1"), "{e}");
    item.aspect = SlotAspect::One(Aspect::new(2, 3));
    assert!(item.validate().is_empty(), "{:?}", item.validate());
    item.cells = Some([3, 3]);
    assert!(item.validate().join("\n").contains("not 3×3"));
    assert!(ITEM_CELLS.contains(&[2, 4]));

    // Style art is a nine-slice with insets that leave a middle; a button draws its normal state.
    let mut button = SlotSpec::new("ok", SlotKind::StyleButton);
    button.insets = Some([0.5, 0.25, 0.5, 0.25]);
    button.states.retain(|s| s != "normal");
    let e = button.validate().join("\n");
    assert!(
        e.contains("leave a middle") && e.contains("normal state"),
        "{e}"
    );
    let mut frame = SlotSpec::new("panel", SlotKind::StyleFrame);
    frame.insets = None;
    assert!(frame.validate().join("\n").contains("declares its insets"));

    // Opaque art has no alpha to cut by; a banner does not nine-slice.
    let mut banner = SlotSpec::new("event", SlotKind::Banner);
    banner.mask = Mask::Alpha;
    banner.fit = Fit::NineSlice;
    let e = banner.validate().join("\n");
    assert!(
        e.contains("no alpha") && e.contains("not a nine-slice"),
        "{e}"
    );
}

/// The brief's line for a kind: `  <name>: ui("<about>", [<aspects>], <transparent>, <focal>, "<fit>"[, { … }]),`.
fn brief_line<'a>(src: &'a str, name: &str) -> &'a str {
    src.lines()
        .find(|l| l.trim_start().starts_with(&format!("{name}: ui(")))
        .unwrap_or_else(|| panic!("brief.ts's UI_KINDS has no {name}"))
}

fn aspects(list: &str) -> Vec<Aspect> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            let (w, h) = s.trim_matches('"').split_once(':').expect("w:h");
            Aspect::new(w.parse().unwrap(), h.parse().unwrap())
        })
        .collect()
}

#[test]
fn the_art_brief_asks_for_the_slots_og_ui_declares() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/knowledge/scripts/brief.ts"),
    )
    .expect("brief.ts");
    let kinds = src
        .split("export const UI_KINDS = {")
        .nth(1)
        .and_then(|s| s.split("} as const").next())
        .expect("UI_KINDS");
    assert_eq!(
        kinds.lines().filter(|l| l.contains(": ui(")).count(),
        SlotKind::COUNT,
        "one brief kind per SlotKind"
    );
    for k in SlotKind::ALL {
        let i = k.info();
        let line = brief_line(kinds, k.name());
        let after = line.split_once("\", [").expect("about, then aspects").1;
        let (list, rest) = after.split_once(']').expect("aspects");
        assert_eq!(aspects(list), i.aspects, "{}: aspects", k.name());
        let rest = rest.trim_start_matches(", ");
        assert!(
            rest.starts_with(if i.transparent { "true" } else { "false" }),
            "{}: transparent",
            k.name()
        );
        let focal = if i.focal == [0.5, 0.5] {
            "centre".to_string()
        } else {
            format!("[{}, {}]", i.focal[0], i.focal[1])
        };
        assert!(
            rest.contains(&format!(", {focal}, ")),
            "{}: focal {focal}",
            k.name()
        );
        let fit = match i.fit {
            Fit::Cover => "cover",
            Fit::Contain => "contain",
            Fit::Fill => "fill",
            Fit::NineSlice => "nineSlice",
            Fit::Tile => "tile",
        };
        assert!(rest.contains(&format!("\"{fit}\"")), "{}: fit", k.name());
        match i.safe {
            Some(s) => {
                let want = format!("safe: [{}]", s.map(|v| v.to_string()).join(", "));
                assert!(line.contains(&want), "{}: {want}", k.name());
            }
            None => assert!(!line.contains("safe:"), "{}: no safe area", k.name()),
        }
        if i.legible_px > 0 {
            assert!(
                line.contains(&format!("legiblePx: {}", i.legible_px)),
                "{}: legible",
                k.name()
            );
        }
        if let Some((lo, hi)) = i.range {
            assert!(
                line.contains(&format!("range: [\"{lo}\", \"{hi}\"]")),
                "{}: range",
                k.name()
            );
        }
        assert_eq!(
            line.contains("nineSlice: true"),
            i.nine_slice,
            "{}",
            k.name()
        );
        assert_eq!(
            line.contains("generated: false"),
            !i.generated,
            "{}",
            k.name()
        );
    }
}

const ARTS: &str = r#"type screen
version 1
---
name "title"
column Root {
  art Hero {
    kind keyArt
    ratio 16 9
    focal 0.7 0.4
    width 320
  }
  row Party {
    art Bust {
      kind bust
      width 96
    }
  }
}
"#;

/// `og ui slots`'s reading: every `art` element of a document, its path
/// as the probe names it and its spec as written (a kind's defaults where
/// a prop is left out). Ways it could fail: a nested slot named without
/// its ancestors, the written ratio or focal point dropped for the kind's.
#[test]
fn a_documents_art_slots_are_listed_with_their_specs() {
    let doc = og_ui::UiDocument::parse("ui/title.ogt", ARTS).expect("parses");
    let slots = og_ui::slot::document_slots(&doc);
    type Row<'a> = (&'a str, SlotKind, Option<Aspect>, [f32; 2], bool);
    let got: Vec<Row> = slots
        .iter()
        .map(|s| {
            let a = match s.spec.aspect {
                SlotAspect::One(a) => Some(a),
                _ => None,
            };
            (s.path.as_str(), s.spec.kind, a, s.spec.focal, s.textured)
        })
        .collect();
    let bust = SlotSpec::new("x", SlotKind::Bust);
    let bust_aspect = match bust.aspect {
        SlotAspect::One(a) => Some(a),
        _ => None,
    };
    assert_eq!(
        got,
        vec![
            (
                "title/Root/Hero",
                SlotKind::KeyArt,
                Some(Aspect::new(16, 9)),
                [0.7, 0.4],
                false
            ),
            (
                "title/Root/Party/Bust",
                SlotKind::Bust,
                bust_aspect,
                bust.focal,
                false
            ),
        ]
    );
}

/// An empty slot draws the style's placeholder (a gradient of the palette
/// and the kind's silhouette), not a flat grey box; with its art it draws
/// the texture covered around its focal point.
#[test]
fn an_empty_slot_draws_a_placeholder_and_a_filled_one_its_art() {
    use crate::support::*;
    use og_ui::*;
    let build = |texture: Option<og_render::TextureHandle>| {
        let mut ui = ui_with_font(test_font_metrics());
        let mut spec = SlotSpec::new("hero", SlotKind::KeyArt);
        spec.focal = [0.7, 0.4];
        for _ in 0..2 {
            ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
            ui.art(
                "Hero",
                &ArtDesc {
                    spec: spec.clone(),
                    texture,
                    size: Some([1600.0, 900.0]),
                    width: Size::Fixed(320.0),
                    height: Size::fit(),
                    label: String::new(),
                },
            );
            ui.end();
        }
        let e = ui.frame().element("Hero").expect("the slot").clone();
        assert!(
            (e.width / e.height - 16.0 / 9.0).abs() < 0.02,
            "it takes the slot's aspect"
        );
        let paints: Vec<og_render::UiPaint> = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| match &i.cmd {
                DrawCmd::Shape { shape, .. } if shape.mode == og_render::UiMode::Fill => {
                    Some(shape.paint.clone())
                }
                _ => None,
            })
            .collect();
        (paints, ui.frame().items.len())
    };
    let (empty, _) = build(None);
    assert!(
        empty
            .iter()
            .any(|p| matches!(p, og_render::UiPaint::Gradient(g) if g.stops.len() == 2)),
        "a gradient ground: {empty:?}"
    );
    let tex = og_render::RenderScene::new().add_texture(og_render::TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    });
    let (filled, _) = build(Some(tex));
    assert!(
        filled
            .iter()
            .any(|p| matches!(p, og_render::UiPaint::Image(im)
            if im.texture == tex && im.fit == og_render::UiFit::Cover && im.focal.x == 0.7)),
        "its art, covered around its focal point: {filled:?}"
    );
}
