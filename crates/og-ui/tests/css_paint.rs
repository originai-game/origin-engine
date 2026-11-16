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
