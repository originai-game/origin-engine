//! Authored packs keep their assets and parse independently of bootstrap choices.
//! Contrast thresholds are not acceptance criteria (docs/ui.md §17).

#[path = "support.rs"]
mod pack_support;
#[path = "../../../crates/og-ui/tests/support/mod.rs"]
mod support;
use og_ui::*;
use pack_support::*;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles")
}

#[test]
fn authored_packs_parse_and_keep_their_assets() {
    let root = root();
    let mut count = 0;
    for entry in std::fs::read_dir(&root).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.join("style.ogt").is_file() {
            continue;
        }
        count += 1;
        let text = std::fs::read_to_string(dir.join("style.ogt")).unwrap();
        let mut file = parse_style(&text).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        assert_eq!(file.version, 3, "{}", dir.display());
        let name = dir.file_name().unwrap().to_str().unwrap();
        file.resolve(&format!("{name}/style.ogt"), |id| root.join(id).exists());
        for asset in file.references() {
            let path = root.join(asset.strip_prefix("@styles/").unwrap_or(&asset));
            assert!(path.is_file(), "{name} references missing {asset}");
        }
        for event in og_ui::tokens::SoundEvent::ALL {
            for sound in file.feedback_sounds[event.index()].iter() {
                assert!(
                    root.join(sound.strip_prefix("@styles/").unwrap_or(sound))
                        .is_file(),
                    "{name}: {} references missing {sound}",
                    event.name()
                );
            }
        }
    }
    assert!(count > 0, "no authored packs discovered");
}

#[test]
fn a_styles_fonts_name_weights_italics_and_each_locales_companion() {
    let text = std::fs::read_to_string(root().join("scifi/style.ogt")).unwrap();
    let fonts = r#"fonts {
  body 300 "Light.ttf"
  body 600 "SemiBold.ttf"
  body italic "Italic.ttf"
  display 800 italic "BlackItalic.ttf"
  numeric "Mono.ttf"
  cjk "SC.ttf"
  cjk ja "JP.ttf"
  cjkDisplay ko 700 "KR-Bold.ttf"
}"#;
    let a = text.find("\nfonts {").expect("a fonts block") + 1;
    let b = a + text[a..].find("\n}").unwrap() + 2;
    let file = parse_style(&format!("{}{fonts}{}", &text[..a], &text[b..]))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(file.fonts[0].as_deref(), Some("Light.ttf"));
    assert_eq!(file.font_weights[0], 300);
    assert_eq!(file.fonts[1], None, "display names only an italic");
    assert_eq!(file.fonts[4].as_deref(), Some("SC.ttf"));
    let faces: Vec<_> = file
        .font_faces
        .iter()
        .map(|f| (f.role, f.cjk, f.locale, f.weight, f.italic, f.path.as_str()))
        .collect();
    assert_eq!(
        faces,
        [
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                600,
                false,
                "SemiBold.ttf"
            ),
            (
                FontRole::Body,
                false,
                CjkLocale::Hans,
                400,
                true,
                "Italic.ttf"
            ),
            (
                FontRole::Display,
                false,
                CjkLocale::Hans,
                800,
                true,
                "BlackItalic.ttf"
            ),
            (FontRole::Body, true, CjkLocale::Ja, 400, false, "JP.ttf"),
            (
                FontRole::Display,
                true,
                CjkLocale::Ko,
                700,
                false,
                "KR-Bold.ttf"
            ),
        ]
    );
    // Each companion with its forms, so the cook ships only the shown ones.
    assert_eq!(
        file.companions(),
        [
            (CjkLocale::Hans, "SC.ttf".to_string()),
            (CjkLocale::Ja, "JP.ttf".to_string()),
            (CjkLocale::Ko, "KR-Bold.ttf".to_string()),
        ]
    );
    assert!(file.references().contains(&"BlackItalic.ttf".to_string()));
    for bad in [
        r#"body 650 "x.ttf""#,
        r#"cjk fr "x.ttf""#,
        r#"body ja "x.ttf""#,
    ] {
        let f = format!("fonts {{\n  {bad}\n}}");
        let e = parse_style(&format!("{}{f}{}", &text[..a], &text[b..])).unwrap_err();
        assert!(e.contains("line"), "{bad}: {e}");
    }
}

/// A style's motion tokens reach the motion code: each built-in style's
/// named springs keep their damping ratio and response in `Theme::motion`.
#[test]
fn a_styles_motion_tokens_drive_the_motion_code() {
    for slug in ["cartoon", "pixel", "scifi"] {
        let path = format!(
            "{}/../../templates/styles/{slug}/style.ogt",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(&path).expect("a built-in style");
        let theme = parse_style(&text).expect("it parses").theme;
        let tokens = &theme.tokens;
        for (name, got) in [
            (og_ui::tokens::SpringName::Ui, theme.motion.ui),
            (og_ui::tokens::SpringName::Playful, theme.motion.playful),
        ] {
            let want = tokens.spring(name);
            assert_eq!(got.duration_ms, want.duration_ms, "{slug} {name:?}");
            let damping = want.to_spring().damping;
            assert!(
                (1.0 - got.bounce - damping).abs() < 1e-4 || got.bounce == -0.95,
                "{slug} {name:?}: bounce {} for damping {damping}",
                got.bounce
            );
        }
        assert_eq!(theme.motion.stagger_ms, tokens.motion.stagger_ms, "{slug}");
    }
}

use og_ui::dress::PartRef;
use og_ui::tokens::ElementClass;
#[test]
fn a_packs_selected_chip_slot_and_tab_share_one_grammar_and_its_wells_read_3_to_1() {
    // Wrong: a pack's selected chip inverting (the default's grammar), a
    // selected slot keeping its thin rim and recess, a tab no hairline; or
    // an unchecked box's rim the 1.3:1 `line`, lost on its panel.
    use og_ui::dress::{token_effects, token_strokes, well_rim};
    use og_ui::paint::{ColorRef, Paint};
    let accent = |s: &[og_ui::paint::Stroke]| {
        s.len() == 1
            && matches!(&s[0].paint, Paint::Solid(ColorRef::Token { name, .. }) if name == "primary")
    };
    let ui = dressed("");
    let pack = ui.theme();
    let hair = pack.tokens.stroke(og_ui::tokens::StrokeWidth::Hairline);
    for (component, part, class) in [
        ("Chip", "base", ElementClass::Chip),
        ("Slot", "frame", ElementClass::Slot),
        ("Tabs", "tab", ElementClass::Tab),
    ] {
        let p = PartRef::new(component, part, class).flag("selected");
        let s = token_strokes(&p, pack);
        assert!(
            accent(&s) && s[0].width == hair,
            "{component}: an accent hairline"
        );
        assert!(
            token_effects(&p, pack).is_empty(),
            "{component}: flat, no recess or shadow"
        );
        let def = token_strokes(&p, &Theme::default());
        let thin = Theme::default()
            .tokens
            .stroke(og_ui::tokens::StrokeWidth::Thin);
        match component {
            "Slot" => assert!(
                accent(&def) && def[0].width == thin,
                "the default's slot rim"
            ),
            _ => assert!(def.is_empty(), "{component}: the default's, no rim"),
        }
    }
    // The chip's ground is one step up and its words the accent's ink.
    let mut ui = dressed("");
    ui.begin(InputFrame::default(), Viewport::new(800.0, 600.0));
    ui.chip(
        "On",
        "All",
        ChipDesc {
            selected: true,
            ..ChipDesc::default()
        },
    );
    ui.end();
    let e = ui.frame().element("On").unwrap().clone();
    let fills = fills_in(&ui, Rect::new(e.x, e.y, e.width, e.height));
    assert!(
        fills.iter().any(|f| near(*f, pack.palette.control_strong)),
        "on controlStrong: {fills:?}"
    );
    assert!(
        !fills.iter().any(|f| near(*f, pack.palette.ink)),
        "not inverted"
    );
    // Every pack's unchecked well is found at 3:1 on its well and panel.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    for e in std::fs::read_dir(dir).unwrap() {
        let f = e.unwrap().path().join("style.ogt");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        let t = parse_style(&text).unwrap().theme(None, [0; 6]);
        let rim = well_rim(&t);
        for ground in [t.palette.panel, t.palette.panel_raised] {
            assert!(
                rim.contrast(ground.fade(1.0)) >= 3.0,
                "{}: {}",
                f.display(),
                rim.contrast(ground)
            );
        }
        let p = PartRef::new("Checkbox", "box", ElementClass::Field);
        assert!(
            matches!(&token_strokes(&p, &t)[..], [s] if matches!(s.paint, Paint::Solid(ColorRef::Literal(c)) if c == rim))
        );
    }
}

#[test]
fn every_packs_bag_cell_takes_its_slots_selected_and_tier_marks() {
    // Wrong: a pack's tier rims and selection dressed on `Slot.frame` only,
    // so the kit's bag (`Inventory.item[rarity=…][selected]`) draws none.
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/styles");
    let mut packs = 0;
    for e in std::fs::read_dir(dir).unwrap() {
        let f = e.unwrap().path().join("style.ogt");
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        packs += 1;
        let set = og_ui::dress::DressSet::new(parse_style(&text).unwrap().dressings);
        for tier in ["none", "common", "rare", "legendary"] {
            for sel in [false, true] {
                let of = |c, p| {
                    PartRef::new(c, p, ElementClass::Slot)
                        .attr("rarity", tier)
                        .flag_if(sel, "selected")
                };
                let slot = set.resolve(&of("Slot", "frame"));
                let bag = set.resolve(&of("Inventory", "item"));
                assert_eq!(
                    bag.draw,
                    slot.draw,
                    "{}: {tier} selected={sel}",
                    f.display()
                );
            }
        }
    }
    assert!(packs >= 12, "{packs} packs");
}

/// What the tools that edit a style write (`og style art`, `og style pack`:
/// read as entries, written back as version 3) reads as the pack did.
#[test]
fn every_pack_writes_back_as_the_same_style() {
    for entry in std::fs::read_dir(root()).unwrap() {
        let path = entry.unwrap().path().join("style.ogt");
        if !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let written = write_style_doc(&read_style_doc(&text).unwrap()).unwrap();
        let (a, b) = (
            parse_style(&text).unwrap(),
            parse_style(&written).unwrap_or_else(|e| panic!("{}: {e}\n{written}", path.display())),
        );
        let sel = |f: &StyleFile| {
            f.dressings
                .iter()
                .map(|d| (d.selector.clone(), d.frame.clone(), d.draw.clone()))
                .collect::<Vec<_>>()
        };
        assert!(
            sel(&a) == sel(&b),
            "{}: the dressings differ",
            path.display()
        );
        assert!(a.theme == b.theme, "{}: the theme differs", path.display());
    }
}
