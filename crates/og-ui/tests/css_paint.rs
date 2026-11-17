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
fn authored_native_backgrounds_replace_material_layers() {
    for (tag, css, gradient, red) in [
        ("panel", "", true, false),
        ("panel", "background: #ff0000;", false, true),
        ("panel", "background: transparent;", false, false),
        ("panel", "background-image: none;", false, false),
        ("panel", "background-color: #ff0000;", true, true),
        ("button", "", false, false),
        ("button", "background-color: #ff0000;", false, true),
        (
            "button",
            "background: #ff0000; border: 3px solid #00ff00; border-radius: 0;",
            false,
            true,
        ),
        ("svg", "background: transparent;", false, true),
    ] {
        let content = if tag == "svg" {
            r##"<rect width="100" height="80" fill="#ff0000"/>"##
        } else {
            ""
        };
        let d = Doc::new(
            &format!(
                r#"<meta name="component" content="t">
<{tag} id="Box">{content}</{tag}><style>#Box {{ width: 100px; height: 80px; {css} }}</style>"#
            ),
            &[],
        );
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{tag} {css}: {:?}",
            d.ui.frame().warnings
        );
        let paints =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|item| match &item.cmd {
                    DrawCmd::Shape { shape, .. } => Some(&shape.paint),
                    _ => None,
                })
                .collect::<Vec<_>>();
        assert_eq!(
            paints
                .iter()
                .any(|paint| matches!(paint, og_draw::UiPaint::GradientImage(_))),
            gradient,
            "{tag} {css}: default images reset only when authored"
        );
        assert_eq!(
            paints.iter().any(
                |paint| matches!(paint, og_draw::UiPaint::Solid(c) if *c == [1.0, 0.0, 0.0, 1.0])
            ),
            red,
            "{tag} {css}: authored colour"
        );
        if css.contains("border:") {
            assert!(d.ui.frame().items.iter().any(|item| {
                matches!(&item.cmd, DrawCmd::Shape { shape, .. }
                    if matches!(shape.paint, og_draw::UiPaint::Solid([0.0, 1.0, 0.0, 1.0]))
                    && matches!(shape.geometry, og_draw::UiGeometry::Rect { radii, .. } if radii == [0.0; 4]))
            }), "the author's square green border replaces the button plate");
        }
    }
}

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

/// An `accent-color` is a seed: the control's primary and what it draws on
/// it are the style's derivation of `--seed-primary` with the accent as the
/// seed. The default style holds its primary out of the band of lightness
/// where neither ink reads, so a mid blue (OKLCH 0.6) is drawn at 0.74 with
/// black on it, in either scheme. Fails when the accent is painted as given.
#[test]
fn accent_color_is_derived_as_the_styles_primary_seed() {
    let want = css_color::parse_str("oklch(0.74 0.12 250)")
        .unwrap()
        .literal()
        .unwrap();
    for scheme in [Scheme::Dark, Scheme::Light] {
        let mut d = Doc::new(
            r#"<meta name="component" content="t">
<column><input type="checkbox" id="Blue" checked style="accent-color: oklch(0.6 0.12 250)" /></column>"#,
            &[],
        );
        d.ui.set_theme(Theme::new(scheme));
        for _ in 0..120 {
            d.step(InputFrame::default());
        }
        let paints = paints_in(&d.ui, &d.get("Blue"));
        assert!(
            paints.iter().any(|&c| same(c, want)),
            "{scheme:?}: no {want:?} in {paints:?}"
        );
        let tick = tick_in(&d.ui, &d.get("Blue"));
        assert!(
            tick.contrast(want) >= 4.5,
            "{scheme:?}: {tick:?} on {want:?}"
        );
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

/// CSS UI 4 §5.2.3 `caret-shape`: `bar` (and `auto`) is a thin bar at the
/// insertion point; `block` is a full-height rectangle over the next
/// character's advance, drawn under the text so the character stays
/// legible; `underscore` is a thin line under that advance at the bottom of
/// the bar's box. With no next character both take 1ch after the last one.
/// The shape inherits.
#[test]
fn caret_shape_draws_a_bar_block_or_underscore_over_the_next_character() {
    let size = 20.0;
    let font = test_font();
    // The spec's widths, from the font: the next character's advance, 1ch.
    let next = font.metrics.measure("i", size);
    let zero = font.metrics.measure("0", size);
    assert!(
        (next - zero).abs() > 2.0,
        "pick a next character unlike 1ch"
    );
    // The caret's rect and the index of its draw after `keys`, and the index
    // of the field's first glyph run.
    let caret = |shape: &str, keys: &[UiKey]| {
        let text = format!(
            r#"<meta name="component" content="t">
<column style="caret-shape: {shape}">
  <input id="F" value="aic" style="font-size: {size}px; caret-color: #00ff00; width: 400px" />
</column>"#
        );
        let mut d = Doc::with_ui(ui_with_font(test_font()), &text);
        d.focus = Some(d.get("F").path.clone());
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
        d.step(InputFrame {
            keys: keys.to_vec(),
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
        let e = d.get("F");
        let items = &d.ui.frame().items;
        let glyphs = items
            .iter()
            .position(|it| matches!(it.cmd, DrawCmd::Text { .. } | DrawCmd::StyledText { .. }));
        let (at, r) = items
            .iter()
            .enumerate()
            .find_map(|(k, it)| match &it.cmd {
                DrawCmd::Shape { shape, .. }
                    if shape.mode == og_draw::UiMode::Fill
                        && matches!(shape.paint, og_draw::UiPaint::Solid(c) if same(c, Srgba::rgb(0, 255, 0)))
                        && (e.x..e.x + e.width).contains(&shape.center.x) =>
                {
                    Some((k, (shape.center - shape.half, shape.half * 2.0)))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{shape}: no caret drawn"));
        (r, at, glyphs.expect("the field's text"))
    };
    let mid = [UiKey::Home, UiKey::Right];
    let close = |a: f32, b: f32| (a - b).abs() <= 1.0;
    // Before "i": the bar marks the insertion point.
    let ((bar_at, bar), _, _) = caret("bar", &mid);
    assert!(bar.x <= 1.0 && bar.y > 0.6 * size, "bar {bar_at:?} {bar:?}");
    let ((auto_at, auto), _, _) = caret("auto", &mid);
    assert!(
        close(auto_at.x, bar_at.x) && close(auto.x, bar.x),
        "auto draws the bar"
    );
    // A block over "i": its advance wide, the bar's height, under the glyphs.
    let ((at, sz), k, glyphs) = caret("block", &mid);
    assert!(
        close(at.x, bar_at.x) && close(sz.x, next),
        "block {at:?} {sz:?}, i {next}"
    );
    assert!(
        close(at.y, bar_at.y) && close(sz.y, bar.y),
        "block {at:?} {sz:?}"
    );
    assert!(
        k < glyphs,
        "the block draws under the text ({k} ≥ {glyphs})"
    );
    // An underscore under "i": its advance wide, thin, at the bar's bottom.
    let ((at, sz), k, glyphs) = caret("underscore", &mid);
    assert!(
        close(at.x, bar_at.x) && close(sz.x, next),
        "underscore {at:?} {sz:?}"
    );
    assert!(sz.y <= 1.0, "underscore thin: {sz:?}");
    assert!(
        close(at.y + sz.y, bar_at.y + bar.y),
        "underscore at the bar's bottom"
    );
    assert!(k > glyphs, "the underscore draws over the text");
    // At the end: no next character, so 1ch after the last one.
    let ((end_at, _), _, _) = caret("bar", &[UiKey::End]);
    assert!(end_at.x > bar_at.x + next, "the end lies past 'i'");
    for shape in ["block", "underscore"] {
        let ((at, sz), _, _) = caret(shape, &[UiKey::End]);
        assert!(
            close(at.x, end_at.x) && close(sz.x, zero),
            "{shape} at the end: {at:?} {sz:?}, 1ch {zero}"
        );
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
            .find_map(|it| match &it.cmd {
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

/// `text-shadow` inherits (CSS Text Decoration 3 §4): a halo written once
/// on a HUD's root reaches each of its texts, its `currentColor` read as
/// each text's own ink, so `contrast-color(currentColor)` is black round
/// gold ink and white round navy, as a browser draws it. A text that sets
/// `text-shadow: none` drops it and one with a shadow of its own draws
/// that alone. The engine adds nothing to text drawn over the scene: what
/// holds a text there is its document's CSS.
///
/// Ways it could fail: the halo not inherited (the HUD's texts bare), its
/// `currentColor` read once on the holder (gold and navy outlined alike),
/// an inherited halo stacked on a text's own shadow, or text with no
/// ground still dressed by the engine.
#[test]
fn a_text_shadow_is_inherited_in_each_texts_own_colour() {
    let text = r#"<meta name="component" content="t">
<column>
  <p>Bare</p>
  <div style="text-shadow: 1px 0 0 contrast-color(currentColor), -1px 0 0 contrast-color(currentColor)">
    <p style="color: #ffd60a">Gold</p>
    <div><p style="color: #1a237e">Navy</p></div>
    <p style="text-shadow: none">Off</p>
    <p style="text-shadow: 0 1px 2px #ff0000">Own</p>
  </div>
</column>"#;
    let d = Doc::with_font(text, &[], test_font());
    let shadows = |t: &str| {
        text_effects(&d.ui, t)
            .1
            .map(|fx| fx.shadows)
            .unwrap_or_default()
    };
    for (t, color) in [
        ("Gold", [0.0, 0.0, 0.0, 1.0]),
        ("Navy", [1.0, 1.0, 1.0, 1.0]),
    ] {
        let s = shadows(t);
        assert!(
            s.len() == 2 && s.iter().all(|s| s.color == color),
            "{t}: {s:?}"
        );
    }
    assert!(shadows("Bare").is_empty(), "{:?}", shadows("Bare"));
    assert!(shadows("Off").is_empty(), "{:?}", shadows("Off"));
    let own = shadows("Own");
    assert!(own.len() == 1 && own[0].blur == 2.0, "{own:?}");
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

#[test]
fn conic_stops_compute_fraction_math_and_registered_percentages() {
    for (registration, custom, stop) in [
        ("", "", "calc(var(--v) * 100%)"),
        (
            "@property --sweep { syntax: '<percentage>'; inherits: false; initial-value: 0%; }",
            "--sweep: calc(var(--v) * 100%);",
            "var(--sweep)",
        ),
        (
            "@property --sweep { syntax: '<angle>'; inherits: false; initial-value: 0deg; }",
            "--sweep: calc(var(--v) * 360deg);",
            "var(--sweep)",
        ),
    ] {
        let text = format!(
            r#"<meta name="component" content="t">
<div id="Sweep" style:--v={{$value}}></div>
<style>
{registration}
#Sweep {{ width: 100px; height: 100px; {custom}
  background: conic-gradient(#000 {stop}, transparent 0); }}
</style>"#
        );
        let mut d = Doc::new(&text, &[("value", serde_json::json!(0.35))]);
        for value in [0.35, 0.8] {
            d.set("value", serde_json::json!(value));
            d.step(InputFrame::default());
            let stops =
                d.ui.frame()
                    .items
                    .iter()
                    .find_map(|item| match &item.cmd {
                        DrawCmd::Shape { shape, .. } => match &shape.paint {
                            og_draw::UiPaint::GradientImage(g) => Some(&g.gradient.stops),
                            _ => None,
                        },
                        _ => None,
                    })
                    .expect("the sweep draws a gradient");
            assert_eq!(stops.len(), 2);
            for stop in stops {
                assert!(
                    (f64::from(stop.offset) - value).abs() < 0.001,
                    "{registration}: {} != {value}",
                    stop.offset
                );
            }
            assert!(
                d.ui.frame().warnings.is_empty(),
                "{:?}",
                d.ui.frame().warnings
            );
        }
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
            // A straight closing segment and ClosePath describe the same polygon.
            if points.len() > 1 && points.last() == points.first() {
                points.pop();
            }
            Some((points, p.fill_rule))
        }
        _ => None,
    })
}

#[test]
fn polygon_math_keeps_each_coordinates_reference_axis_on_resize() {
    let source = r#"<meta name="component" content="t">
<div><div id="Shape" class:large={$large}></div></div>
<style>
#Shape { width: 160px; height: 80px; background: #ff0000;
  clip-path: polygon(evenodd, calc(25% + 7px) calc(100% - 13px), calc(100% - 9px) 0, 100% 100%, calc(0% - 3px) 50%); }
#Shape.large { width: 240px; height: 120px; }
</style>"#;
    let mut d = Doc::new(source, &[("large", serde_json::json!(false))]);
    assert_eq!(
        clip_of(&d, "Shape"),
        Some((
            vec![[47.0, 67.0], [151.0, 0.0], [160.0, 80.0], [-3.0, 40.0]],
            og_draw::UiFillRule::EvenOdd
        ))
    );
    d.set("large", serde_json::json!(true));
    assert_eq!(
        clip_of(&d, "Shape"),
        Some((
            vec![[67.0, 107.0], [231.0, 0.0], [240.0, 120.0], [-3.0, 60.0]],
            og_draw::UiFillRule::EvenOdd
        )),
        "percentages follow each used axis; pixel offsets stay fixed"
    );
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
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

/// `image-orientation` (CSS Images 3 §5.1), inherited: `none` on a parent
/// reaches its `<img>` and its background, which draw their texels as
/// stored, while a child saying `from-image` (the initial value) turns
/// them upright. Where each emitted image samples comes from og-render's
/// own `place` (ui.wgsl's `image_uv`), against a 2 × 1 texture, red then
/// blue, carrying each EXIF orientation (EXIF 2.32 tag 0x0112): under 6
/// (row 0 on the right, column 0 at the top) the upright image is 1 × 2
/// with red, the stored left column, on top; under 8 (row 0 on the left,
/// column 0 at the bottom) blue is on top; under 3 (turned 180°) blue is on
/// the left. `object-fit: none` draws it at its natural size, which is the
/// turned image's (1 × 2) and, under `none`, the stored one's (2 × 1).
///
/// Ways it could fail: `none` not reaching the image or not inherited, a
/// quarter turn taken the wrong way (6 and 8 swapped), the natural size
/// not swapped by a quarter turn, or `none` still turning the texels.
#[test]
fn image_orientation_turns_images_by_their_exif_or_draws_them_as_stored() {
    struct Env(og_draw::TextureHandle);
    impl bind::Env for Env {
        fn texture(&self, _: f64) -> Option<og_draw::TextureHandle> {
            Some(self.0)
        }
    }
    let mut handles = Handles::default();
    let t = handles.texture();
    let doc = r#"<meta name="component" content="or">
<div id="Root"><img id="Stored" src={1} alt="" /><img id="Upright" src={1} alt="" /><div id="BgStored"></div><div id="BgUpright"></div></div>
<style>
#Root { image-orientation: none; }
img { width: 10px; height: 20px; object-fit: none; }
#Upright, #BgUpright { image-orientation: from-image; }
#BgStored, #BgUpright { width: 10px; height: 10px; background: url(a.png); }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_texture("ui/a.png", t);
    ui.define_document(UiDocument::parse("ui/or.svelte", doc).expect("parses"));
    ui.mount_document(
        reactive::SourceId::new("test", "or"),
        "or",
        Default::default(),
        Default::default(),
        &Env(t),
    )
    .unwrap();
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
        ui.build_retained_documents(&Env(t)).unwrap();
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let images: Vec<og_draw::UiImage> = ui
        .frame()
        .items
        .iter()
        .filter_map(|i| match &i.cmd {
            DrawCmd::Shape { shape, .. } => match &shape.paint {
                og_draw::UiPaint::Image(im) => Some(*im),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(images.len(), 4, "{images:?}");
    let (stored, upright) = (images[0], images[1]);
    assert_eq!(
        (stored.fit, upright.fit),
        (og_draw::UiFit::None, og_draw::UiFit::None)
    );
    assert!(!stored.orient, "none inherited from the parent");
    assert!(upright.orient, "from-image on the child");
    assert!(matches!(images[2].fit, og_draw::UiFit::Background(_)));
    assert!(!images[2].orient, "a background under none draws as stored");
    assert!(images[3].orient, "a background under from-image turns");

    // The texel a point of the 10 × 20 box samples: red (the stored left
    // half), blue (the right) or nothing.
    let texel = |im: &og_draw::UiImage, exif: u32, p: [f32; 2]| {
        im.place(
            p,
            [10.0, 20.0],
            [2.0, 1.0],
            og_draw::UiOrientation::from_exif(exif),
        )
        .map(|f| if f[0] < 0.5 { "red" } else { "blue" })
    };
    // 6: 1 × 2 upright, centred at x 4.5..5.5, y 9..11.
    assert_eq!(texel(&upright, 6, [5.0, 9.5]), Some("red"));
    assert_eq!(texel(&upright, 6, [5.0, 10.5]), Some("blue"));
    assert_eq!(texel(&upright, 6, [4.0, 10.0]), None, "1 px wide");
    // 8: the other quarter turn.
    assert_eq!(texel(&upright, 8, [5.0, 9.5]), Some("blue"));
    assert_eq!(texel(&upright, 8, [5.0, 10.5]), Some("red"));
    // 3: 2 × 1, x 4..6, y 9.5..10.5, turned half round.
    assert_eq!(texel(&upright, 3, [4.5, 10.0]), Some("blue"));
    assert_eq!(texel(&upright, 1, [4.5, 10.0]), Some("red"));
    // none: the stored 2 × 1 whatever the texture carries.
    assert_eq!(texel(&stored, 6, [4.5, 10.0]), Some("red"));
    assert_eq!(texel(&stored, 6, [5.0, 10.8]), None, "1 px tall");
    assert_eq!(texel(&upright, 6, [5.0, 10.8]), Some("blue"));
}

/// `cross-fade()` (CSS Images 4 §2.6) draws its images as one: each part
/// placed as the layer places an image, scaled by its weight and added
/// (premultiplied, `plus-lighter`) in a group of its own, which the layer
/// then composites. The weights are the spec's normalisation, worked by
/// hand: `a 25%, b` gives b the 75% left; `a 80%, b 60%` sums to 140% and
/// is scaled to 80/140 and 60/140; `a 30%, red 20%` keeps its 50% (the
/// rest transparent: no third part), red a solid image; `a 70%, b 50%, a`
/// leaves the omitted one max(0, 100 − 120)% = 0 and scales to 70/120 and
/// 50/120; `a 10%, cross-fade(b 30%, a 70%) 90%` flattens to a 10%,
/// b 27%, a 63%. A percentage outside 0–100% or `none` is refused.
///
/// Ways it could fail: the parts drawn over each other (normal blend, so
/// the last hides the first) or outside a group (added to what is behind
/// the element), an omitted weight not taking what is left, a sum over
/// 100% left unscaled, a sum under it padded up, a nested cross-fade
/// weighted once, or the parts placed differently.
#[test]
fn cross_fade_sums_its_weighted_images_in_a_group() {
    let text = r#"<meta name="component" content="t">
<column>
  <div id="Two" style="background: cross-fade(url(a.png) 25%, url(b.png)) 2px 3px / 8px 4px no-repeat"></div>
  <div id="Over" style="background-image: cross-fade(url(a.png) 80%, url(b.png) 60%)"></div>
  <div id="Under" style="background-image: cross-fade(url(a.png) 30%, #ff0000 20%)"></div>
  <div id="Rest" style="background-image: cross-fade(url(a.png) 70%, url(b.png) 50%, url(a.png))"></div>
  <div id="Nested" style="background-image: cross-fade(url(a.png) 10%, cross-fade(url(b.png) 30%, url(a.png) 70%) 90%)"></div>
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
    // The parts each element's group adds, by name and weight (rounded),
    // and how the group composites.
    let group = |id: &str| {
        let e = d.get(id);
        let inside = |s: &og_draw::UiShape| {
            (e.x..=e.x + e.width).contains(&s.center.x)
                && (e.y..=e.y + e.height).contains(&s.center.y)
        };
        let mut parts = Vec::new();
        let mut open = false;
        for it in &d.ui.frame().items {
            match &it.cmd {
                DrawCmd::BeginLayer { .. } => {
                    open = true;
                    parts.clear();
                }
                DrawCmd::Shape { shape, .. } if open && inside(shape) => {
                    let name = match &shape.paint {
                        og_draw::UiPaint::Image(im) if im.texture == a => "a",
                        og_draw::UiPaint::Image(im) if im.texture == b => "b",
                        og_draw::UiPaint::GradientImage(g)
                            if g.gradient
                                .stops
                                .iter()
                                .all(|s| s.color == [1.0, 0.0, 0.0, 1.0]) =>
                        {
                            "red"
                        }
                        p => panic!("{id}: {p:?}"),
                    };
                    assert_eq!(shape.blend, og_draw::UiBlend::Add, "{id}: {name} adds");
                    parts.push((
                        name,
                        (shape.opacity * 1e4).round() / 1e4,
                        shape.paint.clone(),
                    ));
                }
                DrawCmd::EndLayer {
                    rect,
                    opacity,
                    blend,
                    ..
                } if (rect.x, rect.y, rect.width, rect.height) == (e.x, e.y, e.width, e.height)
                    && open =>
                {
                    return (parts, *opacity, *blend);
                }
                _ => {}
            }
        }
        panic!("{id}: no cross-fade group");
    };
    let names = |id: &str| {
        let (parts, opacity, blend) = group(id);
        assert_eq!((opacity, blend), (1.0, og_draw::UiBlend::Normal), "{id}");
        parts
            .into_iter()
            .map(|(n, w, _)| (n, w))
            .collect::<Vec<_>>()
    };
    assert_eq!(names("Two"), [("a", 0.25), ("b", 0.75)]);
    assert_eq!(names("Over"), [("a", 0.5714), ("b", 0.4286)]);
    assert_eq!(names("Under"), [("a", 0.3), ("red", 0.2)]);
    assert_eq!(names("Rest"), [("a", 0.5833), ("b", 0.4167), ("a", 0.0)]);
    assert_eq!(names("Nested"), [("a", 0.1), ("b", 0.27), ("a", 0.63)]);
    // Both parts placed as the layer places one image: 8 x 4 at (2, 3).
    let (two, ..) = group("Two");
    let fits: Vec<_> = two
        .iter()
        .map(|(_, _, p)| match p {
            og_draw::UiPaint::Image(im) => im.fit,
            p => panic!("{p:?}"),
        })
        .collect();
    assert_eq!(fits[0], fits[1]);
    let og_draw::UiFit::Background(bg) = fits[0] else {
        panic!("{:?}", fits[0])
    };
    assert_eq!(
        bg.size,
        og_draw::UiBackgroundSize::Given([Some(8.0), Some(4.0)])
    );
    assert_eq!(bg.offset, og_core::glam::Vec2::new(2.0, 3.0));

    let parse = |value: &str| {
        UiDocument::parse(
            "ui/t.svelte",
            &format!(
                "<meta name=\"component\" content=\"t\"><div id=\"A\"></div><style>#A {{ background-image: {value} }}</style>"
            ),
        )
        .map(|_| ())
    };
    assert!(parse("cross-fade(url(a.png) 50%, url(b.png))").is_ok());
    assert!(parse("cross-fade(url(a.png) 120%, url(b.png))").is_err());
    assert!(parse("cross-fade(none 50%, url(b.png))").is_err());
    assert!(parse("cross-fade(url(a.png) 50% 20%)").is_err());
}

/// `filter(<image>, <filter-value-list>)` (Filter Effects 1 §12) draws its
/// image through the functions in order, as the `filter` property does: a
/// layer per stage, the first applied innermost, the outermost
/// composited as the background layer. Expected values from Filter
/// Effects 1 §13: `grayscale(1)`'s first row is (0.2126, 0.7152, 0.0722),
/// `blur(2px)` is a Gaussian of σ 2 (the layer blur's radius 2σ = 4);
/// `invert(1)` maps a channel c to 1 − c and `opacity(50%)` halves alpha,
/// so `filter(filter(url(a.png), invert(1)), opacity(50%))` is one list,
/// invert then opacity. A cross-fade part filtered on its own is drawn
/// through its stage, which adds at its weight.
///
/// Ways it could fail: the stages applied in reverse (blur innermost), a
/// nested filter() list lost or reordered, the filter applied to the
/// element rather than the layer's image, a cross-fade part's filter
/// applied to the whole sum, or `url(#id)` (a document's filter) taken.
#[test]
fn filter_images_run_their_functions_in_order_on_the_image() {
    let text = r#"<meta name="component" content="t">
<column>
  <div id="Chain" style="background-image: filter(url(a.png), grayscale(1) blur(2px))"></div>
  <div id="Nested" style="background-image: filter(filter(url(a.png), invert(1)), opacity(50%))"></div>
  <div id="Fade" style="background-image: cross-fade(filter(url(a.png), invert(1)) 40%, url(b.png))"></div>
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
    // The commands from just before the first shape in `id` to the end of
    // its layers, as a short script: `[` a layer opening (its material),
    // a shape (its texture, opacity and blend), `]` a layer closing (its
    // opacity and blend).
    #[derive(Debug, PartialEq)]
    enum Step {
        Open(og_draw::UiMaterial, [[f32; 4]; og_draw::UI_MATERIAL_PARAMS]),
        Shape(&'static str, f32, og_draw::UiBlend),
        Close(f32, og_draw::UiBlend),
    }
    let script = |id: &str| {
        let e = d.get(id);
        let inside = |s: &og_draw::UiShape| {
            (e.x..=e.x + e.width).contains(&s.center.x)
                && (e.y..=e.y + e.height).contains(&s.center.y)
        };
        let items = &d.ui.frame().items;
        let first = items
            .iter()
            .position(|it| matches!(&it.cmd, DrawCmd::Shape { shape, .. } if inside(shape)))
            .unwrap_or_else(|| panic!("{id} draws nothing"));
        // Back over the layers opened right before it.
        let start = (0..first)
            .rev()
            .take_while(|&k| matches!(items[k].cmd, DrawCmd::BeginLayer { .. }))
            .last()
            .unwrap_or(first);
        let mut out = Vec::new();
        let mut depth = 0i32;
        for it in &items[start..] {
            match &it.cmd {
                DrawCmd::BeginLayer {
                    material, params, ..
                } => {
                    depth += 1;
                    out.push(Step::Open(*material, *params));
                }
                DrawCmd::Shape { shape, .. } if inside(shape) => {
                    let name = match &shape.paint {
                        og_draw::UiPaint::Image(im) if im.texture == a => "a",
                        og_draw::UiPaint::Image(im) if im.texture == b => "b",
                        p => panic!("{id}: {p:?}"),
                    };
                    out.push(Step::Shape(name, shape.opacity, shape.blend));
                }
                DrawCmd::EndLayer { opacity, blend, .. } => {
                    depth -= 1;
                    out.push(Step::Close(*opacity, *blend));
                }
                _ => {}
            }
            if depth == 0 {
                break;
            }
        }
        out
    };
    use og_draw::UiBlend::{Add, Normal};
    let matrix = og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::ColorMatrix);
    let blur = og_draw::UiMaterial::Builtin(og_draw::UiBuiltin::Blur);
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4);

    // grayscale innermost, blur outside it, the image inside both.
    let chain = script("Chain");
    assert_eq!(chain.len(), 5, "{chain:?}");
    let Step::Open(m0, p0) = &chain[0] else {
        panic!("{chain:?}")
    };
    let Step::Open(m1, p1) = &chain[1] else {
        panic!("{chain:?}")
    };
    assert_eq!((*m0, *m1), (blur, matrix), "blur applied last");
    assert_eq!(p0[0][0], 4.0, "σ 2");
    assert!(near(p1[0], [0.2126, 0.7152, 0.0722, 0.0]), "{:?}", p1[0]);
    assert_eq!(chain[2], Step::Shape("a", 1.0, Normal));
    assert_eq!(
        chain[3..],
        [Step::Close(1.0, Normal), Step::Close(1.0, Normal)]
    );

    // invert then opacity: one colour matrix, or two in that order.
    let nested = script("Nested");
    let mut m = Vec::new();
    for s in &nested {
        if let Step::Open(mat, p) = s {
            assert_eq!(*mat, matrix);
            m.push(*p);
        }
    }
    m.reverse(); // innermost (applied first) first
    let (rows, alpha) = match m.as_slice() {
        [one] => (one[0], one[3][0]),
        [inv, op] => {
            assert!(near(op[0], [1.0, 0.0, 0.0, 0.0]), "{op:?}");
            (inv[0], op[3][0])
        }
        m => panic!("{m:?}"),
    };
    assert!(near(rows, [-1.0, 0.0, 0.0, 1.0]), "{rows:?}");
    assert_eq!(alpha, 0.5);

    // The part's own stage adds at its weight; b adds plainly.
    let fade = script("Fade");
    assert_eq!(
        fade.iter()
            .filter(|s| !matches!(s, Step::Open(..)))
            .collect::<Vec<_>>(),
        [
            &Step::Shape("a", 1.0, Normal),
            &Step::Close(0.4, Add),
            &Step::Shape("b", 0.6, Add),
            &Step::Close(1.0, Normal),
        ]
    );
    let Step::Open(_, p) = &fade[1] else {
        panic!("{fade:?}")
    };
    assert!(near(p[0], [-1.0, 0.0, 0.0, 1.0]), "{fade:?}");

    let parse = |value: &str| {
        UiDocument::parse(
            "ui/t.svelte",
            &format!(
                "<meta name=\"component\" content=\"t\"><div id=\"A\"></div><style>#A {{ background-image: {value} }}</style>"
            ),
        )
        .map(|_| ())
    };
    assert!(parse("filter('a.png', sepia(1))").is_ok());
    assert!(parse("filter(url(a.png), url(#f))").is_err());
    assert!(parse("filter(none, blur(1px))").is_err());
    assert!(parse("filter(url(a.png))").is_err());
}

/// `background-clip: text` (CSS Backgrounds 4 §2.1) cuts the background
/// to the text of the element and of its descendants: a box whose text is
/// all in two child paragraphs draws its gradient in a group whose
/// coverage is both paragraphs' glyphs, opaque (the bottom layer kept on
/// them by source-in), drawn where its background is: before either
/// paragraph's own text, which then draws over it as it is.
///
/// The group goes in when the subtree closes, which moves the commands
/// after it, so a frame that keeps the box's subtree from the last one (a
/// sibling's text changes) must match the same state built whole. Run
/// with `OG_UI_CHECK_BOUNDED=1` too, which compares every frame with its
/// whole twin.
///
/// Ways it could fail: only the element's own (here empty) text clipping
/// it, so nothing draws; the descendants' glyphs drawn in their own
/// colour in the coverage; the group drawn over the text; a reused frame
/// holding the group twice, none, or in the wrong place; or a paragraph
/// kept while its sibling changes copied from where it stood before last
/// frame's group went in.
#[test]
fn background_clip_text_cuts_to_the_descendants_text() {
    let text = r#"<meta name="component" content="t">
<meta name="prop:label" content="string">
<column>
  <div id="Clip"><p>alpha</p><p>beta</p></div>
  <p id="Other">{label}</p>
</column>
<style>
#Clip { width: 120px; background: linear-gradient(90deg, #ff0000, #0000ff); background-clip: text; color: rgb(0 0 0 / 0); }
</style>"#;
    let mut d = Doc::with_font(text, &[("label", serde_json::json!("one"))], test_font());
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    let script = |d: &Doc| {
        let items = &d.ui.frame().items;
        let text_at = |k: usize, s: &str| {
            matches!(&items[k].cmd, DrawCmd::Text { text, .. } if text == s)
                || matches!(&items[k].cmd, DrawCmd::StyledText { text, .. }
                    if matches!(&**text, DrawCmd::Text { text, .. } if text == s))
        };
        let gradient = items
            .iter()
            .position(|it| {
                matches!(&it.cmd, DrawCmd::Shape { shape, .. }
                    if matches!(shape.paint, og_draw::UiPaint::GradientImage(_)))
            })
            .expect("the clipped gradient");
        // Its group: the layer open around it, closed at the same depth.
        let begin = (0..gradient)
            .rev()
            .find(|&k| matches!(items[k].cmd, DrawCmd::BeginLayer { .. }))
            .expect("its group");
        let mut depth = 0;
        let close = (begin..items.len())
            .find(|&k| {
                match items[k].cmd {
                    DrawCmd::BeginLayer { .. } => depth += 1,
                    DrawCmd::EndLayer { .. } => depth -= 1,
                    _ => {}
                }
                depth == 0
            })
            .expect("its group closes");
        // The bottom layer (the shorthand's transparent colour) is kept on
        // the glyphs by source-in; the gradient then blends over it there.
        let bottom = (begin..gradient)
            .find(|&k| matches!(items[k].cmd, DrawCmd::Shape { .. }))
            .expect("its bottom layer");
        let DrawCmd::Shape { shape, .. } = &items[bottom].cmd else {
            unreachable!()
        };
        assert_eq!(
            shape.blend,
            og_draw::UiBlend::SourceIn,
            "kept on the glyphs"
        );
        // The coverage before it: both paragraphs, opaque white.
        let coverage: Vec<(String, [f32; 4])> = items[begin + 1..bottom]
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Text { text, color, .. } => Some((text.clone(), *color)),
                _ => None,
            })
            .collect();
        assert_eq!(
            coverage,
            [
                ("alpha".to_string(), [1.0; 4]),
                ("beta".to_string(), [1.0; 4])
            ]
        );
        // The paragraphs' own text, after the group closes, once each.
        let own: Vec<usize> = (close + 1..items.len())
            .filter(|&k| text_at(k, "alpha") || text_at(k, "beta"))
            .collect();
        assert_eq!(own.len(), 2, "each paragraph's own text, over the group");
    };
    script(&d);
    d.set("label", serde_json::json!("two"));
    assert!(
        d.ui.frame().stats.kept_commands > 0,
        "the box's paint is kept"
    );
    script(&d);
    let whole = Doc::with_font(text, &[("label", serde_json::json!("two"))], test_font());
    assert_eq!(
        d.ui.frame().items,
        whole.ui.frame().items,
        "a reused frame is the whole one"
    );
    // One paragraph inside the box changes and the others are kept: they
    // are copied from where they stood after last frame's group.
    let inner = text.replace("<p>beta</p>", "<p>beta</p><p>{label}</p>");
    let mut d = Doc::with_font(&inner, &[("label", serde_json::json!("one"))], test_font());
    for label in ["two", "three"] {
        d.set("label", serde_json::json!(label));
        assert!(
            d.ui.frame().stats.kept_commands > 0,
            "the unchanged paragraphs are kept"
        );
        let whole = Doc::with_font(&inner, &[("label", serde_json::json!(label))], test_font());
        assert_eq!(
            d.ui.frame().items,
            whole.ui.frame().items,
            "a frame keeping the box's other paragraphs is the whole one"
        );
    }
}

/// `element(#id)` (CSS Images 4 §2.7) draws that element's rendering as a
/// background image: natural size its border box (here 20×10), tiled as
/// CSS Backgrounds 3 places an image (`5px 0 repeat-x` in a 50 px box:
/// tiles at x = −15, 5, 25 and 45), without its own transform (a 30°
/// turn), each tile its drawing scaled from that box. The image follows
/// the element on the frame it changes. An element that holds the
/// image's own group (itself, its ancestor) is an invalid image: nothing.
///
/// The copies go into the group once the pass has drawn every element,
/// which moves every later command; so each frame here is compared with
/// the same state built whole, when the element changes (a) and when
/// only a sibling after the group changes (b), whose earlier siblings
/// are copied from last frame across the insert. Run with
/// `OG_UI_CHECK_BOUNDED=1` too.
///
/// Ways it could fail: tiles placed from the wrong size or origin; the
/// element's transform or its ancestors' kept; last frame's drawing in
/// the tiles; a kept sibling copied from where it stood before last
/// frame's copies; or a self-reference drawing itself without end.
#[test]
fn element_images_draw_the_element_in_each_tile() {
    let text = r#"<meta name="component" content="t">
<meta name="prop:label" content="string">
<meta name="prop:other" content="string">
<column>
  <div id="Dst"></div>
  <div id="Src"><p>{label}</p></div>
  <p id="Mid">mid</p>
  <p id="After">{other}</p>
  <div id="Self"></div>
  <div id="Up"><div id="Down"></div></div>
</column>
<style>
#Src { width: 20px; height: 10px; flex-shrink: 0; background: #ff0000; transform: rotate(30deg); }
#Src p { margin: 0; font-size: 4px; }
#Dst { width: 50px; height: 25px; flex-shrink: 0; background: element(#Src) 5px 0 repeat-x; }
#Self { width: 10px; height: 10px; background: element(#Self) #00ff00; }
#Up { width: 10px; height: 10px; background: #00ff00; }
#Down { width: 10px; height: 10px; background: element(#Up); }
</style>"#;
    let doc = |label: &str, other: &str| {
        Doc::with_font(
            text,
            &[
                ("label", serde_json::json!(label)),
                ("other", serde_json::json!(other)),
            ],
            test_font(),
        )
    };
    let red = |c: &DrawCmd| match c {
        DrawCmd::Quad { color, .. } => same(*color, Srgba::rgb(255, 0, 0)),
        DrawCmd::Shape { shape, .. } => {
            matches!(shape.paint, og_draw::UiPaint::Solid(c) if same(c, Srgba::rgb(255, 0, 0)))
        }
        _ => false,
    };
    let screen = |it: &DrawItem| {
        let r = match &it.cmd {
            DrawCmd::Quad { rect, .. } => *rect,
            DrawCmd::Shape { shape, .. } => Rect::new(
                shape.center.x - shape.half.x,
                shape.center.y - shape.half.y,
                2.0 * shape.half.x,
                2.0 * shape.half.y,
            ),
            _ => unreachable!(),
        };
        let r = it.transform.map_rect(&r);
        [r.x, r.y, r.width, r.height].map(|v| (v * 100.0).round() / 100.0)
    };
    let check = |d: &Doc, label: &str| {
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        let items = &d.ui.frame().items;
        let dst = d.get("Dst");
        let src = d.get("Src");
        // The tiles: the red background, unturned, at the border box's size.
        let mut tiles: Vec<[f32; 4]> = items
            .iter()
            .filter(|it| red(&it.cmd))
            .map(screen)
            .filter(|r| (r[1] - dst.y).abs() < 0.01)
            .collect();
        tiles.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let want: Vec<[f32; 4]> = [-15.0, 5.0, 25.0, 45.0]
            .iter()
            .map(|x| [dst.x + x, dst.y, 20.0, 10.0])
            .collect();
        assert_eq!(tiles, want, "one copy a tile, from the 20×10 border box");
        // The element itself still draws turned where it is.
        let turned = items
            .iter()
            .filter(|it| red(&it.cmd))
            .map(screen)
            .find(|r| (r[1] - dst.y).abs() >= 0.01)
            .expect("the element's own drawing");
        assert!(turned[2] > 20.5 && turned[3] > 10.5, "{turned:?} {src:?}");
        // Its text in each tile is this frame's.
        let texts = items
            .iter()
            .filter(|it| {
                matches!(&it.cmd, DrawCmd::StyledText { text, .. }
                if matches!(&**text, DrawCmd::Text { text, .. } if text == label))
                    || matches!(&it.cmd, DrawCmd::Text { text, .. } if text == label)
            })
            .count();
        assert_eq!(texts, 5, "the element's text and one in each tile");
        // No tile more; and itself and its ancestor give no image: only
        // their own colours draw.
        let reds = items.iter().filter(|it| red(&it.cmd)).count();
        assert_eq!(reds, 5);
        let green = |c: &DrawCmd| match c {
            DrawCmd::Quad { color, .. } => same(*color, Srgba::rgb(0, 255, 0)),
            DrawCmd::Shape { shape, .. } => {
                matches!(shape.paint, og_draw::UiPaint::Solid(c) if same(c, Srgba::rgb(0, 255, 0)))
            }
            _ => false,
        };
        assert_eq!(
            items.iter().filter(|it| green(&it.cmd)).count(),
            2,
            "a cycle is no image"
        );
    };
    let mut d = doc("one", "x");
    check(&d, "one");
    // (a) The element changes: its tiles follow on that frame.
    d.set("label", serde_json::json!("two"));
    check(&d, "two");
    assert_eq!(d.ui.frame().items, doc("two", "x").ui.frame().items);
    // (b) Only a later sibling changes: the siblings between are kept and
    // copied across the group's copies.
    for other in ["y", "z"] {
        d.set("other", serde_json::json!(other));
        assert!(
            d.ui.frame().stats.kept_commands > 0,
            "the siblings are kept"
        );
        check(&d, "two");
        assert_eq!(
            d.ui.frame().items,
            doc("two", other).ui.frame().items,
            "a frame keeping the siblings is the whole one"
        );
    }
}

/// `box-decoration-break` (CSS Fragmentation 3 §5.4) on a text that two
/// balanced columns split: 15 lines of 20 px, 3 px padding and a 2 px
/// border (5 px a side), 80 px columns.
///
/// - `slice` (the initial value) draws the background and border of the
///   box as if unbroken, 5 + 15 × 20 + 5 = 310 tall, and cuts it at the
///   break: the first piece shows its top 5 + 8 × 20 = 165 (no bottom
///   border), the second the rest moved into the next column, 0 to 145
///   (no top border).
/// - `clone` gives each piece its own box, padding and border included:
///   the break needs 10 px more, so the columns balance at 170 (8 lines)
///   and 150 (7 lines), each drawn whole.
///
/// Ways it could fail: one box for the first piece only (the second
/// column bare); each piece cut from a box of its own size (borders at
/// the break); clone's padding left out of the balancing.
#[test]
fn box_decoration_break_slices_or_clones_a_split_text() {
    let doc = |mode: &str| {
        let text = format!(
            r#"<meta name="component" content="t">
<div><div id="Tx"><p id="P">one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty</p></div></div>
<style>
#Tx {{ columns: 2; column-gap: 0; width: 160px; }}
p {{ margin: 0; background: #ff0000; border: 2px solid #00ff00; padding: 3px; box-decoration-break: {mode}; }}
</style>"#
        );
        let d = Doc::with_font(&text, &[], test_font());
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        d
    };
    let r = |x: f32, y: f32, w: f32, h: f32| [x, y, w, h];
    // Each red background and green border: its box, and what of it the
    // clips it is drawn under leave.
    let drawn = |d: &Doc| {
        let mut clips: Vec<Rect> = Vec::new();
        let mut out = Vec::new();
        for it in &d.ui.frame().items {
            let (rect, green) = match &it.cmd {
                DrawCmd::PushClip { center, half, .. } => {
                    clips.push(it.transform.map_rect(&Rect::new(
                        center.0 - half.0,
                        center.1 - half.1,
                        2.0 * half.0,
                        2.0 * half.1,
                    )));
                    continue;
                }
                DrawCmd::PopClip => {
                    clips.pop();
                    continue;
                }
                DrawCmd::Quad { rect, color, .. } if same(*color, Srgba::rgb(255, 0, 0)) => {
                    (*rect, false)
                }
                DrawCmd::Shape { shape, .. } if matches!(shape.paint, og_draw::UiPaint::Solid(c) if same(c, Srgba::rgb(0, 255, 0))) => {
                    (
                        Rect::new(
                            shape.center.x - shape.half.x,
                            shape.center.y - shape.half.y,
                            2.0 * shape.half.x,
                            2.0 * shape.half.y,
                        ),
                        true,
                    )
                }
                _ => continue,
            };
            let rect = it.transform.map_rect(&rect);
            let shown = clips.iter().fold(rect, |a, c| a.intersect(c));
            let q = |r: Rect| [r.x, r.y, r.width, r.height].map(|v| (v * 100.0).round() / 100.0);
            out.push((green, q(rect), q(shown)));
        }
        out
    };
    let slice = doc("slice");
    assert_eq!(
        (slice.get("P").height, slice.get("Tx").height),
        (165.0, 165.0)
    );
    let whole = r(0.0, 0.0, 80.0, 310.0);
    let moved = r(80.0, -165.0, 80.0, 310.0);
    let (first, second) = (r(0.0, 0.0, 80.0, 165.0), r(80.0, 0.0, 80.0, 145.0));
    assert_eq!(
        drawn(&slice),
        [
            (false, whole, first),
            (false, moved, second),
            (true, whole, first),
            (true, moved, second)
        ],
        "slice: the unbroken box, cut at the break"
    );
    let clone = doc("clone");
    assert_eq!(
        (clone.get("P").height, clone.get("Tx").height),
        (170.0, 170.0)
    );
    let (first, second) = (r(0.0, 0.0, 80.0, 170.0), r(80.0, 0.0, 80.0, 150.0));
    assert_eq!(
        drawn(&clone),
        [
            (false, first, first),
            (false, second, second),
            (true, first, first),
            (true, second, second)
        ],
        "clone: a whole box for each piece"
    );
    // The second piece's first line sits inside its own padding and border.
    let fourteen = clone
        .ui
        .frame()
        .items
        .iter()
        .find_map(|it| match &it.cmd {
            DrawCmd::Text { text, bounds, .. } if text == "fourteen" => Some(*bounds),
            _ => None,
        })
        .expect("the second piece's first line");
    assert_eq!((fourteen.x, fourteen.y), (85.0, 5.0));
}

#[test]
fn border_shape_strokes_one_shape_and_fills_between_two() {
    // CSS Borders 4 §7: the spec's own examples. One shape is stroked
    // along its path over the half-border box; two fill the border
    // between the outer (over the border box) and the inner (over the
    // padding box). The outer shape is the background's; the corner
    // radius is ignored; layout is unchanged.
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Circle"></div>
  <div id="Diamond"></div>
  <div id="Side"></div>
  <div id="Rtl" dir="rtl"><div id="Inner"></div></div>
</div></div>
<style>
#Root { width: 400px; }
#Circle { width: 100px; height: 100px; border: 5px solid #0000ff; border-shape: circle(50%); }
#Diamond { width: 200px; height: 200px; border: 10px solid #dc143c; border-radius: 30px;
  background: #ff0000;
  border-shape: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%) polygon(50% 10%, 90% 50%, 50% 90%, 10% 50%); }
#Side, #Inner { width: 40px; height: 40px; border-style: none solid solid solid;
  border-width: 1px 2px 3px 4px; border-color: #ff00ff #00ff00 #00ffff #ffff00;
  border-shape: inset(0); }
#Inner { border-width: 1px 6px 3px 8px; }
</style>"#;
    let d = Doc::with_font(text, &[], test_font_metrics());
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    // Layout is the border's as ever.
    assert_eq!(
        (d.get("Circle").width, d.get("Diamond").height),
        (100.0, 200.0)
    );
    // Each shape drawn in `colour`: its geometry, mode and centre.
    let shapes = |c: Srgba| -> Vec<(og_draw::UiGeometry, og_draw::UiMode, [f32; 2])> {
        d.ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. }
                    if matches!(shape.paint, og_draw::UiPaint::Solid(p) if same(p, c)) =>
                {
                    Some((shape.geometry.clone(), shape.mode, shape.center.into()))
                }
                _ => None,
            })
            .collect()
    };
    // A filled path's contours, flattened, from the box's centre.
    let contours = |g: &og_draw::UiGeometry| match g {
        og_draw::UiGeometry::Path(p) => og_draw::polylines(p.path.iter(), 0.1)
            .into_iter()
            .map(|(pts, _)| pts.iter().map(|v| [v.x, v.y]).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        g => panic!("not a path: {g:?}"),
    };
    let has = |pts: &[[f32; 2]], want: [f32; 2]| {
        pts.iter()
            .any(|p| (p[0] - want[0]).abs() < 0.01 && (p[1] - want[1]).abs() < 0.01)
    };

    // Stroke mode: `circle(50%)` over the half-border box (100px in by
    // 2.5px each side: 95px), radius 50% of its normalised diagonal, so
    // 47.5px round the box's centre, the line 5px wide and centred on it.
    let circle = d.get("Circle");
    let blue = shapes(Srgba::rgb(0, 0, 255));
    assert_eq!(blue.len(), 1, "one stroked shape, no rectangle ring");
    let (g, mode, centre) = &blue[0];
    assert_eq!(
        *centre,
        [circle.x + 50.0, circle.y + 50.0],
        "centred on its box"
    );
    match mode {
        og_draw::UiMode::Stroke(s) => {
            assert_eq!((s.width, s.align), (5.0, og_draw::UiAlign::Center))
        }
        m => panic!("a stroke, not {m:?}"),
    }
    let og_draw::UiGeometry::Polyline { points, closed, .. } = g else {
        panic!("a line along the circle, not {g:?}");
    };
    assert!(*closed && points.len() > 16);
    for p in points {
        assert!((p.length() - 47.5).abs() < 0.3, "{p:?} is off the circle");
    }

    // Fill mode: the diamond over the 200px border box and the inner one
    // over the 180px padding box 10px in, the border between them
    // (even-odd), and the background in the outer diamond, not the
    // rounded rectangle.
    let crimson = shapes(Srgba::rgb(220, 20, 60));
    assert_eq!(crimson.len(), 1, "one filled border, no rectangle ring");
    let (g, mode, _) = &crimson[0];
    assert_eq!(*mode, og_draw::UiMode::Fill);
    let og_draw::UiGeometry::Path(p) = g else {
        panic!("a path, not {g:?}")
    };
    assert_eq!(p.fill_rule, og_draw::UiFillRule::EvenOdd);
    let c = contours(g);
    assert_eq!(c.len(), 2, "outer and inner contours: {c:?}");
    for want in [[0.0, -100.0], [100.0, 0.0], [0.0, 100.0], [-100.0, 0.0]] {
        assert!(has(&c[0], want), "outer {want:?} in {:?}", c[0]);
    }
    // 50% 10% of the padding box: 10 + 90, 10 + 18 from the top left.
    for want in [[0.0, -72.0], [72.0, 0.0], [0.0, 72.0], [-72.0, 0.0]] {
        assert!(has(&c[1], want), "inner {want:?} in {:?}", c[1]);
    }
    let red = shapes(Srgba::rgb(255, 0, 0));
    assert_eq!(red.len(), 1, "the background: {red:?}");
    let bg = contours(&red[0].0);
    for want in [[0.0, -100.0], [100.0, 0.0], [0.0, 100.0], [-100.0, 0.0]] {
        assert!(has(&bg[0], want), "background {want:?} in {:?}", bg[0]);
    }

    // §7.6: the first side with a style of block-start, inline-start,
    // block-end, inline-end. The top's is none, so left to right the
    // left's (4px yellow), right to left the right's (6px green).
    let width = |c: Srgba| -> Vec<f32> {
        shapes(c)
            .iter()
            .map(|(_, m, _)| match m {
                og_draw::UiMode::Stroke(s) => s.width,
                m => panic!("{m:?}"),
            })
            .collect()
    };
    assert_eq!(width(Srgba::rgb(255, 255, 0)), [4.0], "left to right: left");
    assert_eq!(width(Srgba::rgb(0, 255, 0)), [6.0], "right to left: right");
}

#[test]
fn glass_takes_any_backdrop_filter_chain() {
    // CSS Filter Effects 2 §3: backdrop-filter's functions run in order
    // over what is under the box. Glass folds blurs then colour into its
    // own backdrop; any other chain (here a colour function before a
    // blur and a drop-shadow) runs as the staged backdrop filter, in
    // order, and the glass draws over what it leaves without filtering
    // again. Ways it could fail: the chain refused with a warning and no
    // filter drawn, its functions reordered, or the glass blurring twice.
    let text = r#"<meta name="component" content="t">
<div><div id="Root">
  <div id="Fold"></div>
  <div id="Chain"></div>
</div></div>
<style>
#Root { width: 400px; }
#Fold, #Chain { width: 100px; height: 60px; -og-material: glass regular lift; }
#Fold { backdrop-filter: blur(4px) saturate(0); }
#Chain { margin-top: 10px; backdrop-filter: saturate(0) blur(4px) drop-shadow(2px 3px 4px #000000); }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.set_part_css(&format!(
        "{DEFAULT_CSS}\n:root {{ --glass-backdrop: auto; }}"
    ))
    .unwrap();
    let d = Doc::mounted(ui, text, &[]);
    assert!(
        d.ui.frame().warnings.is_empty(),
        "{:?}",
        d.ui.frame().warnings
    );
    let rect = |id: &str| {
        let e = d.get(id);
        Rect::new(e.x, e.y, e.width, e.height)
    };
    let (fold, chain) = (rect("Fold"), rect("Chain"));
    let within = |y: f32, r: &Rect| y >= r.y && y <= r.y + r.height;
    // Each box's backdrop layers' filters and glass backdrops, in order.
    #[derive(Debug)]
    enum Drawn {
        Staged(Vec<og_draw::UiFilterOp>),
        Glass(f32, f32, bool),
    }
    let drawn = |r: &Rect| -> Vec<Drawn> {
        let mut out = Vec::new();
        for it in &d.ui.frame().items {
            match &it.cmd {
                DrawCmd::BeginLayer {
                    role: LayerRole::Backdrop,
                    filter: Some(f),
                    ..
                } if within(f.0.y + f.0.height / 2.0, r) => out.push(Drawn::Staged(
                    f.1.primitives.iter().map(|p| p.op.clone()).collect(),
                )),
                DrawCmd::Shape { shape, .. } if within(shape.center.y, r) => {
                    if let og_draw::UiMode::Backdrop {
                        blur,
                        saturation,
                        glass,
                        ..
                    } = shape.mode
                    {
                        out.push(Drawn::Glass(blur, saturation, glass.matrix.is_some()));
                    }
                }
                _ => {}
            }
        }
        out
    };
    // Folded: one glass backdrop, the CSS blur at the glass scale, the
    // grey matrix in it and no staged layer.
    let f = drawn(&fold);
    assert!(
        matches!(f.as_slice(), [Drawn::Glass(b, 1.0, true)] if *b == 4.0 * og_draw::ui::GLASS_BLUR_SCALE),
        "{f:?}"
    );
    // Staged: saturate(0) is Filter Effects 1's matrix at s = 0 (each
    // row 0.213 0.715 0.072), then the 4px blur, then the shadow: its
    // 4px blur is a deviation of 2, moved 2 3, black, in under the image.
    let c = drawn(&chain);
    let [Drawn::Staged(ops), Drawn::Glass(0.0, 1.0, false)] = c.as_slice() else {
        panic!("a staged filter, then glass with no filter of its own: {c:?}");
    };
    use og_draw::UiFilterOp as Op;
    let Op::ColorMatrix(m) = &ops[0] else {
        panic!("{ops:?}")
    };
    for row in 0..3 {
        assert_eq!(m[row * 5..row * 5 + 3], [0.213, 0.715, 0.072], "{m:?}");
    }
    assert!(
        matches!(
            ops[1],
            Op::Blur {
                std_dev: [4.0, 4.0]
            }
        ),
        "{ops:?}"
    );
    assert!(
        matches!(
            ops[2],
            Op::Blur {
                std_dev: [2.0, 2.0]
            }
        ),
        "{ops:?}"
    );
    assert!(matches!(ops[3], Op::Offset { d: [2.0, 3.0] }), "{ops:?}");
    assert!(
        matches!(ops[4], Op::Flood { color } if color == [0.0, 0.0, 0.0, 1.0]),
        "{ops:?}"
    );
    assert!(
        matches!(ops[5], Op::Composite { .. }) && matches!(ops[6], Op::Merge),
        "{ops:?}"
    );
    assert_eq!(ops.len(), 7);
}

#[test]
fn backdrop_sampling_is_opt_in_and_reduced_transparency_wins_live() {
    let mut d = Doc::with_font(
        r#"<meta name="component" content="t">
<div><div id="Glass"></div><div id="Filter"></div></div>
<style>
#Glass, #Filter { width: 100px; height: 60px; }
#Glass { -og-material: glass thick lift; }
#Filter { backdrop-filter: saturate(0) blur(4px); background: #123456; }
@media (-og-glass-backdrop: auto) { #Glass { width: 120px; } }
</style>"#,
        &[],
        test_font_metrics(),
    );
    let sampling = |ui: &Ui| {
        ui.frame()
            .items
            .iter()
            .filter(|item| {
                matches!(&item.cmd,
        DrawCmd::Shape { shape, .. } if matches!(shape.mode, og_draw::UiMode::Backdrop { .. }))
                    || matches!(
                        &item.cmd,
                        DrawCmd::BeginLayer {
                            role: LayerRole::Backdrop,
                            ..
                        }
                    )
            })
            .count()
    };
    assert_eq!(
        sampling(&d.ui),
        0,
        "default off suppresses materials and standalone filters"
    );
    assert_eq!(d.get("Glass").width, 100.0);
    assert!(
        d.ui.frame().items.iter().any(|item| matches!(&item.cmd,
        DrawCmd::Shape { shape, .. } if shape.center.y < 60.0
            && matches!(shape.mode, og_draw::UiMode::Fill)
            && matches!(shape.paint, og_draw::UiPaint::Solid(c) if c[3] >= 0.97))),
        "material keeps its opaque ground"
    );
    for (reduced, enabled, width) in [
        (false, true, 120.0),
        (true, false, 100.0),
        (false, true, 120.0),
    ] {
        d.ui.set_theme(Theme {
            glass_backdrop: Some(GlassBackdrop::Auto),
            reduced_transparency: reduced,
            ..Theme::dark()
        });
        d.step(InputFrame::default());
        assert_eq!(sampling(&d.ui) > 0, enabled);
        assert_eq!(
            d.get("Glass").width,
            width,
            "retained media refreshes immediately"
        );
    }
    for reduced in [true, false] {
        d.viewport.reduced_transparency = reduced;
        d.step(InputFrame::default());
        assert_eq!(sampling(&d.ui) > 0, !reduced, "host preference wins too");
        assert_eq!(d.get("Glass").width, if reduced { 100.0 } else { 120.0 });
    }
    d.ui.set_theme(Theme {
        glass_backdrop: Some(GlassBackdrop::None),
        ..Theme::dark()
    });
    d.step(InputFrame::default());
    assert_eq!(sampling(&d.ui), 0);
    assert_eq!(d.get("Glass").width, 100.0);
}

/// CSS Color HDR 1 `dynamic-range-limit` on a document's root is the
/// frame's: the peak the display pass ends it at, worked on a display with
/// 8× headroom (3 stops). `standard` is SDR white (1×), `constrained` one
/// stop (2×), `no-limit` and no declaration the display's 8×. A mix is its
/// computed weights (§3.3): the spec's nested example, normalized from 10,
/// 20 and 20 %, is standard 10 %, constrained 40 %, no-limit 50 %, so
/// 0.4 × 1 + 0.5 × 3 = 1.9 stops, 2^1.9 = 3.73×. A value with no
/// percentage, or one over 100 %, is refused.
///
/// Ways it could fail: the mix's percentages not normalized (the example
/// summing to 50 % would end at 2^0.95), a nested mix read as a keyword,
/// the limit read from a child rather than the root, or mixed in linear
/// luminance rather than stops.
#[test]
fn dynamic_range_limit_sets_the_frames_peak() {
    let nested = "dynamic-range-limit-mix(no-limit 10%, dynamic-range-limit-mix(standard 25%, constrained 75%) 20%, dynamic-range-limit-mix(constrained 10%, no-limit 30%) 20%)";
    for (root, child, want) in [
        ("", "standard", 8.0),
        ("standard", "no-limit", 1.0),
        ("constrained", "", 2.0),
        ("no-limit", "", 8.0),
        (
            "dynamic-range-limit-mix(standard 50%, no-limit 50%)",
            "",
            2f32.powf(1.5),
        ),
        (nested, "", 2f32.powf(1.9)),
    ] {
        let decl = |v: &str| {
            if v.is_empty() {
                String::new()
            } else {
                format!("dynamic-range-limit: {v};")
            }
        };
        let text = format!(
            "<meta name=\"component\" content=\"t\">\n<column id=\"Root\"><div id=\"Kid\"></div></column>\n<style>\n#Root {{ {} }}\n#Kid {{ {} }}\n</style>",
            decl(root),
            decl(child)
        );
        let d = Doc::new(&text, &[]);
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        let peak = d.ui.range_limit().peak(8.0);
        assert!(
            (peak - want).abs() < 1e-3,
            "{root} / {child}: {peak}, not {want}"
        );
    }
    for bad in [
        "dynamic-range-limit-mix(standard, no-limit 50%)",
        "dynamic-range-limit-mix(standard 150%)",
        "high",
    ] {
        let text = format!(
            "<meta name=\"component\" content=\"t\">\n<column id=\"Root\"></column>\n<style>#Root {{ dynamic-range-limit: {bad}; }}</style>"
        );
        let e = UiDocument::parse("ui/t.svelte", &text).err();
        assert!(
            e.is_some_and(|e| format!("{e:?}").contains("dynamic-range-limit")),
            "{bad}"
        );
    }
}

/// The frame's `dynamic-range-limit` follows the roots as they change and
/// as documents mount: a bound class moving the root between `no-limit`
/// and `standard` moves the 8x display's peak between 8x and 1x, and a
/// second mounted document whose root asks for `constrained` holds the
/// frame at 2x beside the first's `no-limit` (the most limiting root).
///
/// Ways it could fail: the limit read once at mount, a class change not
/// restyling it, or the last mounted root, not the most limiting, ruling.
#[test]
fn dynamic_range_limit_follows_its_roots() {
    let text = "<meta name=\"component\" content=\"t\">\n<column id=\"Root\" class={$c}></column>\n<style>.std { dynamic-range-limit: standard; } .free { dynamic-range-limit: no-limit; }</style>";
    let mut d = Doc::new(text, &[("c", serde_json::json!("free"))]);
    let peak = |d: &Doc| d.ui.range_limit().peak(8.0);
    assert_eq!(peak(&d), 8.0, "no-limit");
    d.set("c", serde_json::json!("std"));
    assert_eq!(peak(&d), 1.0, "standard after the class changed");
    d.set("c", serde_json::json!("free"));
    assert_eq!(peak(&d), 8.0, "no-limit again");
    let hud = d
        .ui
        .parse_document(
            "ui/hud.svelte",
            "<meta name=\"component\" content=\"hud\">\n<column id=\"Cap\"></column>\n<style>#Cap { dynamic-range-limit: constrained; }</style>",
            Vec::new(),
        )
        .unwrap();
    d.ui.define_document(hud);
    d.ui.mount_document(
        reactive::SourceId::new("test", "hud"),
        "hud",
        Default::default(),
        Default::default(),
        &bind::NoEnv,
    )
    .unwrap();
    d.step(InputFrame::default());
    assert_eq!(peak(&d), 2.0, "the HUD's constrained holds the frame");
}

#[test]
fn inline_boxes_draw_on_their_line_fragments() {
    // CSS 2 §10.8 and CSS Fragmentation 3 §5.4: an inline element's
    // background and border draw on each line fragment, under its words.
    // With 20 px Noto Sans SC (each ideograph 20 px), あい日本東京中漢です
    // in 105 px breaks after 東: the span 日本東京中漢, its 2 px border
    // taking room at its edges, has fragments (border boxes) from 40 to
    // 102 on the first line and 0 to 62 on the second under `slice`, the
    // initial value, each around its words' content area (ascent 1.160
    // em, descent 0.288 em); each draws the box as if unbroken (124 px
    // long), moved so its part shows and cut at the break, so the
    // border's start edge is drawn on the first line only and its end
    // edge on the last. Under `clone` each line holds both edges (40 to
    // 104, 0 to 64) and each fragment draws its own whole box. A label change outside the text keeps the boxes as a whole
    // build draws them (run with OG_UI_CHECK_BOUNDED=1 too).
    //
    // Ways it could fail: one box over the union of the lines, or none;
    // the box over the words; both edges on every fragment under slice,
    // or the unbroken box not continued across the break; the right to
    // left line's box continued the wrong way.
    const SC: &str = "tests/fixtures/ui-text/assets/fonts/NotoSansSC-Regular-subset.ttf";
    let doc = |mode: &str, dir: &str, css: &str| {
        let text = format!(
            r#"<meta name="component" content="t">
<meta name="prop:label" content="string">
<div><div id="Root">
  <p id="P" dir="{dir}">あい<span id="S">日本東京中漢</span>です</p>
  <p id="Other">{{label}}</p>
</div></div>
<style>
#Root {{ width: 400px; }}
p {{ margin: 0; font-size: 20px; width: 105px; }}
#S {{ background: #ff0000; border: 2px solid #00ff00; box-decoration-break: {mode}; }}
{css}
</style>"#
        );
        let mut font = crate::features::font_at(SC);
        font.handle = test_font().handle;
        let d = Doc::with_font(&text, &[("label", serde_json::json!("one"))], font);
        assert!(
            d.ui.frame().warnings.is_empty(),
            "{:?}",
            d.ui.frame().warnings
        );
        d
    };
    // Each red background and green border: its box, and what of it the
    // clips it is drawn under leave; and where the words are drawn.
    let drawn = |d: &Doc| {
        let mut clips: Vec<Rect> = Vec::new();
        let mut out = Vec::new();
        let mut words = None;
        for (k, it) in d.ui.frame().items.iter().enumerate() {
            let (rect, green) = match &it.cmd {
                DrawCmd::PushClip { center, half, .. } => {
                    clips.push(it.transform.map_rect(&Rect::new(
                        center.0 - half.0,
                        center.1 - half.1,
                        2.0 * half.0,
                        2.0 * half.1,
                    )));
                    continue;
                }
                DrawCmd::PopClip => {
                    clips.pop();
                    continue;
                }
                DrawCmd::Text { text, .. } if text.contains('日') => {
                    words.get_or_insert(k);
                    continue;
                }
                DrawCmd::Shape { shape, .. } => {
                    let og_draw::UiPaint::Solid(c) = shape.paint else {
                        continue;
                    };
                    let green = same(c, Srgba::rgb(0, 255, 0));
                    if !green && !same(c, Srgba::rgb(255, 0, 0)) {
                        continue;
                    }
                    (
                        Rect::new(
                            shape.center.x - shape.half.x,
                            shape.center.y - shape.half.y,
                            2.0 * shape.half.x,
                            2.0 * shape.half.y,
                        ),
                        green,
                    )
                }
                _ => continue,
            };
            let rect = it.transform.map_rect(&rect);
            let shown = clips.iter().fold(rect, |a, c| a.intersect(c));
            let q = |r: Rect| [r.x, r.y, r.width, r.height].map(|v| (v * 100.0).round() / 100.0);
            out.push((green, q(rect), q(shown), k));
        }
        (out, words.expect("the span's words"))
    };
    let r = |x: f32, y: f32, w: f32, h: f32| [x, y, w, h].map(|v| (v * 100.0).round() / 100.0);

    let slice = doc("slice", "ltr", "");
    let p = slice.get("P");
    let (boxes, words) = drawn(&slice);
    assert!(
        boxes.iter().all(|b| b.3 < words),
        "under the words: {boxes:?} {words}"
    );
    let strip = |b: Vec<(bool, [f32; 4], [f32; 4], usize)>| -> Vec<(bool, [f32; 4], [f32; 4])> {
        b.into_iter().map(|(g, a, s, _)| (g, a, s)).collect()
    };
    // Clone: a whole box on each fragment, 40 to 104 then 0 to 64. Across
    // the line each is its line's fragment (its words' content area): the
    // second line's below the first's, each around its words' baseline.
    let clone = strip(drawn(&doc("clone", "ltr", "")).0);
    assert_eq!(clone.len(), 4, "{clone:?}");
    let (first, second) = (clone[0].1, clone[2].1);
    assert_eq!(
        clone,
        [
            (false, first, first),
            (true, first, first),
            (false, second, second),
            (true, second, second),
        ],
        "clone: a whole box on each fragment"
    );
    assert_eq!(
        [first[0], first[2], second[0], second[2]],
        [p.x + 40.0, 64.0, p.x, 64.0]
    );
    // Across the line: its words' content area (1.448 em) and its 2 px
    // border above and below, which may overflow its line without growing
    // it (CSS 2 §10.6.1); the second a line below the first.
    let line = p.height / 2.0;
    assert!(
        (first[3] - (1.448 * 20.0 + 4.0)).abs() < 0.1
            && second[3] == first[3]
            && (second[1] - first[1] - line).abs() < 0.5
            && first[1] < p.y + line / 2.0
            && first[1] + first[3] > p.y + line / 2.0,
        "{first:?} {second:?} in {p:?}"
    );
    // Slice: the 124 px box from 40 on the first line, from -62 on the
    // second, each showing its fragment along the line (its block sides
    // and the edge it holds not cut).
    let unbroken = |f: [f32; 4], x: f32, w: f32| r(x, f[1], w, f[3]);
    let (one, two) = (
        r(p.x + 40.0, first[1], 62.0, first[3]),
        r(p.x, second[1], 62.0, second[3]),
    );
    let (a, b) = (
        unbroken(first, p.x + 40.0, 124.0),
        unbroken(second, p.x - 62.0, 124.0),
    );
    assert_eq!(
        strip(boxes),
        [
            (false, a, one),
            (true, a, one),
            (false, b, two),
            (true, b, two),
        ],
        "slice"
    );

    // Right to left: the ideographs are a left to right run, so the
    // fragments stay where they were, but the element's start edge is at
    // its right: the unbroken background runs on leftward from the first
    // fragment's right edge (-20 to 100) and the second shows its left
    // part (0 to 120). No border here: where a wrapped element's words run
    // against its direction, its edges' room follows its words (its left
    // edge on its first line), not yet CSS 2 §8.6's lines.
    let rtl = strip(drawn(&doc("slice", "rtl", "#S { border: none; }")).0);
    let (one, two) = (
        r(p.x + 40.0, first[1] + 2.0, 60.0, first[3] - 4.0),
        r(p.x, second[1] + 2.0, 60.0, second[3] - 4.0),
    );
    let (a, b) = (unbroken(one, p.x - 20.0, 120.0), unbroken(two, p.x, 120.0));
    assert_eq!(
        rtl,
        [(false, a, one), (false, b, two)],
        "right to left slice"
    );

    // A change outside the text: the same boxes.
    let mut slice = slice;
    let before = drawn(&slice).0;
    slice.set("label", serde_json::json!("two"));
    assert_eq!(drawn(&slice).0.len(), before.len());
    assert_eq!(
        drawn(&slice)
            .0
            .iter()
            .map(|b| (b.0, b.1, b.2))
            .collect::<Vec<_>>(),
        before.iter().map(|b| (b.0, b.1, b.2)).collect::<Vec<_>>()
    );
}
