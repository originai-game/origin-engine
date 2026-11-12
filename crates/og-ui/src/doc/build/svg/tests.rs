use crate::doc::UiDocument;
use crate::doc::build::DocsState;
use crate::doc::build::ext::*;
use crate::reactive::SourceId;
use crate::reactive::session::Command;
use crate::{DrawCmd, InputFrame, Ui, UiDesc, Viewport};
use serde_json::json;
use std::collections::BTreeMap;

const RING: &str = r##"
<meta name="component" content="ring">
<div id="Box">
  <svg id="Ring" viewBox="0 0 40 40" style:--s={$size}>
<g id="G" stroke="#ff0000">
  <circle id="Track" cx="20" cy="20" r="18" fill="none" stroke-width="1"/>
  <circle id="Fill" cx="20" cy="20" r="18" fill="none" pathLength="1"
    stroke-dasharray="1 1" stroke-dashoffset={1 - $ready}/>
</g>
<rect id="Bar" x="0" y={40 - $h} width="8" height={$h} fill="url(#Grad)"/>
<linearGradient id="Grad"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#fff"/></linearGradient>
  </svg>
</div>
<style>#Ring { width: calc(var(--s) * 1px); height: calc(var(--s) * 1px); }</style>
"##;

/// The frame's stroke widths and whether a gradient fill drew.
fn strokes(ui: &Ui) -> Vec<f32> {
    ui.frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } => match shape.mode {
                og_draw::UiMode::Stroke(s) => Some(s.width),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// An empty meter's `stroke-dasharray="0 1"` draws nothing with butt
/// caps, as a browser does; a round cap's zero dashes are dots. Fails
/// when the zero dash reaches og-render, which reads it as solid (the
/// radial meter's empty wheels drew full rings).
#[test]
fn zero_length_dashes_draw_no_ring() {
    let ring = |cap: &str| {
        let doc = format!(
            r##"<meta name="component" content="z">
<svg id="S" viewBox="0 0 40 40" style="width: 80px; height: 80px">
  <circle cx="20" cy="20" r="18" fill="none" stroke="#f00" stroke-width="4"
stroke-linecap="{cap}" pathLength="1" stroke-dasharray="0 1"/>
</svg>"##
        );
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(UiDocument::parse("z.svelte", &doc).unwrap());
        ui.mount_document(
            SourceId::new("test", "z"),
            "z",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        ui.begin(InputFrame::default(), Viewport::new(200.0, 200.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
        strokes(&ui).len()
    };
    assert_eq!(ring("butt"), 0, "butt caps: nothing");
    assert!(ring("round") > 0, "round caps: a dot");
}

#[test]
fn inline_svg_shapes_bind_attributes_and_repaint_only_what_changed() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("ring.svelte", RING).unwrap());
    let mut inputs = BTreeMap::new();
    for (name, v) in [("ready", 0.25), ("h", 10.0), ("size", 80.0)] {
        let source = SourceId::new("test", name);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: json!(v),
                readable: false,
            })
            .unwrap();
        inputs.insert(name.to_string(), source);
    }
    ui.mount_document(
        SourceId::new("test", "ring"),
        "ring",
        inputs,
        BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    let tick = |ui: &mut Ui, w: f32| {
        ui.begin(InputFrame::default(), Viewport::new(w, 400.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end().stats
    };
    for _ in 0..3 {
        tick(&mut ui, 400.0);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // A 1-unit stroke in a 2× box is 2 px, as in browsers.
    let widths = strokes(&ui);
    assert_eq!(widths.len(), 2, "{widths:?}");
    assert!(widths.iter().all(|w| (w - 2.0).abs() < 1e-4), "{widths:?}");
    let gradient = ui.frame().items.iter().any(|it| {
        matches!(&it.cmd, DrawCmd::Shape { shape, .. }
            if matches!(shape.paint, og_draw::UiPaint::Gradient(_)))
    });
    assert!(gradient, "the bar's url(#Grad) fill draws");
    let idle = tick(&mut ui, 400.0);
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    // One bound attribute: that shape alone (the store and the
    // expression `1 - $ready` are its two evaluations).
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Set {
            source: SourceId::new("test", "ready"),
            value: json!(0.75),
            force: false,
        })
        .unwrap();
    let changed = tick(&mut ui, 400.0);
    assert_eq!(
        (changed.bindings, changed.reconciled),
        (2, 1),
        "{changed:?}"
    );
    // The ring's dash offset is a quarter of its circumference, in px.
    let offset = ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::Shape { shape, .. } => match shape.mode {
            og_draw::UiMode::Stroke(s) if s.dash[0] > 0.0 => Some((s.dash, s.dash_offset)),
            _ => None,
        },
        _ => None,
    });
    let (dash, off) = offset.expect("the dashed ring");
    let circumference = 2.0 * std::f32::consts::PI * 18.0 * 2.0;
    assert!((dash[0] - circumference).abs() < 0.5, "{dash:?}");
    assert!((off - 0.25 * circumference).abs() < 0.5, "{off}");
    // Only the box changes: its shapes' attributes stay, their draws
    // are placed in the new box (a 1-unit stroke at 3× is 3 px).
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Set {
            source: SourceId::new("test", "size"),
            value: json!(120.0),
            force: false,
        })
        .unwrap();
    for _ in 0..3 {
        tick(&mut ui, 400.0);
    }
    let widths = strokes(&ui);
    assert_eq!(widths.len(), 2, "{widths:?}");
    assert!(widths.iter().all(|w| (w - 3.0).abs() < 1e-4), "{widths:?}");
    let idle = tick(&mut ui, 400.0);
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
}

const DEFS: &str = r##"
<meta name="component" content="icons">
<div id="Box" style:--s={$size} style:--m={$margin}>
  <svg id="Icons" viewBox="0 0 100 40">
<title>{$label}</title>
<defs>
  <symbol id="Coin" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4" fill="#f0b429"/></symbol>
  <clipPath id="Half"><rect x="40" y="0" width="10" height="40"/></clipPath>
  <mask id="Fade"><rect x="60" y="0" width="20" height="40" fill="url(#Ramp)"/></mask>
  <linearGradient id="Ramp"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#000"/></linearGradient>
  <pattern id="Dots" patternUnits="userSpaceOnUse" width="5" height="5"><circle cx="2.5" cy="2.5" r="1" fill="#fff"/></pattern>
  <clipPath id="Wedge"><path d="M82 22 L98 22 Q90 30 82 38 Z"/></clipPath>
</defs>
<use id="A" href="#Coin" x={$x} y="0" width="20" height="20"/>
<use id="B" href="#Coin" x="20" y="0" width="10" height="10" fill="#000"/>
<g id="Clipped" clip-path="url(#Half)"><circle id="Big" cx="45" cy="20" r="20" fill="#f00"/></g>
<g id="Masked" mask="url(#Fade)"><rect id="Under" x="60" y="0" width="20" height="40" fill="#0f0"/></g>
<rect id="Dotted" x="80" y="0" width="20" height="20" fill="url(#Dots)"/>
<g id="Wedged" clip-path="url(#Wedge)"><circle id="Disc" cx="90" cy="30" r="8" fill="#00f"/></g>
  </svg>
</div>
<style>#Box { padding: calc(var(--m) * 1px); } #Icons { width: calc(var(--s) * 1px); height: calc(var(--s) * 0.4px); }</style>
"##;

/// `<use>` of a `<symbol>`, `clip-path` (a rect and a path), a gradient `<mask>`, a
/// pattern fill and `<title>`: each draws through the native draw, a
/// bound attribute of a copy costs that copy, and moving or resizing
/// the `<svg>` builds nothing.
///
/// Ways it could fail: a use drawn at the symbol's own size, or not
/// inheriting the use's fill; a clip that cuts nothing or the unfitted
/// shape; a mask gradient left in view-box units; tiles that miss the
/// shape's edge; a title drawn as text instead of naming the svg.
#[test]
fn svg_definitions_draw_and_cost_only_what_changed() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("icons.svelte", DEFS).unwrap());
    let mut inputs = BTreeMap::new();
    for (name, v) in [
        ("x", json!(0)),
        ("label", json!("Purse")),
        ("size", json!(200)),
        ("margin", json!(0)),
    ] {
        let source = SourceId::new("test", name);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: v,
                readable: false,
            })
            .unwrap();
        inputs.insert(name.to_string(), source);
    }
    ui.mount_document(
        SourceId::new("test", "icons"),
        "icons",
        inputs,
        BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    let tick = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end().stats
    };
    for _ in 0..3 {
        tick(&mut ui);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let shapes = |ui: &Ui| -> Vec<og_draw::UiShape> {
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => Some((**shape).clone()),
                _ => None,
            })
            .collect()
    };
    let all = shapes(&ui);
    // The coins: one filled with the symbol's gold at 2× (20 units in
    // a 2 px/unit box: 40 px wide), the other black (the use's fill
    // does not reach a child that sets its own: still gold).
    let gold = all.iter().filter(|s| matches!(&s.paint, og_draw::UiPaint::Solid(c) if c[0] > 0.8 && c[2] < 0.1 && c[1] > 0.3)).count();
    assert_eq!(gold, 2, "both coins draw the symbol's gold");
    let clipped = ui
        .frame()
        .items
        .iter()
        .any(|it| matches!(it.cmd, DrawCmd::PushClip { .. }));
    assert!(clipped, "clip-path pushes a clip");
    // A clip to a path cuts by the path (no warning, above).
    let path_clip = ui.frame().items.iter().any(|it| {
        matches!(
            &it.cmd,
            DrawCmd::PushClip {
                geometry: og_draw::UiGeometry::Path(_),
                ..
            }
        )
    });
    assert!(path_clip, "the wedge clips to its path");
    let masked = ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::BeginLayer {
            mask: Some(og_draw::UiMask::Gradient(g)),
            ..
        } => Some(g.clone()),
        _ => None,
    });
    let g = masked.expect("a gradient mask");
    // x 60..80 of 100 units: 0.6..0.8 of the box.
    assert!(
        (g.from.x - 0.6).abs() < 1e-3 && (g.to.x - 0.8).abs() < 1e-3,
        "{g:?}"
    );
    let dots = all.iter().find(|s| matches!(&s.geometry, og_draw::UiGeometry::Path(p) if p.verbs.iter().filter(|v| matches!(v, og_draw::UiVerb::Move)).count() == 16));
    assert!(dots.is_some(), "16 dot tiles over the 20×20 rect");
    let name = |ui: &Ui| {
        ui.a11y()
            .nodes
            .iter()
            .find(|a| a.path == "test:icons/Icons")
            .map(|a| (a.role.clone(), a.name.clone()))
    };
    assert_eq!(name(&ui), Some(("image".into(), "Purse".into())));
    let idle = tick(&mut ui);
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    let set = |ui: &mut Ui, n: &str, v: serde_json::Value| {
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: SourceId::new("test", n),
                value: v,
                force: false,
            })
            .unwrap();
    };
    // A use's bound x: the use and its copy, nothing else.
    set(&mut ui, "x", json!(5));
    let moved = tick(&mut ui);
    assert_eq!(moved.reconciled, 2, "{moved:?}");
    // The title: one binding; the svg (and the defs it re-seeds), not
    // the shapes.
    set(&mut ui, "label", json!("Wallet"));
    let named = tick(&mut ui);
    assert!(named.bindings == 1 && named.reconciled <= 2, "{named:?}");
    assert_eq!(name(&ui).unwrap().1, "Wallet");
    // The svg moves without resizing, then resizes: no binding of a
    // shape is evaluated, and both settle.
    for (n, v) in [("margin", json!(30)), ("size", json!(300))] {
        set(&mut ui, n, v);
        for _ in 0..3 {
            tick(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{n}: {idle:?}");
    }
    let g = ui
        .frame()
        .items
        .iter()
        .find_map(|it| match &it.cmd {
            DrawCmd::BeginLayer {
                mask: Some(og_draw::UiMask::Gradient(g)),
                ..
            } => Some(g.clone()),
            _ => None,
        })
        .unwrap();
    assert!((g.from.x - 0.6).abs() < 1e-3, "the mask refits: {g:?}");
}

const TEXT: &str = r##"
<meta name="component" content="chart">
<div id="Box" style:--s={$size} style:--m={$margin}>
  <svg id="Chart" viewBox="0 0 100 50">
<text id="Mid" x="50" y="25" text-anchor="middle" dominant-baseline="central" font-size="10">{$label}</text>
<text id="End" x={$x} y="40" text-anchor="end" font-size="8">Max <tspan font-weight="bold" fill="#f00">99</tspan></text>
  </svg>
</div>
<style>#Box { padding: calc(var(--m) * 1px); } #Chart { width: calc(var(--s) * 1px); height: calc(var(--s) * 0.5px); }</style>
"##;

/// SVG `<text>`: its anchor on the point fitted into the `<svg>`'s box,
/// scaled with it (hit and accessibility bounds too), right to left
/// where its view box shows it; a bound `x` costs that text, and moving
/// or resizing the `<svg>` builds nothing.
///
/// Ways it could fail: the text left at its unscaled size or place; a
/// start-anchored rather than end-anchored run; bounds of the unscaled
/// box; mirrored inside the view box, or scaled twice.
#[test]
fn svg_text_sits_on_its_point_and_scales_with_the_view_box() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("chart.svelte", TEXT).unwrap());
    let mut inputs = BTreeMap::new();
    for (name, v) in [
        ("x", json!(90)),
        ("label", json!("Gold")),
        ("size", json!(200)),
        ("margin", json!(0)),
    ] {
        let source = SourceId::new("test", name);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial: v,
                readable: false,
            })
            .unwrap();
        inputs.insert(name.to_string(), source);
    }
    ui.mount_document(
        SourceId::new("test", "chart"),
        "chart",
        inputs,
        BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    let tick = |ui: &mut Ui| {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end().stats
    };
    let set = |ui: &mut Ui, n: &str, v: serde_json::Value| {
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Set {
                source: SourceId::new("test", n),
                value: v,
                force: false,
            })
            .unwrap();
    };
    let rect = |ui: &Ui, id: &str| {
        let a = ui
            .a11y()
            .nodes
            .iter()
            .find(|a| a.path == format!("test:chart/Chart/{id}"))
            .unwrap_or_else(|| {
                panic!(
                    "{id} in the a11y tree: {:?}",
                    ui.a11y().nodes.iter().map(|a| &a.path).collect::<Vec<_>>()
                )
            });
        (a.x, a.y, a.width, a.height)
    };
    for _ in 0..3 {
        tick(&mut ui);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    // Checks against the svg's box `(sx, sy)`, 2 px a unit at size 200.
    // The svg's box: at the padding's corner, or right to left at
    // the surface's far side.
    let check = |ui: &Ui, k: f32, x: f32, m: f32| {
        let sx = if ui.direction() == og_i18n::Direction::Rtl {
            400.0 - m - 100.0 * k
        } else {
            m
        };
        let sy = m;
        let (mx, my, mw, mh) = rect(ui, "Mid");
        // Centred on (50, 25): central baseline, middle anchor.
        assert!(
            (mx + mw / 2.0 - (sx + 50.0 * k)).abs() < 0.5,
            "Mid x {mx} {mw}"
        );
        assert!(
            (my + mh / 2.0 - (sy + 25.0 * k)).abs() < 0.5,
            "Mid y {my} {mh}"
        );
        // The font's extents at 10 units, scaled: taller than 10 px at k 2.
        assert!(mh > 10.0 * k * 0.9 && mh < 10.0 * k * 1.6, "Mid h {mh}");
        let (ex, _, ew, _) = rect(ui, "End");
        assert!(ew > 0.0);
        assert!(
            (ex + ew - (sx + x * k)).abs() < 0.5,
            "End right {ex}+{ew} k {k} sx {sx} rtl {:?}",
            ui.direction()
        );
    };
    check(&ui, 2.0, 90.0, 0.0);
    let idle = tick(&mut ui);
    assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{idle:?}");
    set(&mut ui, "x", json!(70));
    // That text alone: rebuilt with its own bindings, nothing else.
    let moved = tick(&mut ui);
    assert!(moved.bindings <= 2 && moved.reconciled == 1, "{moved:?}");
    tick(&mut ui);
    check(&ui, 2.0, 70.0, 0.0);
    set(&mut ui, "label", json!("Silver"));
    let named = tick(&mut ui);
    assert!(named.bindings == 1 && named.reconciled <= 2, "{named:?}");
    tick(&mut ui);
    check(&ui, 2.0, 70.0, 0.0);
    // The svg moves, then resizes: nothing is built, and the texts follow.
    for (n, v, k) in [("margin", json!(30), 2.0), ("size", json!(300), 3.0)] {
        set(&mut ui, n, v);
        for _ in 0..3 {
            tick(&mut ui);
        }
        let idle = tick(&mut ui);
        assert_eq!((idle.bindings, idle.reconciled), (0, 0), "{n}: {idle:?}");
        check(&ui, k, 70.0, 30.0);
    }
    // Right to left the svg moves across the surface, but its texts
    // stay where its view box shows them.
    ui.set_direction(og_i18n::Direction::Rtl);
    for _ in 0..3 {
        tick(&mut ui);
    }
    check(&ui, 3.0, 70.0, 30.0);
}

/// An `<image href>` of an atlas icon fills its rect through the view
/// box (contained), and resizing the `<svg>` refits it unbuilt.
#[test]
fn an_svg_image_fills_its_rect_from_the_atlas() {
    let mut ui = Ui::new(UiDesc::default());
    ui.set_icons(crate::IconAtlas {
        texture: og_draw::TextureHandle::from_slot(3),
        columns: 2,
        rows: 1,
        names: vec!["coin".into(), "gem".into()],
        ..Default::default()
    });
    let doc = r##"<meta name="component" content="pic">
<div id="Box"><svg id="S" viewBox="0 0 100 50"><image id="I" href="gem" x="50" y="0" width="50" height="50"/></svg></div>
<style>#S { width: 200px; height: 100px; } #Box { padding: 0; }</style>"##;
    ui.define_document(UiDocument::parse("pic.svelte", doc).unwrap());
    ui.mount_document(
        SourceId::new("test", "pic"),
        "pic",
        BTreeMap::new(),
        BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    for _ in 0..3 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
        ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
        ui.end();
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    let s = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.path == "test:pic/S")
        .unwrap();
    let img = ui.frame().items.iter().find_map(|it| match &it.cmd {
        DrawCmd::Shape { shape, .. } => match &shape.paint {
            og_draw::UiPaint::Image(im) => Some((shape.center, shape.half, im.uv)),
            _ => None,
        },
        _ => None,
    });
    let (c, h, uv) = img.expect("the image draws");
    // The right half of a 200×100 box: centre (150, 50), 50 px half.
    assert!(
        (c.x - (s.x + 150.0)).abs() < 1e-3 && (c.y - (s.y + 50.0)).abs() < 1e-3,
        "{c:?}"
    );
    assert!(
        (h.x - 50.0).abs() < 1e-3 && (h.y - 50.0).abs() < 1e-3,
        "{h:?}"
    );
    assert_eq!(uv, [0.5, 0.0, 1.0, 1.0]);
}
