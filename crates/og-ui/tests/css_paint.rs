//! CSS paint in documents (docs/ui.md, CSS paint): gradient backgrounds
//! (linear, radial, conic, repeating), `mask-image`, `clip-path` shapes,
//! `text-shadow`, `filter` chains (a drop shadow that follows a clipped
//! star's alpha, colour functions in order, opacity and blur), and
//! `backdrop-filter` colour functions over stripes, and image layers
//! (`url()` of texture assets) sized, placed and repeated: 16 px tiles cut
//! by a radius, a cover photo under a badge 6 px from its corner, a
//! contained icon over a gradient, and tiles spaced edge to edge.
//! The frames match `references/vulkan/css-paint-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: a gradient's angle read in maths' convention or not
//! lengthened to the corners of a wide box, a clip's percentages read
//! against the wrong box (or the clip drawn uncut), a mask that fades the
//! wrong way, or a retained element whose resolved paint rebuilds every
//! frame (idle must build nothing).

use crate::support;

use og_ui::*;
use support::*;

#[test]
fn image_urls_resolve_files_not_atlas_names_or_handles() {
    use og_draw::UiPaint;
    let mut handles = Handles::default();
    let mut ui = ui_with_font(test_font_metrics());
    let mut textures = Vec::new();
    for name in ["coin.svg", "7", "atlas"] {
        let texture = handles.texture();
        ui.define_texture(&format!("ui/{name}"), texture);
        textures.push(texture);
    }
    ui.set_icons(IconAtlas {
        texture: Some(textures[2]),
        columns: 1,
        rows: 1,
        names: vec!["coin.svg".into()],
        ..Default::default()
    });
    ui.define_document(
        UiDocument::parse(
            "ui/urls.svelte",
            r#"
<meta name="component" content="urls" />
<section><div id="file"></div><div id="missing"></div><div id="number"></div></section>
<style>
div { width: 8px; height: 8px; }
#file { background-image: url(coin.svg); }
#missing { background-image: url(missing/coin.svg); }
#number { background-image: url(7); }
</style>"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "urls"),
        "urls",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(100.0, 100.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    let images: Vec<_> = ui
        .frame()
        .items
        .iter()
        .filter_map(|item| match &item.cmd {
            DrawCmd::Shape { shape, .. } => match &shape.paint {
                UiPaint::Image(image) => Some(image.texture),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(images, textures[..2]);
    assert!(
        ui.frame()
            .warnings
            .iter()
            .any(|w| w.contains("missing/coin.svg"))
    );
    assert!(!ui.frame().warnings.iter().any(|w| w.contains("url(7)")));
}

#[test]
fn image_sets_reselect_on_density_changes() {
    use og_draw::UiPaint;
    let mut handles = Handles::default();
    let mut ui = ui_with_font(test_font_metrics());
    let mut textures = Vec::new();
    for name in ["low.png", "high.png"] {
        let texture = handles.texture();
        ui.define_texture(&format!("ui/{name}"), texture);
        textures.push(texture);
    }
    ui.define_document(
        UiDocument::parse(
            "ui/set.svelte",
            r#"
<meta name="component" content="set" />
<div id="image"></div>
<style>
#image { width: 32px; height: 32px;
  background-image: image-set("low.png" calc(2dppx - 96dpi),
    url(high.png) clamp(1x, calc(480dpi / 2), 2dppx));
  background-repeat: no-repeat; }
</style>"#,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "set"),
        "set",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    for (dpr, chosen) in [(1.0, 0), (1.5, 1), (3.0, 1), (0.75, 0)] {
        let mut viewport = Viewport::new(100.0, 100.0);
        viewport.dpr = dpr;
        ui.begin(InputFrame::default(), viewport);
        ui.build_retained_documents(&bind::NoEnv).unwrap();
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let images: Vec<_> = ui
            .frame()
            .items
            .iter()
            .filter_map(|item| match &item.cmd {
                DrawCmd::Shape { shape, .. } => match &shape.paint {
                    UiPaint::Image(image) => Some((image.texture, image.density)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(
            images,
            [(textures[chosen], (chosen + 1) as f32)],
            "DPR {dpr}"
        );
    }
}

#[test]
fn a_css_filter_function_list_filters_an_svg_shape() {
    // The `filter` property a rule sets on an SVG shape is the one its
    // attribute presents: a function list draws as a box's does.
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(
        UiDocument::parse(
            "ui/arc.svelte",
            r##"<meta name="component" content="arc" />
<svg viewBox="0 0 100 100" style="width: 100px; height: 100px"><path id="Arc" d="M10 10 L90 90" stroke="#ff0000" stroke-width="8"></path></svg>
<style>#Arc { filter: drop-shadow(0 0 6px #0000ff); }</style>"##,
        )
        .unwrap(),
    );
    ui.mount_document(
        reactive::SourceId::new("test", "arc"),
        "arc",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    ui.begin(InputFrame::default(), Viewport::new(200.0, 200.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let items = &ui.frame().items;
    let shadow = items
        .iter()
        .position(|i| {
            matches!(&i.cmd, DrawCmd::BeginLayer { material, params, .. }
                if *material == og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::DropShadow)
                    && params[1] == [0.0, 0.0, 1.0, 1.0])
        })
        .expect("a drop-shadow layer in the shadow's colour");
    let stroke = items
        .iter()
        .position(|i| matches!(&i.cmd, DrawCmd::Shape { shape, .. } if shape.paint == og_draw::UiPaint::Solid([1.0, 0.0, 0.0, 1.0])))
        .expect("the stroke");
    assert!(shadow < stroke, "the stroke draws inside the layer");
}

/// The solid fills (linear RGBA) and the stroked or filled shape colours
/// drawn inside `e`'s box, in draw order.
fn paints_in(ui: &Ui, e: &UiElement) -> Vec<[f32; 4]> {
    let inside = |r: &Rect| {
        r.x >= e.x - 0.5
            && r.y >= e.y - 0.5
            && r.x + r.width <= e.x + e.width + 0.5
            && r.y + r.height <= e.y + e.height + 0.5
    };
    ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Quad { rect, color, .. } if inside(rect) => Some(*color),
            DrawCmd::Shape { shape, .. } => match shape.paint {
                og_draw::UiPaint::Solid(c)
                    if inside(&Rect::new(
                        shape.center.x - shape.half.x,
                        shape.center.y - shape.half.y,
                        2.0 * shape.half.x,
                        2.0 * shape.half.y,
                    )) =>
                {
                    Some(c)
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn same(a: [f32; 4], b: Srgba) -> bool {
    let b = b.linear();
    (0..4).all(|k| (a[k] - b[k]).abs() < 2e-3)
}

/// The colour a check mark (an open polyline) is stroked in inside `e`.
fn tick_in(ui: &Ui, e: &UiElement) -> Srgba {
    ui.frame()
        .items
        .iter()
        .find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if matches!(shape.geometry, og_draw::UiGeometry::Polyline { .. })
                    && (e.x..e.x + e.width).contains(&shape.center.x)
                    && (e.y..e.y + e.height).contains(&shape.center.y) =>
            {
                match shape.paint {
                    og_draw::UiPaint::Solid(c) => Some(Srgba::from_linear(c)),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no check mark in {}", e.path))
}

/// The last opaque fill drawn under the check mark inside `e` (a switch's
/// knob), each command read through its element's map (a knob that has
/// travelled is translated).
fn under_tick(ui: &Ui, e: &UiElement) -> Srgba {
    let items = &ui.frame().items;
    let screen = |it: &DrawItem, p: og_core::glam::Vec2| {
        let (x, y) = it.transform.apply(p.x, p.y);
        og_core::glam::Vec2::new(x, y)
    };
    let (k, at) = items
        .iter()
        .enumerate()
        .find_map(|(k, it)| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if matches!(shape.geometry, og_draw::UiGeometry::Polyline { .. }) =>
            {
                let c = screen(it, shape.center);
                ((e.x..e.x + e.width).contains(&c.x) && (e.y..e.y + e.height).contains(&c.y))
                    .then_some((k, c))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no check mark in {}", e.path));
    items[..k]
        .iter()
        .rev()
        .find_map(|it| {
            let (x, y) = it.transform.invert()?.apply(at.x, at.y);
            let local = og_core::glam::Vec2::new(x, y);
            match &it.cmd {
                DrawCmd::Quad { rect, color, .. }
                    if color[3] > 0.99
                        && (rect.x..rect.x + rect.width).contains(&local.x)
                        && (rect.y..rect.y + rect.height).contains(&local.y) =>
                {
                    Some(Srgba::from_linear(*color))
                }
                DrawCmd::Shape { shape, .. }
                    if shape.mode == og_draw::UiMode::Fill
                        && (local - shape.center).abs().cmple(shape.half).all() =>
                {
                    match shape.paint {
                        og_draw::UiPaint::Solid(c) if c[3] > 0.99 => Some(Srgba::from_linear(c)),
                        _ => None,
                    }
                }
                _ => None,
            }
        })
        .expect("a fill under the check mark")
}

/// CSS `accent-color` inks the built-in controls (CSS UI 4 §6.1): a checked
/// box, a switch that is on, a chosen radio, a range's filled track and a
/// progress bar's fill take it, set on the control (in `<style>` or its
/// `style` attribute) or inherited from an ancestor, while a control with
/// none keeps the style's primary. What the control draws on its accent
/// (the check mark) is `contrast-color()` of it, so it reads 4.5:1 on a
/// light accent and on a dark one in either scheme.
///
/// Ways it could fail: the property refused inline (it was: inline
/// declarations missed the inherited text properties), one control kind
/// not reading it, the accent leaking to a sibling, or the mark keeping
/// the scheme's on-primary ink (a dark-scheme tick, near black, on a navy
/// accent read 1.2:1).
#[test]
fn accent_color_inks_the_built_in_controls_and_what_they_draw_on_it_reads() {
    let gold = Srgba::rgb(0xff, 0xd6, 0x0a);
    let navy = Srgba::rgb(0x1a, 0x23, 0x7e);
    let text = r#"<meta name="component" content="t">
<column>
  <input type="checkbox" id="Gold" checked />
  <input type="checkbox" id="Navy" checked style="accent-color: #1a237e" />
  <input type="checkbox" id="Plain" checked />
  <input type="checkbox" switch id="GoldSwitch" checked style="accent-color: #ffd60a" />
  <div class="navy">
    <input type="checkbox" switch id="Switch" checked />
    <label><input type="radio" id="Radio" value="a" bind:group={$pick} /> A</label>
    <input type="range" id="Range" min="0" max="10" value="5" />
  </div>
  <progress id="Progress" value="0.5" style="accent-color: light-dark(#1a237e, #ffd60a)"></progress>
</column>
<style>
#Gold { accent-color: #ffd60a; }
.navy { accent-color: #1a237e; }
</style>"#;
    for scheme in [Scheme::Dark, Scheme::Light] {
        // Built dark first, then switched: the accent follows the scheme.
        // The steps let the controls' colour transitions settle (after
        // four, the light knob was still on its way from the dark one).
        let mut d = Doc::new(text, &[("pick", serde_json::json!("a"))]);
        d.ui.set_theme(Theme::new(scheme));
        for _ in 0..120 {
            d.step(InputFrame::default());
        }
        let primary = Theme::new(scheme).palette.primary;
        let ui = &d.ui;
        for (id, accent) in [
            ("Gold", gold),
            ("Navy", navy),
            ("Switch", navy),
            ("Range", navy),
        ] {
            let paints = paints_in(ui, &d.get(id));
            assert!(
                paints.iter().any(|&c| same(c, accent)),
                "{scheme:?} {id} draws no {accent:?}: {paints:?}"
            );
        }
        // The progress bar's accent follows the scheme (light-dark()).
        let bar = if scheme == Scheme::Dark { gold } else { navy };
        assert!(
            paints_in(ui, &d.get("Progress"))
                .iter()
                .any(|&c| same(c, bar))
        );
        assert!(
            paints_in(ui, &d.get("Radio"))
                .iter()
                .any(|&c| same(c, navy))
        );
        let plain = paints_in(ui, &d.get("Plain"));
        assert!(
            plain.iter().any(|&c| same(c, primary)),
            "{scheme:?} {plain:?}"
        );
        assert!(!plain.iter().any(|&c| same(c, gold) || same(c, navy)));
        // A switch's knob, which carries a tick in the accent, reads on
        // the accent (a white knob on gold in the light scheme did not).
        let knob = under_tick(ui, &d.get("GoldSwitch")).contrast(gold);
        assert!(
            knob >= 3.0,
            "{scheme:?}: the gold switch's knob reads {knob:.2}:1"
        );
        for (id, accent) in [("Gold", gold), ("Navy", navy)] {
            let tick = tick_in(ui, &d.get(id));
            let ratio = tick.contrast(accent);
            assert!(
                ratio >= 4.5,
                "{scheme:?} {id}'s mark {tick:?} reads {ratio:.2}:1"
            );
        }
    }
}

/// CSS `caret-color` paints a focused field's caret: its own colour, one it
/// inherits, `currentColor` as the field's text colour, and `auto` the
/// style's primary.
#[test]
fn caret_color_paints_a_focused_fields_caret() {
    let text = r#"<meta name="component" content="t">
<column>
  <input id="Own" value="abc" style="caret-color: #00ff00" />
  <div style="caret-color: #ff00ff"><input id="Inherited" value="abc" /></div>
  <input id="Current" value="abc" style="color: #ff8800; caret-color: currentColor" />
  <input id="Auto" value="abc" />
</column>"#;
    let primary = Theme::dark().palette.primary;
    for (id, want) in [
        ("Own", Srgba::rgb(0, 255, 0)),
        ("Inherited", Srgba::rgb(255, 0, 255)),
        ("Current", Srgba::rgb(255, 0x88, 0)),
        ("Auto", primary),
    ] {
        let mut d = Doc::with_ui(ui_with_font(test_font()), text);
        d.focus = Some(d.get(id).path.clone());
        for _ in 0..4 {
            d.step(InputFrame::default());
        }
        let e = d.get(id);
        // The caret: a thin filled bar inside the field.
        let caret =
            d.ui.frame()
                .items
                .iter()
                .find_map(|it| match &it.cmd {
                    DrawCmd::Shape { shape, .. }
                        if shape.mode == og_draw::UiMode::Fill
                            && shape.half.x <= 1.0
                            && shape.half.y > 4.0
                            && (e.x..e.x + e.width).contains(&shape.center.x)
                            && (e.y..e.y + e.height).contains(&shape.center.y) =>
                    {
                        match shape.paint {
                            og_draw::UiPaint::Solid(c) => Some(c),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{id}: no caret drawn"));
        assert!(same(caret, want), "{id}: caret {caret:?}, not {want:?}");
    }
}

/// CSS Color 5's `contrast-color()` and `color-mix()` over any number of
/// colours reach what a document draws: a mix of red, lime and blue is
/// their average (85 85 85; a pairwise halving would draw 64 64 128), a
/// mix without a method is in OKLab, and `contrast-color()` of a role is
/// resolved where it is drawn, so the text on a panel turns black when
/// the scheme turns light.
#[test]
fn contrast_color_and_variadic_color_mix_reach_the_frame() {
    let text = r#"<meta name="component" content="t">
<column>
  <div id="Three"></div>
  <div id="Two"></div>
  <div id="Oklab"></div>
  <div id="Panel"><p>Ink</p></div>
</column>
<style>
#Three, #Two, #Oklab { width: 20px; height: 20px; }
#Three { background: color-mix(in srgb, red, lime, blue); }
#Two { background: color-mix(in srgb, red 30%, blue); }
#Oklab { background: color-mix(red, blue); }
#Panel { background: var(--panel); }
#Panel p { color: contrast-color(var(--panel)); }
</style>"#;
    let mut d = Doc::with_font(text, &[], test_font());
    let fill = |d: &Doc, id: &str| -> Srgba {
        let e = d.get(id);
        let c = paints_in(&d.ui, &e)[0];
        Srgba::from_linear(c)
    };
    assert_eq!(fill(&d, "Three").rgb, [85, 85, 85]);
    assert_eq!(fill(&d, "Two").rgb, [77, 0, 179]);
    let oklab = css_color::parse_str("color-mix(in oklab, red, blue)")
        .unwrap()
        .literal()
        .unwrap();
    assert_eq!(fill(&d, "Oklab").rgb, oklab.rgb);
    let ink = |d: &Doc| {
        d.ui.frame()
            .items
            .iter()
            .find_map(|it| match it.cmd.unstyled() {
                DrawCmd::Text { text, color, .. } if text == "Ink" => {
                    Some(Srgba::from_linear(*color))
                }
                _ => None,
            })
            .expect("the panel's text is drawn")
    };
    assert_eq!(ink(&d), Srgba::WHITE, "on the dark panel");
    d.ui.set_theme(Theme::light());
    for _ in 0..3 {
        d.step(InputFrame::default());
    }
    assert_eq!(ink(&d), Srgba::BLACK, "on the light panel");
}

/// The effects (`None`: drawn plain) and the frame index of the text
/// command that draws `text`.
fn text_effects(ui: &Ui, text: &str) -> (usize, Option<og_draw::UiTextEffects>) {
    ui.frame()
        .items
        .iter()
        .enumerate()
        .find_map(|(k, it)| match &it.cmd {
            DrawCmd::Text { text: t, .. } if t == text => Some((k, None)),
            DrawCmd::StyledText { text: t, effects } => match &**t {
                DrawCmd::Text { text: t, .. } if t == text => Some((k, Some((**effects).clone()))),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or_else(|| panic!("no text {text}"))
}

/// Text drawn straight over the scene takes the style's tag treatment
/// when nothing under it holds it 4.5:1 over a black and a white scene:
/// the default's `--tag-halo`, eight hard copies in the contrast-color()
/// of the text's own ink (black round pale or gold ink, white round
/// navy). An opaque panel holds its text, which stays plain; a 30 % black
/// plate does not (pale ink on it over white reads 2.3:1). A text's own
/// soft shadow, which outlines nothing, keeps its place and gets the halo
/// beside it. A pack that sets `--tag-fill` gives such text a plate.
///
/// Ways it could fail: every text outlined (a panel's ground not read),
/// the halo in the style's ink-inverse instead of the text's own
/// contrast (navy ink outlined in near-black read 1.4:1), a text with a
/// shadow of its own left bare, or the plate drawn over the glyphs.
#[test]
fn text_over_the_scene_takes_the_tag_treatment() {
    let text = r#"<meta name="component" content="t">
<column>
  <p id="Bare">Bare</p>
  <div style="background: #1c1c1e"><p>Panel</p></div>
  <div style="background: rgb(0 0 0 / 30%)"><p>Thin</p></div>
  <p style="text-shadow: 0 1px 2px #000000">Soft</p>
  <p style="color: #ffd60a">Gold</p>
  <p style="color: #1a237e">Navy</p>
  <div style="background: #1c1c1e; width: 40px; height: 40px"><p style="position: absolute; top: 48px">Under</p></div>
</column>"#;
    let d = Doc::with_font(text, &[], test_font());
    let black = [0.0, 0.0, 0.0, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];
    for (t, color) in [
        ("Bare", black),
        ("Thin", black),
        ("Gold", black),
        ("Navy", white),
        // Placed below its dark box: over the scene, not over the box.
        ("Under", black),
    ] {
        let fx = text_effects(&d.ui, t)
            .1
            .unwrap_or_else(|| panic!("{t} is drawn bare"));
        assert_eq!(fx.shadows.len(), 8, "{t}: {:?}", fx.shadows);
        assert!(
            fx.shadows.iter().all(|s| s.color == color && s.blur == 0.0),
            "{t}: {:?}",
            fx.shadows
        );
    }
    assert!(
        text_effects(&d.ui, "Panel")
            .1
            .is_none_or(|fx| fx.shadows.is_empty())
    );
    let soft = text_effects(&d.ui, "Soft").1.unwrap();
    assert_eq!(soft.shadows.len(), 9, "its own first, then the halo");
    assert_eq!(soft.shadows[0].blur, 2.0);

    let mut ui = ui_with_font(test_font());
    let pack = og_ui::style_pack::StylePack::parse(
        "styles/test",
        vec![(
            "styles/test/main.css".into(),
            ":root { --tag-halo: none; --tag-fill: #102030; --tag-pad: 2px 4px; --tag-radius: 3px; }".into(),
        )],
    );
    ui.set_style_pack(&pack.unwrap()).unwrap();
    let d = Doc::with_ui(ui, text);
    let (k, fx) = text_effects(&d.ui, "Bare");
    assert!(fx.is_none_or(|fx| fx.shadows.is_empty()));
    let plate = Srgba::rgb(0x10, 0x20, 0x30).linear();
    let DrawCmd::Quad {
        rect,
        radius,
        color,
        ..
    } = &d.ui.frame().items[k - 1].cmd
    else {
        panic!(
            "no plate under the text: {:?}",
            d.ui.frame().items[k - 1].cmd
        );
    };
    assert!(
        same(*color, Srgba::from_linear(plate)) && *radius == 3.0,
        "{color:?} {radius}"
    );
    let bare = d.get("Bare");
    assert!(
        rect.x < bare.x && rect.width > bare.width,
        "{rect:?} {bare:?}"
    );

    // An image only in a plate's corner (a filigree) leaves its text on
    // the thin fill, over the scene: it takes the halo. The same image
    // tiled under the text is a ground its author drew, and stays bare.
    let mut ui = ui_with_font(test_font());
    ui.define_texture("ui/corner.png", Handles::default().texture());
    let d = Doc::with_ui(
        ui,
        r#"<meta name="component" content="t">
<column>
  <div style="padding: 16px; background: url(corner.png) left top / 8px 8px no-repeat, rgb(0 0 0 / 30%)"><p>Cornered</p></div>
  <div style="padding: 16px; background: url(corner.png) left top / 8px 8px repeat, rgb(0 0 0 / 30%)"><p>Tiled</p></div>
</column>"#,
    );
    let fx = text_effects(&d.ui, "Cornered")
        .1
        .unwrap_or_else(|| panic!("Cornered is drawn bare"));
    assert_eq!(fx.shadows.len(), 8, "{:?}", fx.shadows);
    assert!(
        text_effects(&d.ui, "Tiled")
            .1
            .is_none_or(|fx| fx.shadows.is_empty())
    );
}

/// The colour `id` fills itself with: its one solid fill, as sRGB.
fn fill_of(d: &Doc, id: &str) -> Srgba {
    let e = d.get(id);
    match paints_in(&d.ui, &e).as_slice() {
        [c, ..] => Srgba::from_linear(*c),
        [] => panic!("{id} draws no fill"),
    }
}

/// `a` is `rgb` at `alpha`, each channel within one step of 255 (the
/// spec's values are rounded; an f32 conversion may fall either side).
fn about(a: Srgba, rgb: [u8; 3], alpha: f32) -> bool {
    a.rgb.iter().zip(rgb).all(|(a, b)| a.abs_diff(b) <= 1) && (a.alpha - alpha).abs() < 0.01
}

/// Every colour syntax of CSS Color 4 and 5 fills a box with the colour
/// the spec gives for it: the numbers are the specs' worked examples
/// (`lab(29.2345% 39.3825 20.0664)`, its `lch()`, `oklab()` and `oklch()`
/// forms are all #7d2329) or arithmetic on their formulas (`hwb(120 20%
/// 30%)` is green scaled by 1 − w − b plus w: 51 179 51), never values
/// read back from og-css. The system colours are the style's roles, the
/// "system" a game's UI runs in.
///
/// Ways it could fail: a percentage of `rgb()` read as a fraction of 1
/// (128 drawn as 1), a hue in the wrong unit, a lab or OKLab channel
/// scaled wrongly, a mix that is not premultiplied (red with transparent
/// would be half-transparent dark red), a relative colour that drops its
/// origin's alpha, `alpha()` refused, or a system colour drawn as the
/// missing-token magenta.
#[test]
fn every_colour_syntax_draws_the_colour_the_spec_gives() {
    let rows: &[(&str, [u8; 3], f32)] = &[
        ("#0f08", [0, 255, 0], 136.0 / 255.0),
        ("rgb(255 50% 0 / 0.5)", [255, 128, 0], 0.5),
        ("rgba(0, 128, 255, 25%)", [0, 128, 255], 0.25),
        ("hsl(120deg 100% 25%)", [0, 128, 0], 1.0),
        ("hsla(0.5turn, 100%, 50%, 0.5)", [0, 255, 255], 0.5),
        ("hwb(120 20% 30%)", [51, 179, 51], 1.0),
        ("lab(29.2345% 39.3825 20.0664)", [125, 35, 41], 1.0),
        ("lch(29.2345% 44.2 27)", [125, 35, 41], 1.0),
        ("oklab(40.101% 0.1147 0.0453)", [125, 35, 41], 1.0),
        ("oklch(40.101% 0.12332 21.555)", [125, 35, 41], 1.0),
        ("RebeccaPurple", [102, 51, 153], 1.0),
        ("color(srgb 0.5 0 1)", [128, 0, 255], 1.0),
        ("color(srgb-linear 0.05088 0 0)", [64, 0, 0], 1.0),
        ("color-mix(in srgb, red 25%, blue)", [64, 0, 191], 1.0),
        ("color-mix(in srgb, red, transparent)", [255, 0, 0], 0.5),
        ("hsl(from red calc(h + 120) s l)", [0, 255, 0], 1.0),
        ("rgb(from #204060 r g b / 50%)", [32, 64, 96], 0.5),
        ("oklch(from #7d2329 l c h)", [125, 35, 41], 1.0),
        ("alpha(from #ff8000 / 25%)", [255, 128, 0], 0.25),
        (
            "alpha(from rgb(0 0 255 / 50%) / calc(alpha / 2))",
            [0, 0, 255],
            0.25,
        ),
        ("alpha(from rgb(0 0 255 / 50%))", [0, 0, 255], 0.5),
    ];
    let p = Theme::dark().palette;
    let system = [
        ("Canvas", p.backdrop),
        ("CanvasText", p.ink),
        ("AccentColor", p.primary),
        ("AccentColorText", p.on_primary),
        ("ButtonFace", p.control),
        ("ButtonBorder", p.line_strong),
        ("Mark", p.primary_soft),
        ("Highlight", p.selection),
    ];
    let mut body = String::new();
    let mut style = String::new();
    let all = rows
        .iter()
        .map(|&(css, rgb, a)| (css, rgb, a))
        .chain(system.iter().map(|(css, c)| (*css, c.rgb, c.alpha)))
        .collect::<Vec<_>>();
    for (k, (css, ..)) in all.iter().enumerate() {
        body += &format!("<div id=\"K{k}\"></div>");
        style += &format!("#K{k} {{ background-color: {css}; }}\n");
    }
    let text = format!(
        "<meta name=\"component\" content=\"t\">\n<column>{body}</column>\n<style>\ndiv {{ width: 10px; height: 10px; }}\n{style}</style>"
    );
    let d = Doc::new(&text, &[]);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    for (k, (css, rgb, a)) in all.iter().enumerate() {
        let got = fill_of(&d, &format!("K{k}"));
        assert!(
            about(got, *rgb, *a),
            "{css}: drew {got:?}, not {rgb:?} at {a}"
        );
    }
}

/// The first drawn gradient's stop colours inside `id`, as sRGB.
fn gradient_stops(d: &Doc, id: &str) -> Vec<Srgba> {
    let e = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (e.x..=e.x + e.width).contains(&shape.center.x)
                    && (e.y..=e.y + e.height).contains(&shape.center.y) =>
            {
                match &shape.paint {
                    og_draw::UiPaint::GradientImage(g) => Some(
                        g.gradient
                            .stops
                            .iter()
                            .map(|s| Srgba::from_linear(s.color))
                            .collect(),
                    ),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} draws no gradient"))
}

/// The textures `id` draws, in draw order.
fn textures_in(d: &Doc, id: &str) -> Vec<og_draw::TextureHandle> {
    let e = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (e.x..=e.x + e.width).contains(&shape.center.x)
                    && (e.y..=e.y + e.height).contains(&shape.center.y) =>
            {
                match &shape.paint {
                    og_draw::UiPaint::Image(i) => Some(i.texture),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

/// `currentColor` is the element's own `color`, declared or inherited
/// (CSS Color 4 §6.4), wherever a colour is written: a border with no
/// colour, an outline, a gradient's stops, a background and `color`
/// itself (whose `currentColor` is the inherited colour). `light-dark()`
/// takes the element's used colour scheme (CSS Color 5 §2): `color-scheme:
/// light` or `dark` on it or an ancestor forces it, `light dark` and none
/// leave the style's, and `light-dark()` of two images picks between them
/// the same way.
///
/// Ways it could fail (each did before): `currentColor` in a background or
/// gradient resolved as the style's ink instead of the inherited red,
/// `light-dark()` ignoring `color-scheme` and taking the style's scheme,
/// or `light-dark(url(), url())` refused as a colour.
#[test]
fn current_color_and_light_dark_resolve_in_the_element() {
    let text = r#"<meta name="component" content="t">
<column>
  <div style="color: #ff0000">
    <div id="Border" style="border: 2px solid; outline: 1px solid; background: linear-gradient(currentColor, #0000ff)"></div>
    <div id="Own" style="color: #00ff00; background: currentColor"></div>
    <div id="Mixed" style="color: color-mix(in srgb, currentColor, #0000ff)"><div id="MixedKid" style="background: currentColor"></div></div>
  </div>
  <div style="color-scheme: light">
    <div id="Light" style="background: light-dark(#ff0000, #0000ff)"></div>
    <div id="LightImage" style="background: light-dark(url(a.png), url(b.png))"></div>
  </div>
  <div style="color-scheme: dark">
    <div id="Forced" style="background: light-dark(#ff0000, #0000ff)"></div>
    <div id="Both" style="color-scheme: light dark; background: light-dark(#ff0000, #0000ff)"></div>
  </div>
  <div id="Style" style="background: light-dark(#ff0000, #0000ff)"></div>
  <div id="StyleImage" style="background: light-dark(url(a.png), url(b.png))"></div>
</column>
<style>div div, #Style, #StyleImage { width: 10px; height: 10px; }</style>"#;
    let red = Srgba::rgb(255, 0, 0);
    let blue = Srgba::rgb(0, 0, 255);
    for scheme in [Scheme::Dark, Scheme::Light] {
        let mut ui = ui_with_font(test_font_metrics());
        let mut h = Handles::default();
        let (a, b) = (h.texture(), h.texture());
        ui.define_texture("ui/a.png", a);
        ui.define_texture("ui/b.png", b);
        ui.set_theme(Theme::new(scheme));
        let d = Doc::with_ui(ui, text);
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        let e = d.get("Border");
        let strokes: Vec<_> =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|it| match &it.cmd {
                    DrawCmd::Shape { shape, .. }
                        if matches!(shape.mode, og_draw::UiMode::Stroke(_))
                            && (shape.center.y - (e.y + e.height / 2.0)).abs() < 0.5 =>
                    {
                        match shape.paint {
                            og_draw::UiPaint::Solid(c) => Some(Srgba::from_linear(c)),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .collect();
        assert_eq!(
            strokes,
            [red, red],
            "{scheme:?}: the border and the outline"
        );
        assert_eq!(gradient_stops(&d, "Border"), [red, blue]);
        assert_eq!(fill_of(&d, "Own"), Srgba::rgb(0, 255, 0));
        assert_eq!(fill_of(&d, "MixedKid"), Srgba::rgb(128, 0, 128));
        let style = if scheme == Scheme::Dark { blue } else { red };
        for (id, want) in [
            ("Light", red),
            ("Forced", blue),
            ("Both", style),
            ("Style", style),
        ] {
            assert_eq!(fill_of(&d, id), want, "{scheme:?} {id}");
        }
        assert_eq!(textures_in(&d, "LightImage"), [a]);
        let styled = if scheme == Scheme::Dark { b } else { a };
        assert_eq!(textures_in(&d, "StyleImage"), [styled], "{scheme:?}");
    }
}

/// The background layer `id` draws with texture `t`: its placement and
/// its shape's box (left, top, width, height).
fn layer_of(
    d: &Doc,
    id: &str,
    t: og_draw::TextureHandle,
) -> (og_draw::UiBackground, [f32; 4], og_draw::UiBlend) {
    let e = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .find_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (e.x..=e.x + e.width).contains(&shape.center.x)
                    && (e.y..=e.y + e.height).contains(&shape.center.y) =>
            {
                match &shape.paint {
                    og_draw::UiPaint::Image(i) if i.texture == t => match i.fit {
                        og_draw::UiFit::Background(bg) => Some((
                            bg,
                            [
                                shape.center.x - shape.half.x,
                                shape.center.y - shape.half.y,
                                2.0 * shape.half.x,
                                2.0 * shape.half.y,
                            ],
                            shape.blend,
                        )),
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} draws no background layer of {t:?}"))
}

/// CSS Backgrounds 3 and 4's layer properties place each image where the
/// spec's arithmetic puts it: og-render's own `place` reads the emitted
/// layer, and each answer is worked out by hand from §3.6–3.11.
/// `right 3px top 25%` of an 8×4 image in a 20 px box starts at
/// x = 20 − 8 − 3 = 9 and y = 25% × (20 − 4) = 4; `space` puts three 6 px
/// tiles at 0, 7 and 14; `round` stretches them to 20 / 3; `cover` makes a
/// 4×4 image 20×20 in a 20×10 box, centred 5 px above it; a `fixed` layer
/// tiles from the viewport's origin. `background-position-x` overrides the
/// shorthand's x only, `background-origin: content-box` places inside the
/// border and padding, `background-clip: padding-box` cuts the colour to
/// inside the border, and each layer keeps its own blend mode, the first
/// layer drawn last.
///
/// Ways it could fail: a position keyword's offset taken from the wrong
/// edge, a percentage of the box rather than of the box less the image,
/// space's gaps at the edges, round not keeping a whole number, cover
/// fitting the short side, a fixed layer placed from the element, the
/// layers drawn in written order, or one blend mode applied to all.
#[test]
fn background_layers_size_place_repeat_clip_and_blend_as_css_does() {
    let text = r#"<meta name="component" content="t">
<column>
  <div id="Place" style="background: url(a.png) right 3px top 25% / 8px 4px no-repeat"></div>
  <div id="Axis" style="background: url(a.png) no-repeat; background-size: 8px 4px; background-position: left 1px top 2px; background-position-x: right 3px"></div>
  <div id="RepeatX" style="background: url(a.png) 0 0 / 8px 4px repeat-x"></div>
  <div id="Space" style="background: url(a.png) 0 0 / 6px 6px space"></div>
  <div id="Round" style="background: url(a.png) 0 0 / 6px 6px round"></div>
  <div id="Cover" style="height: 10px; background: url(a.png) center / cover no-repeat"></div>
  <div id="Contain" style="height: 10px; background: url(a.png) center / contain no-repeat"></div>
  <div id="Origin" style="box-sizing: border-box; padding: 2px; border: 2px solid transparent; background: url(a.png) no-repeat content-box"></div>
  <div id="Clip" style="box-sizing: border-box; border: 2px solid transparent; background-color: #ff0000; background-clip: padding-box"></div>
  <div id="Fixed" style="margin-left: 30px; background: url(a.png) fixed"></div>
  <div id="Blend" style="background: url(a.png), url(b.png), #ff0000; background-blend-mode: multiply, screen"></div>
</column>
<style>div { width: 20px; height: 20px; flex-shrink: 0; }</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    let mut h = Handles::default();
    let (a, b) = (h.texture(), h.texture());
    ui.define_texture("ui/a.png", a);
    ui.define_texture("ui/b.png", b);
    let d = Doc::with_ui(ui, text);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    // Where box point `p` (from the element's top left) falls in the image.
    let at = |id: &str, p: [f32; 2], nat: [f32; 2]| {
        let e = d.get(id);
        let (bg, r, _) = layer_of(&d, id, a);
        bg.place([e.x + p[0] - r[0], e.y + p[1] - r[1]], [r[2], r[3]], nat)
            .map(|f| f.map(|v| (v * 1000.0).round() / 1000.0))
    };
    let nat = [4.0, 4.0];
    assert_eq!(at("Place", [10.0, 5.0], nat), Some([0.125, 0.25]));
    assert_eq!(at("Place", [8.5, 5.0], nat), None);
    assert_eq!(at("Place", [10.0, 8.5], nat), None);
    assert_eq!(at("Axis", [10.0, 3.0], nat), Some([0.125, 0.25]));
    assert_eq!(at("RepeatX", [18.0, 1.0], nat), Some([0.25, 0.25]));
    assert_eq!(at("RepeatX", [18.0, 5.0], nat), None);
    assert_eq!(at("Space", [6.5, 0.5], nat), None);
    assert_eq!(at("Space", [7.5, 0.5], nat), Some([0.083, 0.083]));
    assert_eq!(at("Space", [14.5, 14.5], nat), Some([0.083, 0.083]));
    assert_eq!(at("Round", [7.0, 7.0], nat), Some([0.05, 0.05]));
    assert_eq!(at("Cover", [5.0, 0.0], nat), Some([0.25, 0.25]));
    assert_eq!(at("Contain", [4.5, 5.0], nat), None);
    assert_eq!(at("Contain", [7.5, 5.0], nat), Some([0.25, 0.5]));
    assert_eq!(at("Origin", [3.5, 4.5], nat), None);
    assert_eq!(at("Origin", [4.5, 4.5], nat), Some([0.125, 0.125]));
    let f = d.get("Fixed");
    let tile = |v: f32| ((v.rem_euclid(8.0) / 8.0) * 1000.0).round() / 1000.0;
    assert_eq!(
        at("Fixed", [0.0, 0.0], [8.0, 8.0]),
        Some([tile(f.x), tile(f.y)])
    );
    // The colour, cut inside the border.
    let c = d.get("Clip");
    let cut = d.ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::Quad { rect, color, .. }
            if same(*color, Srgba::rgb(255, 0, 0)) && (rect.y - c.y).abs() < 3.0 =>
        {
            Some(*rect)
        }
        DrawCmd::Shape { shape, .. }
            if shape.paint == og_draw::UiPaint::Solid([1.0, 0.0, 0.0, 1.0])
                && (shape.center.y - c.y - 10.0).abs() < 0.5 =>
        {
            Some(Rect::new(
                shape.center.x - shape.half.x,
                shape.center.y - shape.half.y,
                2.0 * shape.half.x,
                2.0 * shape.half.y,
            ))
        }
        _ => None,
    });
    assert_eq!(cut, Some(Rect::new(c.x + 2.0, c.y + 2.0, 16.0, 16.0)));
    assert_eq!(textures_in(&d, "Blend"), [b, a], "the first layer on top");
    assert_eq!(layer_of(&d, "Blend", a).2, og_draw::UiBlend::Multiply);
    assert_eq!(layer_of(&d, "Blend", b).2, og_draw::UiBlend::Screen);
}

/// The shapes drawn with their centre inside `id`, in draw order.
fn shapes_in(d: &Doc, id: &str) -> Vec<og_draw::UiShape> {
    let e = d.get(id);
    d.ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. }
                if (e.x..=e.x + e.width).contains(&shape.center.x)
                    && (e.y..=e.y + e.height).contains(&shape.center.y) =>
            {
                Some((**shape).clone())
            }
            _ => None,
        })
        .collect()
}

/// The clip `id` pushes: its path's points from the element's top left,
/// and its fill rule.
fn clip_of(d: &Doc, id: &str) -> Option<(Vec<[f32; 2]>, og_draw::UiFillRule)> {
    let e = d.get(id);
    d.ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::PushClip {
            geometry: og_draw::UiGeometry::Path(p),
            center,
            ..
        } if (e.x..=e.x + e.width).contains(&center.0)
            && (e.y..=e.y + e.height).contains(&center.1) =>
        {
            let mut points = Vec::new();
            for el in p.path.elements() {
                let end = match *el {
                    og_draw::kurbo::PathEl::MoveTo(q)
                    | og_draw::kurbo::PathEl::LineTo(q)
                    | og_draw::kurbo::PathEl::QuadTo(_, q)
                    | og_draw::kurbo::PathEl::CurveTo(_, _, q) => q,
                    og_draw::kurbo::PathEl::ClosePath => continue,
                };
                let q = [
                    (end.x as f32 + center.0 - e.x).round(),
                    (end.y as f32 + center.1 - e.y).round(),
                ];
                if points.last() != Some(&q) {
                    points.push(q);
                }
            }
            Some((points, p.fill_rule))
        }
        _ => None,
    })
}

/// `opacity` (a number or, CSS Color 4 §5.2, a percentage), `outline`
/// (CSS UI 4 §4: `thin`, `medium` and `thick` are 1, 3 and 5 px, a
/// `double` two lines a third as wide whose outer edge is the width out,
/// `outline-offset` moving it out), `image-rendering: crisp-edges`
/// (nearest sampling, CSS Images 3 §5.3), CSS 2's `clip: rect()` (CSS
/// Masking 1 §8.1: edges offset from the top left, `auto` the box's own,
/// only out of flow) and `clip-path: path()` (its points in the box, its
/// fill rule) draw as their specs say.
///
/// Ways it could fail: `50%` refused or read as 50, an outline drawn
/// inside the box or a double's lines both at the box edge, crisp-edges
/// sampled smoothly, `clip`'s right and bottom read as insets from the far
/// edges (rect() is not inset()), `clip` cutting a box in flow, or a
/// path's even-odd rule lost.
#[test]
fn opacity_outline_crisp_edges_and_clips_draw_as_css_says() {
    let text = r#"<meta name="component" content="t">
<column>
  <div id="Half" style="opacity: 50%; background: #ff0000"></div>
  <div id="Quarter" style="opacity: 0.25; background: #ff0000"></div>
  <div id="Thin" style="outline: thin solid #00ff00"></div>
  <div id="Medium" style="outline: medium dashed #00ff00"></div>
  <div id="Thick" style="outline: thick double #00ff00"></div>
  <div id="Offset" style="outline: 2px solid #00ff00; outline-offset: 3px"></div>
  <div id="None" style="outline: none"></div>
  <div id="Crisp" style="background: url(a.png); image-rendering: crisp-edges"></div>
  <div id="Smooth" style="background: url(a.png); image-rendering: smooth"></div>
  <div style="position: relative">
    <div id="Clip" style="position: absolute; clip: rect(2px, 10px, 12px, 4px); background: #ff0000"></div>
  </div>
  <div style="position: relative">
    <div id="Auto" style="position: absolute; clip: rect(2px auto auto 4px); background: #ff0000"></div>
  </div>
  <div id="Flow" style="clip: rect(2px, 10px, 12px, 4px); background: #ff0000"></div>
  <div id="Path" style="clip-path: path(evenodd, 'M0 0 L20 0 L0 20 Z'); background: #ff0000"></div>
  <div id="NonZero" style="clip-path: path('M 2 2 h 10 v 10 z'); background: #ff0000"></div>
</column>
<style>column > div, column > div > div { width: 20px; height: 20px; flex-shrink: 0; margin-bottom: 8px; }</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_texture("ui/a.png", Handles::default().texture());
    let d = Doc::with_ui(ui, text);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    for (id, alpha) in [("Half", 0.5), ("Quarter", 0.25)] {
        let fill = fill_of(&d, id);
        assert!(about(fill, [255, 0, 0], alpha), "{id}: {fill:?}");
    }
    // Each outline's strokes: width, how far out its centre line lies
    // past the 10 px half box, and dashed.
    let outline = |id: &str| -> Vec<(f32, f32, bool)> {
        shapes_in(&d, id)
            .iter()
            .filter_map(|s| match s.mode {
                og_draw::UiMode::Stroke(st) => {
                    let mid = match st.align {
                        og_draw::UiAlign::Outside => st.width / 2.0,
                        og_draw::UiAlign::Inside => -st.width / 2.0,
                        og_draw::UiAlign::Center => 0.0,
                    };
                    let out = s.half.x - 10.0 + mid;
                    Some((
                        (st.width * 1000.0).round() / 1000.0,
                        (out * 1000.0).round() / 1000.0,
                        st.dash[0] > 0.0,
                    ))
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(outline("Thin"), [(1.0, 0.5, false)]);
    // A dashed line is placed side by side (its corners solid).
    let medium = outline("Medium");
    assert!(
        medium.iter().all(|&(w, out, _)| (w, out) == (3.0, 1.5)) && medium.iter().any(|l| l.2),
        "{medium:?}"
    );
    // The outer line ends 5 px out, the inner starts at the box.
    assert_eq!(
        outline("Thick"),
        [(1.667, 0.833, false), (1.667, 4.167, false)]
    );
    assert_eq!(outline("Offset"), [(2.0, 4.0, false)]);
    assert_eq!(outline("None"), []);
    let nearest = |id: &str| {
        shapes_in(&d, id).iter().find_map(|s| match &s.paint {
            og_draw::UiPaint::Image(i) => Some(i.nearest),
            _ => None,
        })
    };
    assert_eq!(nearest("Crisp"), Some(true));
    assert_eq!(nearest("Smooth"), Some(false));
    let rect = |l: f32, t: f32, r: f32, b: f32| vec![[l, t], [r, t], [r, b], [l, b]];
    let bounds = |id: &str| {
        let (points, _) = clip_of(&d, id).unwrap_or_else(|| panic!("{id} is not cut"));
        let xs = points.iter().map(|p| p[0]);
        let ys = points.iter().map(|p| p[1]);
        let (l, r) = (
            xs.clone().fold(f32::MAX, f32::min),
            xs.fold(f32::MIN, f32::max),
        );
        let (t, b) = (
            ys.clone().fold(f32::MAX, f32::min),
            ys.fold(f32::MIN, f32::max),
        );
        rect(l, t, r, b)
    };
    assert_eq!(bounds("Clip"), rect(4.0, 2.0, 10.0, 12.0));
    assert_eq!(bounds("Auto"), rect(4.0, 2.0, 20.0, 20.0));
    assert!(
        clip_of(&d, "Flow").is_none(),
        "clip cuts only a box out of flow"
    );
    assert_eq!(
        clip_of(&d, "Path"),
        Some((
            vec![[0.0, 0.0], [20.0, 0.0], [0.0, 20.0]],
            og_draw::UiFillRule::EvenOdd
        ))
    );
    assert_eq!(
        clip_of(&d, "NonZero"),
        Some((
            vec![[2.0, 2.0], [12.0, 2.0], [12.0, 12.0]],
            og_draw::UiFillRule::NonZero
        ))
    );
}

/// An SVG shape's `fill-opacity` and `stroke-opacity` (attribute, number
/// or percentage, and CSS) scale each paint's alpha alone; `paint-order`
/// (SVG 2 §13.7, on the shape or inherited) draws its stroke under its
/// fill, the paints it leaves out in their default order; a `<mask>`'s
/// `mask-type: alpha` (CSS Masking 1 §7.1) cuts by its content's alpha
/// where the default `luminance` cuts by luminance times alpha (red's is
/// 0.2125); and `mask-border` (CSS Masking 1 §8) slices its image: a
/// gradient's 30 % of a 20 px border area is 6 px.
///
/// Ways it could fail: one opacity applied to both paints, a percentage
/// refused, `paint-order` parsed but never drawn (it was), an inherited
/// order lost, an alpha mask still read by luminance, or a mask border's
/// slice taken of the gradient as if it had no size.
#[test]
fn svg_paint_opacities_order_and_mask_types_draw_as_svg_says() {
    let text = r##"<meta name="component" content="t">
<column>
  <svg id="Opacity" viewBox="0 0 10 10"><rect width="10" height="10" fill="#ff0000" fill-opacity="0.25" stroke="#0000ff" stroke-opacity="50%" stroke-width="2"/></svg>
  <svg id="CssOpacity" viewBox="0 0 10 10"><rect width="10" height="10" fill="#ff0000" stroke="#0000ff" stroke-width="2" style="fill-opacity: 75%; stroke-opacity: 0.125"/></svg>
  <svg id="Normal" viewBox="0 0 10 10"><rect width="10" height="10" fill="#ff0000" stroke="#0000ff" stroke-width="2"/></svg>
  <svg id="Stroke" viewBox="0 0 10 10"><rect width="10" height="10" fill="#ff0000" stroke="#0000ff" stroke-width="2" style="paint-order: stroke"/></svg>
  <svg id="Inherited" viewBox="0 0 10 10" style="paint-order: markers stroke"><rect width="10" height="10" fill="#ff0000" stroke="#0000ff" stroke-width="2"/></svg>
  <svg id="Masks" viewBox="0 0 20 10">
    <defs>
      <linearGradient id="g"><stop offset="0" stop-color="#ff0000"/><stop offset="1" stop-color="#ff0000" stop-opacity="0.5"/></linearGradient>
      <mask id="alpha" mask-type="alpha"><rect width="10" height="10" fill="url(#g)"/></mask>
      <mask id="luminance"><rect width="10" height="10" fill="url(#g)"/></mask>
    </defs>
    <rect id="AlphaCut" width="10" height="10" fill="#0000ff" mask="url(#alpha)"/>
    <rect id="LuminanceCut" x="10" width="10" height="10" fill="#0000ff" mask="url(#luminance)"/>
  </svg>
  <div id="MaskBorder" style="width: 20px; height: 20px; background: #ff0000; mask-border: linear-gradient(#000, transparent) 30% / 4px"></div>
</column>
<style>svg { width: 20px; height: 20px; flex-shrink: 0; } #Masks { width: 40px; }</style>"##;
    let d = Doc::new(text, &[]);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    // What a shape draws, in order: fill or stroke, and its alpha.
    let paints = |id: &str| -> Vec<(&'static str, f32)> {
        shapes_in(&d, id)
            .iter()
            .filter_map(|s| match s.paint {
                og_draw::UiPaint::Solid(c) => Some((
                    if matches!(s.mode, og_draw::UiMode::Fill) {
                        "fill"
                    } else {
                        "stroke"
                    },
                    (c[3] * 1000.0).round() / 1000.0,
                )),
                _ => None,
            })
            .collect()
    };
    assert_eq!(paints("Opacity"), [("fill", 0.25), ("stroke", 0.5)]);
    assert_eq!(paints("CssOpacity"), [("fill", 0.75), ("stroke", 0.125)]);
    assert_eq!(paints("Normal"), [("fill", 1.0), ("stroke", 1.0)]);
    assert_eq!(paints("Stroke"), [("stroke", 1.0), ("fill", 1.0)]);
    assert_eq!(paints("Inherited"), [("stroke", 1.0), ("fill", 1.0)]);
    // The masks' gradients, as their stops' colour and alpha.
    let masks: Vec<Vec<[f32; 4]>> =
        d.ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::BeginLayer {
                    mask: Some(og_draw::UiMask::Gradient(g)),
                    ..
                } => Some(
                    g.stops
                        .iter()
                        .map(|s| s.color.map(|v| (v * 10_000.0).round() / 10_000.0))
                        .collect(),
                ),
                _ => None,
            })
            .collect();
    assert_eq!(
        masks,
        [
            vec![[1.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, 0.5]],
            vec![[1.0, 1.0, 1.0, 0.2125], [1.0, 1.0, 1.0, 0.1063]],
        ],
        "alpha, then luminance"
    );
    let border = shapes_in(&d, "MaskBorder")
        .iter()
        .find_map(|s| match &s.paint {
            og_draw::UiPaint::GradientImage(g) => match g.fit {
                og_draw::UiFit::BorderImage(b) => Some(b),
                _ => None,
            },
            _ => None,
        });
    let border = border.expect("the mask border's image");
    assert_eq!(
        (border.slice, border.width, border.fill),
        ([6.0; 4], [4.0; 4], false)
    );
}
