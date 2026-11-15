//! The default style is the design language's: `src/theme.rs` is exactly
//! what the generator writes from `packages/design/tokens/*.css` (a hand
//! edit, a stale file or a token changed without regenerating fail here),
//! its glass tints are og-render's presets, and its CSS parses and puts
//! glass on floating chrome only.

#[path = "../scripts/theme_codegen.rs"]
mod theme_codegen;

use og_ui::{Srgba, dress::PartRef};
use std::path::PathBuf;

fn tokens() -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    theme_codegen::load(&root.join("../../packages/design/tokens")).expect("the tokens load")
}

#[test]
fn theme_rs_is_generated_from_the_design_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = tokens();
    let committed = std::fs::read_to_string(root.join("src/theme.rs")).expect("theme.rs");
    let generated = theme_codegen::generate(&tokens).expect("the tokens generate");
    if committed != generated {
        let line = committed
            .lines()
            .zip(generated.lines())
            .position(|(a, b)| a != b)
            .unwrap_or(committed.lines().count().min(generated.lines().count()));
        panic!(
            "src/theme.rs differs from the generator's output at line {}:\n  committed: {:?}\n  generated: {:?}\nrun crates/og-ui/scripts/gen-theme.sh (never edit theme.rs by hand)",
            line + 1,
            committed.lines().nth(line),
            generated.lines().nth(line)
        );
    }
    // The theme the toolkit starts from is the generated one.
    assert_eq!(og_ui::Theme::dark().palette, og_ui::theme::DARK);
    assert_eq!(og_ui::Theme::light().palette, og_ui::theme::LIGHT);
    assert_eq!(og_ui::Theme::dark().radius, og_ui::theme::RADIUS);
}

#[test]
fn tokens_json_is_generated_from_the_design_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let committed = std::fs::read_to_string(root.join("../../packages/design/tokens.json"))
        .expect("tokens.json");
    let docs =
        theme_codegen::docs(&root.join("../../packages/design/tokens")).expect("the notes load");
    assert_eq!(
        committed,
        theme_codegen::json(&tokens(), &docs).expect("the tokens write as JSON"),
        "run crates/og-ui/scripts/gen-theme.sh (never edit tokens.json by hand)"
    );
    // origin-agent reads tokens.json: the groups keep the notes it had
    // before the tokens became CSS.
    let tree: serde_json::Value = serde_json::from_str(&committed).expect("tokens.json is JSON");
    for group in ["state", "controls", "derived", "glassContrast"] {
        assert!(
            tree[group]["_doc"].is_string(),
            "tokens.json: {group} has no _doc"
        );
    }
    assert!(
        tree["motion"]["timing"]["_doc"].is_string(),
        "tokens.json: motion.timing has no _doc"
    );
}

#[test]
fn default_css_is_generated_from_the_design_tokens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tokens = tokens();
    let committed = std::fs::read_to_string(root.join("default.css")).expect("default.css");
    let generated = theme_codegen::generate_css(&tokens).expect("the tokens generate CSS");
    assert_eq!(
        committed, generated,
        "run crates/og-ui/scripts/gen-theme.sh (never edit default.css by hand)"
    );
    // The generated public selectors must be accepted by the same parser
    // packs use, with host facts before ::part and state after it.
    use og_ui::doc::selectors::{Selector as CssSelector, Simple};
    let comments = regex_lite::Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let sheet = comments.replace_all(&generated, "");
    let rules = regex_lite::Regex::new(r"([^{}]+)\{").unwrap();
    for rule in rules.captures_iter(&sheet) {
        let header = rule[1].trim();
        if header.starts_with('@') || header.starts_with(":root") {
            continue;
        }
        for selector in CssSelector::parse_list(header).expect("valid CSS selector") {
            // A role rule is made only of `:where()`, so it has no specificity
            // and any pack or document rule wins over it.
            if matches!(&selector.compounds[0].0[0], Simple::Where(_)) {
                assert!(
                    selector
                        .compounds
                        .iter()
                        .all(|c| matches!(c.0.as_slice(), [Simple::Where(_)])),
                    "{header}"
                );
                continue;
            }
            assert_eq!(selector.compounds.len(), 1, "{header}");
            let simple = &selector.compounds[0].0;
            assert!(matches!(&simple[0], Simple::Type(tag) if tag == &tag.to_lowercase()));
            let Some(part) = simple.iter().position(|s| matches!(s, Simple::Part(_))) else {
                continue;
            };
            assert!(
                simple[1..part]
                    .iter()
                    .all(|s| matches!(s, Simple::Class(_) | Simple::Attribute { .. })),
                "{header}"
            );
            assert!(
                simple[part + 1..]
                    .iter()
                    .all(|s| matches!(s, Simple::State(_))),
                "{header}"
            );
        }
    }
}

/// The tokens' material tints are the ones og-render's glass draws: a
/// tint changed on one side only would hold the language's contrast pairs
/// in the test and not on the screen.
#[test]
fn the_tokens_glass_tints_are_og_renders_presets() {
    use og_draw::{UiGlassStyle, UiGlassWeight};
    let t = tokens();
    for (dark, scheme) in [(false, &t["color"]), (true, &t["dark"]["color"])] {
        for (weight, key) in [
            (UiGlassWeight::Thin, "material-thin"),
            (UiGlassWeight::Regular, "material-regular"),
            (UiGlassWeight::Thick, "material-thick"),
        ] {
            let s = scheme[key]
                .as_str()
                .or(t["color"][key].as_str())
                .expect(key);
            let inner = s.trim_start_matches("rgb(").trim_end_matches(')');
            let (rgb, a) = inner.split_once('/').expect("an alpha");
            let c: Vec<u8> = rgb
                .split_whitespace()
                .map(|p| p.parse().expect("a channel"))
                .collect();
            let want = Srgba {
                rgb: [c[0], c[1], c[2]],
                alpha: a.trim().parse().expect("an alpha"),
            }
            .linear();
            let (_, got) = UiGlassStyle {
                weight,
                dark,
                ..UiGlassStyle::default()
            }
            .backdrop();
            for k in 0..4 {
                assert!(
                    (want[k] - got[k]).abs() < 1e-4,
                    "{key} ({}): tokens {want:?}, og-render {got:?}",
                    if dark { "dark" } else { "light" }
                );
            }
        }
    }
}

/// Floating chrome is glass and nothing else is: dialogs, sheets, menus,
/// popovers and tooltips thick, HUD capsules regular; cards and controls
/// carry no backdrop, and controls no border.
#[test]
fn the_default_css_puts_glass_on_floating_chrome_only() {
    use og_ui::paint::Effect;
    let ui = og_ui::Ui::new(Default::default());
    let glass = |sel: &str| -> Option<og_draw::UiGlassWeight> {
        ui.resolve_part(&PartRef::parse(sel, None).unwrap(), [100.0, 40.0])
            .unwrap()
            .effects
            .iter()
            .find_map(|e| match e {
                Effect::Backdrop { glass, .. } => glass.weight,
                _ => None,
            })
    };
    use og_draw::UiGlassWeight::{Regular, Thick};
    for s in ["panel.glass::part(frame)", "tooltip::part(frame)"] {
        assert_eq!(glass(s), Some(Thick), "{s}");
    }
    assert_eq!(glass("hud::part(frame)"), Some(Regular));
    for s in [
        "panel::part(frame)",
        "button::part(base)",
        "input::part(frame)",
        "slot::part(frame)",
    ] {
        assert_eq!(glass(s), None, "{s}");
    }
    for s in [
        "button::part(base)",
        "input::part(frame)",
        "slot::part(frame)",
        "input[type=checkbox][switch]::part(row)",
        "input[type=checkbox]::part(row)",
        "input[type=radio]::part(option)",
        "input[type=radio]::part(card)",
    ] {
        let x = ui
            .resolve_part(&PartRef::parse(s, None).unwrap(), [100.0, 40.0])
            .unwrap();
        assert!(
            x.strokes.is_empty()
                && x.borders
                    .as_ref()
                    .is_none_or(|b| b.borders.iter().all(Option::is_none)),
            "{s} is borderless"
        );
    }
    // A row's lit top edge (the tokens' sheen on a button) read as an outline.
    for s in [
        "input[type=checkbox][switch]::part(row)",
        "input[type=checkbox]::part(row)",
        "input[type=radio]::part(option)",
    ] {
        let x = ui
            .resolve_part(&PartRef::parse(s, None).unwrap(), [100.0, 40.0])
            .unwrap();
        assert!(x.effects.is_empty(), "{s} has no lit edge");
    }
}

/// The og-ui unit (docs/ui.md §6.1) zooms the whole UI: a 1080p window
/// lays out on 1280×720 units and draws them at 1.5 CSS px, so a fixed
/// 300 px box a game writes is 450 CSS px wide, body text draws at 21 px
/// and the caption step at 19.5, a pointer at the CSS point lands on the
/// box and the probe reports CSS px. 720p and touch keep CSS px, a pin
/// wins, and the text scale multiplies over the unit.
#[test]
fn the_unit_zooms_the_whole_ui_on_a_mouse_surface() {
    use crate::support::{Doc, test_font, ui_with_font};
    use og_ui::{Cursor, DrawCmd, InputFrame, Theme, Viewport};
    // The button's pointer cursor is what the frame answers while the
    // pointer is over it.
    const GO: &str = r#"<meta name="component" content="t">
<column><button id="Go">Go</button></column>
<style>#Go { width: 300px; cursor: pointer; }</style>"#;
    let frame = |vp: Viewport, configure: &dyn Fn(&mut Theme), input: InputFrame| {
        let mut ui = ui_with_font(test_font());
        let mut t = *ui.theme();
        configure(&mut t);
        ui.set_theme(t);
        let mut d = Doc::with_ui(ui, GO);
        d.viewport = vp;
        for _ in 0..2 {
            d.step(input.clone());
        }
        let ui = d.ui;
        let hovered = ui.frame().cursor == Cursor::Pointer;
        let text = ui
            .frame()
            .items
            .iter()
            .find_map(|i| {
                let cmd = match &i.cmd {
                    DrawCmd::StyledText { text, .. } => &**text,
                    c => c,
                };
                match cmd {
                    DrawCmd::Text { size, bounds, .. } => Some((*size, bounds.x)),
                    _ => None,
                }
            })
            .expect("the label is drawn");
        (ui, hovered, text)
    };
    let body = Theme::dark().type_ramp.body.size;
    let hd = Viewport::new(1920.0, 1080.0);
    let (ui, _, (size, _)) = frame(hd, &|_| {}, InputFrame::default());
    let (f, t) = (ui.frame(), *ui.theme());
    assert_eq!(t.unit, 1.5);
    assert_eq!(
        (f.viewport.width, f.viewport.height, f.viewport.dpr),
        (1280.0, 720.0, 1.5)
    );
    assert_eq!(
        t.type_ramp.body.size, body,
        "lengths stay the style's, in units"
    );
    let units = ui.elements().iter().find(|e| e.id == "Go");
    assert_eq!(units.expect("the button").width, 300.0);
    assert_eq!(size, body * 1.5, "drawn at 1.5 CSS px a unit");
    assert_eq!(
        t.type_ramp.xs.size * 1.5,
        19.5,
        "xs aliases the 13 px caption, scaled to 1080p"
    );
    let css = ui.probe_elements();
    let go = css
        .iter()
        .find(|e| e.id == "Go")
        .expect("the probe's button");
    assert_eq!(go.width, 450.0, "the probe reports CSS px");
    // A pointer at the button's CSS centre is over it; one past its CSS
    // right edge (inside its width in units) is not.
    let (cx, cy) = go.center();
    assert!(frame(hd, &|_| {}, InputFrame::hover(cx, cy)).1);
    assert!(!frame(hd, &|_| {}, InputFrame::hover(go.x + 460.0, cy)).1);
    // An ultrawide 1080p is still 1.5 (its height), a 16:10 window by its
    // 16:9 height, 720p and below 1, 4K at most 2.
    let unit = |vp: Viewport| frame(vp, &|_| {}, InputFrame::default()).0.theme().unit;
    assert_eq!(unit(Viewport::new(2560.0, 1080.0)), 1.5);
    assert_eq!(unit(Viewport::new(1440.0, 900.0)), 1.125);
    assert_eq!(unit(Viewport::new(1280.0, 720.0)), 1.0);
    assert_eq!(unit(Viewport::new(3840.0, 2160.0)), 2.0);
    // Touch keeps CSS px; a pin wins.
    let touch = Viewport {
        coarse_pointer: true,
        ..hd
    };
    assert_eq!(frame(touch, &|_| {}, InputFrame::default()).2.0, body);
    assert_eq!(
        frame(hd, &|t| t.unit_pin = 1.0, InputFrame::default()).2.0,
        body
    );
    // The text scale multiplies over the unit.
    assert_eq!(
        frame(hd, &|t| t.text_scale = 1.5, InputFrame::default())
            .2
            .0,
        body * 2.25
    );
}

/// A zoomed frame's shapes inside a scroll view are clipped where the view
/// stands on the screen: every clip is the 1× frame's times the unit, from
/// the origin (a clip scaled about its own centre cut the rows' fills).
#[test]
fn a_zoomed_scroll_view_clips_where_it_stands() {
    use crate::support::{Doc, test_font, ui_with_font};
    use og_ui::{EmitDesc, InputFrame, Viewport};
    let rows: String = (0..8)
        .map(|i| format!(r#"<button id="B{i}">Row</button>"#))
        .collect();
    let text = format!(
        r#"<meta name="component" content="t">
<column><scroll id="S">{rows}</scroll></column>
<style>#S {{ width: 300px; height: 200px; }}</style>"#
    );
    let clips = |vp: Viewport| {
        let mut d = Doc::with_ui(ui_with_font(test_font()), &text);
        d.viewport = vp;
        for _ in 0..2 {
            d.step(InputFrame::default());
        }
        let ui = d.ui;
        let mut overlay = og_draw::Overlay::new(vp.width, vp.height, 0.0);
        ui.frame().emit(&EmitDesc::default(), &mut overlay);
        overlay
            .ui
            .shapes
            .iter()
            .map(|s| overlay.clips[s.clip.expect("clipped") as usize])
            .collect::<Vec<_>>()
    };
    let one = clips(Viewport::new(1280.0, 720.0));
    let zoomed = clips(Viewport::new(1920.0, 1080.0));
    assert!(!one.is_empty());
    let want: Vec<[f32; 4]> = one.iter().map(|c| c.map(|v| v * 1.5)).collect();
    assert_eq!(zoomed, want);
}

/// A scrim made stronger stays a scrim: the dark scrim reads CSS 0.42 over
/// mid grey (linear 0.687), the spotlight's 1.6 times it reads CSS 0.67,
/// linear about 0.89, and high contrast's 1.5 times stays see-through.
/// Scaling the linear alpha itself drew both opaque black.
#[test]
fn a_deeper_scrim_reads_deeper_and_stays_translucent() {
    let scrim = og_ui::Theme::dark().palette.scrim;
    assert!((scrim.alpha - 0.687).abs() < 0.01, "{scrim:?}");
    // CSS 0.42 × 1.6 = 0.672 over #808080: 128 × 0.328 = 42 → linear
    // 0.0232 against the grey's 0.2158, so 1 − 0.0232 / 0.2158 = 0.893.
    let spot = scrim.deepen(1.6).alpha;
    assert!((spot - 0.893).abs() < 0.01, "{spot}");
    assert!((scrim.deepen(1.0).alpha - scrim.alpha).abs() < 0.005);
    let high = og_ui::Theme::dark().to_high_contrast().palette.scrim.alpha;
    assert!(high > scrim.alpha && high < 0.95, "{high}");
}

#[test]
fn css_tokens_and_theme_writes_share_the_same_values() {
    use og_ui::*;
    let mut theme = Theme::dark();
    theme.unit_pin = 1.0;
    theme.space.md = 23.0;
    theme.palette.primary = Srgba {
        rgb: [17, 83, 149],
        alpha: 0.137,
    };
    theme.durations_ms.slow = 444.0;
    for bp in Breakpoint::ALL {
        theme.tokens.typography.steps[tokens::TypeRole::Body.index()][bp as usize].size = 21.0;
    }
    let mut ui = Ui::new(UiDesc {
        theme,
        ..Default::default()
    });
    ui.set_part_css(&format!(
        "{DEFAULT_CSS}\n:root {{ --radius-button: 9px; --type-button-size: 19px; --type-button-font: 'Missing Face', monospace; --motion-named-ui-duration: 0.25s; --state-hover-wash: 0.2; --control-md: 41px; }}"
    ))
    .unwrap();
    let mut rebuilt = Ui::new(UiDesc {
        theme: Theme {
            unit_pin: 1.0,
            ..Theme::dark()
        },
        ..Default::default()
    });
    rebuilt.inherit_part_css(&ui);
    let mut ui = rebuilt;
    ui.define_document(
        UiDocument::parse(
            "tokens.svelte",
            r#"
<meta name="component" content="tokens">
<column><column id="Box"></column></column>
<style>#Box { width: var(--type-body-size); height: var(--space-md); background: var(--primary); }</style>
"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "t"),
        "tokens",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for (width, height) in [(1280.0, 720.0), (390.0, 844.0)] {
        ui.begin(InputFrame::default(), Viewport::new(width, height));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let box_ = ui.element("Box").unwrap();
        assert_eq!((box_.width, box_.height), (21.0, 23.0));
        let fill = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .find(|(r, _)| r.width == 21.0)
            .unwrap()
            .1;
        assert_eq!(
            fill,
            theme.palette.primary.linear(),
            "CSS keeps fractional alpha, not an 8-bit hex conversion"
        );
        assert_eq!(ui.theme().radius.control, 9.0);
        assert_eq!(ui.type_step(tokens::TypeRole::Button).size, 19.0);
        assert_eq!(ui.type_font(tokens::TypeRole::Button), FontRole::Mono);
        assert_eq!(ui.type_step(tokens::TypeRole::Body).size, 21.0);
        // Motion, state and density read back, written from a theme or set
        // by a root rule: `--control-md` is the input's density's (mouse at
        // 1280, touch under 768).
        let t = ui.theme();
        assert_eq!(t.durations_ms.slow, 444.0);
        assert_eq!(t.motion.ui.duration_ms, 250.0);
        assert_eq!(t.state.hover_wash, 0.2);
        assert_eq!(ui.sizes().control_md, 41.0);
    }
}

/// Whatever seeds a style or a game sets, the palette derived from them
/// keeps the language's pairs in both schemes (packages/design/tokens/
/// contrast.css): text 4.5:1, focus, rims and team marks 3:1, text over
/// glass 4.5:1 over the brightest and darkest scenes. The seeds sweep
/// lightness, chroma and hue, with the neutral the seed itself or grey.
#[test]
fn any_seeds_keep_the_contrast_pairs() {
    let base = tokens();
    let srgb = |rgb: [u8; 3], alpha: f32| Srgba { rgb, alpha };
    let hex = |s: &str| -> Option<Srgba> {
        let n = u32::from_str_radix(s.strip_prefix('#')?, 16).ok()?;
        Some(Srgba::rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
    };
    let mut low = Vec::new();
    let mut seeds = Vec::new();
    for l in [
        0.02, 0.15, 0.3, 0.42, 0.5, 0.56, 0.6, 0.65, 0.72, 0.8, 0.9, 0.98,
    ] {
        for c in [0.0, 0.1, 0.2, 0.37] {
            for h in (0..360).step_by(30) {
                if c == 0.0 && h > 0 {
                    continue;
                }
                seeds.push(format!("oklch({l} {c} {h})"));
            }
        }
    }
    for seed in &seeds {
        for neutral in [seed.as_str(), "#808080"] {
            let mut t = base.clone();
            for path in ["/seed", "/dark/seed"] {
                let m = t
                    .pointer_mut(path)
                    .and_then(|v| v.as_object_mut())
                    .expect("seeds");
                for key in [
                    "primary", "accent", "danger", "warn", "success", "info", "team",
                ] {
                    m.insert(key.into(), seed.as_str().into());
                }
                m.insert("neutral".into(), neutral.into());
            }
            for dark in [false, true] {
                let scheme = if dark { "dark" } else { "light" };
                let color = |name: &str| -> Srgba {
                    hex(name).unwrap_or_else(|| {
                        let (rgb, a) = theme_codegen::role_colour(&t, dark, "color", name)
                            .unwrap_or_else(|e| panic!("{seed}: {name}: {e}"));
                        srgb(rgb, a)
                    })
                };
                let linear = |name: &str| -> Srgba {
                    let (rgb, a) = theme_codegen::role_colour(&t, dark, "linear", name)
                        .unwrap_or_else(|e| panic!("{seed}: {name}: {e}"));
                    srgb(rgb, a)
                };
                // A translucent ground over `under` (the ground `bg` when
                // unnamed), mixed as a browser mixes it.
                let ground = |name: &str, under: Option<&str>| -> Srgba {
                    let g = color(name);
                    if g.alpha < 1.0 {
                        g.over(color(under.unwrap_or("bg")))
                    } else {
                        g
                    }
                };
                let words = |v: &serde_json::Value| -> Vec<Vec<String>> {
                    v.as_array()
                        .expect("a list of pairs")
                        .iter()
                        .map(|p| {
                            p.as_array()
                                .expect("a pair")
                                .iter()
                                .map(|w| w.as_str().expect("a name").to_owned())
                                .collect()
                        })
                        .collect()
                };
                let mut check = |what: String, r: f32, bar: f32| {
                    if r < bar {
                        low.push(format!("{scheme} {seed} neutral {neutral}: {what} {r:.2}"));
                    }
                };
                for p in words(&t["contrast"]) {
                    if dark && p.iter().any(|w| w.starts_with("stage")) {
                        continue;
                    }
                    let r = color(&p[0]).contrast(ground(&p[1], p.get(2).map(String::as_str)));
                    check(p.join(" "), r, 4.5);
                }
                for p in words(&t["nonText"]) {
                    check(p.join(" "), color(&p[0]).contrast(ground(&p[1], None)), 3.0);
                }
                for p in words(&t["glassContrast"]["pairs"]) {
                    let mut g = color(&p[1]).over(color(&p[2]));
                    if let Some(fill) = p.get(3) {
                        g = linear(fill).over_linear(g);
                    }
                    check(p.join(" "), color(&p[0]).contrast(g), 4.5);
                }
            }
        }
    }
    let n = low.len();
    low.truncate(40);
    assert!(n == 0, "{n} pairs read low:\n{}", low.join("\n"));
}

/// Seeds given at run time (UiDesc.seeds) restyle the whole UI: the
/// palette the widgets read, the team colours and what a document's CSS
/// draws all follow a second set of seeds as they followed the first.
#[test]
fn seeds_set_at_run_time_restyle_the_whole_ui() {
    use og_ui::*;
    let linear = |hex: u32| {
        let c = Srgba::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8).linear();
        [c[0], c[1], c[2]]
    };
    let hue = |c: Srgba| {
        let [r, g, b] = c.rgb;
        // Red or blue the strongest channel.
        (r > g && r > b, b > r && b > g)
    };
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(
        UiDocument::parse(
            "seeds.svelte",
            r#"
<meta name="component" content="seeds">
<column><column id="Box"></column></column>
<style>#Box { width: 33px; height: 7px; background: var(--primary); }</style>
"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "t"),
        "seeds",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    let neutral = ui.theme().palette;
    let drawn = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(1280.0, 720.0));
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        let fill = ui
            .frame()
            .items
            .iter()
            .filter_map(|i| i.cmd.filled())
            .find(|(r, _)| r.width == 33.0)
            .unwrap()
            .1;
        (fill, *ui.theme())
    };
    for (primary, team, red) in [(0xc8102e, 0x2f7fef, true), (0x1f6fd0, 0xe8364f, false)] {
        let mut theme = Theme::dark();
        theme.seeds.0[0] = Some(linear(primary));
        theme.seeds.0[7] = Some(linear(team));
        ui.set_theme(theme);
        let (fill, t) = drawn(&mut ui);
        let p = t.palette;
        assert_eq!(
            fill,
            p.primary.linear(),
            "the document draws the seeded primary"
        );
        assert_eq!(hue(p.primary), (red, !red), "{:?}", p.primary);
        assert_ne!(p.primary_hover, neutral.primary_hover);
        assert!(p.on_primary.contrast(p.primary) >= 4.5);
        assert!(p.on_primary.contrast(p.primary_hover) >= 4.5);
        // The accent and the neutral follow the primary.
        assert_eq!(hue(p.highlight), (red, !red), "{:?}", p.highlight);
        assert_ne!(p.panel, neutral.panel);
        assert!(p.ink.contrast(p.panel_raised) >= 4.5);
        assert_eq!(
            hue(t.tokens.colors.team[0].color),
            (!red, red),
            "{:?}",
            t.tokens.colors.team[0].color
        );
    }
}
