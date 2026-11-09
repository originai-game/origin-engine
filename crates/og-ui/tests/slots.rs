//! Art slots (docs/ui.md §3.3): every kind's defaults are a sound spec, a
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
fn missing_art_marks_stay_small_and_square_in_every_slot_kind() {
    use og_ui::*;
    for &kind in SlotKind::ALL {
        for (width, height) in [(390.0, 844.0), (1280.0, 720.0), (32.0, 96.0)] {
            let mut ui = Ui::new(UiDesc::default());
            ui.set_style_name("placeholder-test");
            ui.begin(InputFrame::default(), Viewport::new(1280.0, 900.0));
            ui.art(
                "Art",
                &ArtDesc {
                    spec: SlotSpec::new("fixture", kind),
                    width: Size::Fixed(width),
                    height: Size::Fixed(height),
                    texture: None,
                    size: None,
                    label: String::new(),
                    offset: [0.0; 2],
                    nearest: false,
                },
            );
            ui.end();
            let marks: Vec<_> = ui
                .frame()
                .items
                .iter()
                .filter_map(|item| {
                    if let DrawCmd::Quad { rect, radius, .. } = item.cmd {
                        (radius > 0.0).then_some(rect)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(marks.len(), 1, "{}", kind.name());
            assert_eq!(
                marks[0].width,
                marks[0].height,
                "{} {width}×{height}",
                kind.name()
            );
            assert!(marks[0].width <= 32.0, "{}: {:?}", kind.name(), marks[0]);
        }
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
            Fit::None => "none",
            Fit::ScaleDown => "scaleDown",
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
