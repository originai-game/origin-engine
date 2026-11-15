use crate::doc::UiDocument;
use crate::reactive::SourceId;
use crate::{DrawCmd, InputFrame, Ui, UiDesc, Viewport};
use og_core::glam::Vec2;
use std::collections::BTreeMap;

/// A dash list cut by hand: pattern 4 2 1 2 along a 20-unit line.
/// Ways it could fail: only the first dash and gap honoured; the
/// offset entering the pattern at the wrong element; a zero dash with
/// no dot for its caps; a dash cut short at a segment's end.
#[test]
fn dash_lists_cut_every_dash_and_gap() {
    let line = og_draw::kurbo::BezPath::from_svg("M0 0 L8 0 L20 0").unwrap();
    // Along the line (kurbo emits a subpath's first dash last, to join a
    // closed one's across its start).
    let spans = |pattern: &[f32], offset: f32| -> Vec<(f32, f32)> {
        let mut s: Vec<(f32, f32)> =
            og_draw::polylines(crate::svg::dashed(&line, pattern, offset), 0.01)
                .iter()
                .map(|(d, _)| (d[0].x, d.last().unwrap().x))
                .map(|(a, b)| ((a * 1000.0).round() / 1000.0, (b * 1000.0).round() / 1000.0))
                .collect();
        s.sort_by(|a, b| a.0.total_cmp(&b.0));
        s
    };
    let p = [4.0, 2.0, 1.0, 2.0];
    assert_eq!(
        spans(&p, 0.0),
        [
            (0.0, 4.0),
            (6.0, 7.0),
            (9.0, 13.0),
            (15.0, 16.0),
            (18.0, 20.0)
        ]
    );
    assert_eq!(
        spans(&p, 5.0),
        [
            (1.0, 2.0),
            (4.0, 8.0),
            (10.0, 11.0),
            (13.0, 17.0),
            (19.0, 20.0)
        ]
    );
    // Zero-length dashes are dots, each a tiny dash its caps round; one
    // due exactly at the open end is past it (kurbo's dashes are
    // half-open).
    assert_eq!(
        spans(&[0.0, 5.0], 0.0)
            .iter()
            .map(|s| s.0)
            .collect::<Vec<_>>(),
        [0.0, 5.0, 10.0, 15.0]
    );
}

const DOC: &str = r##"
<meta name="component" content="pic">
<div id="Box">
  <svg id="S" viewBox="0 0 100 100">
<defs>
  <marker id="Tip" viewBox="0 0 10 10" refX="0" refY="5" markerWidth="2" markerHeight="2" orient="auto-start-reverse">
    <path d="M0 0 L10 5 L0 10 z" fill="#0000ff"/>
  </marker>
  <path id="Arc" d="M10 80 L90 80"/>
</defs>
<polyline id="P" points="10,10 30,10 30,30" fill="none" stroke="#000" stroke-width="2"
  marker-start="url(#Tip)" marker-mid="url(#Tip)" marker-end="url(#Tip)"/>
<text id="T" x="50" y="40" text-anchor="middle" font-size="6"><tspan x="50">One</tspan><tspan x="50" dy="2em">Two</tspan></text>
<text id="W" x="80" y="20" font-size="6" white-space="pre-line">Up
  Down</text>
<text id="On" font-size="6" text-anchor="middle"><textPath href="#Arc" startOffset="50%">{$word}</textPath></text>
<foreignObject id="F" x="10" y="50" width="40" height="20"><div id="B"></div></foreignObject>
  </svg>
</div>
<style>#S { width: 200px; height: 200px; } #Box { padding: 0; } #B { width: 30px; height: 10px; }</style>
"##;

/// Markers, lines of a text, a text on a path and a foreignObject, in
/// a 2× box. Ways it could fail: markers not drawn at every vertex, or
/// not turned (auto-start-reverse not flipping the start); a tspan's
/// dy not moving its line, or lines not each on their anchor; a kept
/// line break dropped; text on a path drawn in a line; a foreignObject
/// laid out at its user size but not placed and scaled into the box.
#[test]
fn markers_lines_text_paths_and_foreign_objects_draw_in_the_view_box() {
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("pic.svelte", DOC).unwrap());
    let word = SourceId::new("test", "word");
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(crate::reactive::session::Command::Signal {
            source: word.clone(),
            initial: serde_json::json!("Along"),
            readable: false,
        })
        .unwrap();
    let mut inputs = BTreeMap::new();
    inputs.insert("word".to_string(), word);
    ui.mount_document(
        SourceId::new("test", "pic"),
        "pic",
        inputs,
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
    let o = Vec2::new(s.x, s.y);
    // The three blue triangles, 4 px long (2 user units, stroke 2, ×2
    // box... markerWidth 2 × stroke 2 = 4 units = 8 px), tail on the vertex.
    let mut tips: Vec<(Vec2, Vec2)> = ui
        .frame()
        .items
        .iter()
        .filter_map(|it| match &it.cmd {
            DrawCmd::Shape { shape, .. } => match (&shape.paint, &shape.geometry) {
                (og_draw::UiPaint::Solid(c), og_draw::UiGeometry::Path(p))
                    if c[2] > 0.9 && c[0] < 0.1 =>
                {
                    let (lo, hi) = p.bounds().unwrap();
                    let mid = shape.center + (lo + hi) / 2.0;
                    Some(((mid - o) / 2.0, (hi - lo) / 4.0))
                }
                _ => None,
            },
            _ => None,
        })
        .collect();
    tips.sort_by(|a, b| (a.0.x + a.0.y).total_cmp(&(b.0.x + b.0.y)));
    assert_eq!(tips.len(), 3, "{tips:?}");
    let near = |a: Vec2, b: Vec2| (a - b).length() < 0.2;
    // Start (10,10), reversed: pointing back along -x, 4 units long.
    assert!(near(tips[0].0, Vec2::new(8.0, 10.0)), "{tips:?}");
    // Mid (30,10): the bisector of +x and +y, a diagonal.
    let (c, _) = tips[1];
    assert!(
        c.x > 30.0 && c.y > 10.0 && near(c - Vec2::new(30.0, 10.0), Vec2::splat(c.x - 30.0)),
        "{tips:?}"
    );
    // End (30,30): pointing down +y.
    assert!(near(tips[2].0, Vec2::new(30.0, 32.0)), "{tips:?}");
    let rect = |id: &str| {
        ui.a11y()
            .nodes
            .iter()
            .filter(|a| a.path.starts_with(&format!("test:pic/S/{id}")) && !a.name.is_empty())
            .map(|a| (a.name.clone(), a.x, a.y, a.width, a.height))
            .collect::<Vec<_>>()
    };
    // Two lines centred on x 50, the second 2em (12 units, 24 px) down.
    let lines = rect("T");
    assert_eq!(
        lines.iter().map(|l| l.0.as_str()).collect::<Vec<_>>(),
        ["One", "Two"],
        "{lines:?}"
    );
    for l in &lines {
        assert!((l.1 + l.3 / 2.0 - (o.x + 100.0)).abs() < 0.6, "{l:?}");
    }
    assert!((lines[1].2 - lines[0].2 - 24.0).abs() < 0.6, "{lines:?}");
    // pre-line: its break kept, so two lines tall.
    let w = rect("W");
    assert_eq!(w[0].0, "Up\nDown");
    assert!(w[0].4 > 1.8 * lines[0].4, "{w:?} vs {lines:?}");
    // On the path: the text's box is the path's extent from the view
    // box's origin (glyphs are drawn along it; no font draws in tests).
    let on = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.path.ends_with("/On"))
        .unwrap();
    assert!(
        (on.x - o.x).abs() < 0.5
            && (on.y - o.y).abs() < 0.5
            && (on.width - 180.0).abs() < 0.5
            && (on.height - 160.0).abs() < 0.5,
        "{on:?}"
    );
    // The 30×10 box inside the foreignObject: at its corner (10,50),
    // laid out in user units and scaled 2× with the view box.
    let b = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.path.ends_with("/B"))
        .unwrap();
    let got = [b.x - o.x, b.y - o.y, b.width, b.height];
    assert!(
        got.iter()
            .zip([20.0, 100.0, 60.0, 20.0])
            .all(|(g, e)| (g - e).abs() < 0.5),
        "{b:?}"
    );
}

const SMIL: &str = r##"
<meta name="component" content="smil">
<div id="Box">
<svg id="S" viewBox="0 0 100 100">
  <g id="G">
<animateTransform attributeName="transform" type="translate" from="0 0" to="40 0" dur="1s" fill="freeze"/>
<path d="M0 0 H10 V10 Z" fill="#00ff00"/>
  </g>
  <polygon id="P" points="0,50 10,50 10,60" fill="#ff0000">
<animate id="Grow" attributeName="points" values="0,50 10,50 10,60;0,50 30,50 30,60" begin="0.5s" dur="1s" fill="freeze"/>
<set attributeName="fill" to="#0000ff" begin="Grow.end"/>
  </polygon>
</svg>
</div>
<style>#S { width: 100px; height: 100px; } #Box { padding: 0; }</style>
"##;

/// SMIL on the UI clock (10 ticks a second): a group's
/// animateTransform moving the path it holds, a polygon's points
/// growing from a delayed begin, and a set chained to its end.
/// Ways it could fail: a group's animated transform not reaching its
/// children; the element not resolving again each frame (frozen at its
/// first value); begin read from the wrong clock; fill="freeze" not
/// holding the end; a syncbase begin never firing; reduced motion
/// still playing.
#[test]
fn smil_animations_play_on_the_ui_clock() {
    let run = |reduced: bool, frames: u32| -> Vec<Vec<([f32; 3], f32, f32)>> {
        let mut ui = Ui::new(UiDesc {
            ticks_per_second: 10,
            ..UiDesc::default()
        });
        if reduced {
            let mut theme = *ui.theme();
            theme.reduced_motion = true;
            ui.set_theme(theme);
        }
        ui.define_document(UiDocument::parse("smil.svelte", SMIL).unwrap());
        ui.mount_document(
            SourceId::new("test", "smil"),
            "smil",
            BTreeMap::new(),
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let mut out = Vec::new();
        for _ in 0..frames {
            ui.begin(InputFrame::default(), Viewport::new(200.0, 200.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
            let s = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.path == "test:smil/S")
                .unwrap();
            // Each solid path's color and its left and right edges.
            let mut shapes: Vec<_> = ui
                .frame()
                .items
                .iter()
                .filter_map(|it| match &it.cmd {
                    DrawCmd::Shape { shape, .. } => match (&shape.paint, &shape.geometry) {
                        (og_draw::UiPaint::Solid(c), og_draw::UiGeometry::Path(p)) => {
                            let (lo, hi) = p.bounds().map(|(a, b)| (a.x, b.x)).unwrap();
                            let x = shape.center.x - s.x;
                            Some(([c[0], c[1], c[2]], x + lo, x + hi))
                        }
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            shapes.sort_by(|a, b| a.0[1].total_cmp(&b.0[1]).reverse());
            out.push(shapes);
        }
        out
    };
    let frames = run(false, 25);
    let at = |tick: usize| {
        assert_eq!(frames[tick].len(), 2, "{tick}: {:?}", frames[tick]);
        (frames[tick][0], frames[tick][1])
    };
    let near = |a: f32, b: f32| (a - b).abs() < 0.05;
    // Tick 0: the clock starts as it shows; nothing has moved.
    let (g, p) = at(0);
    assert!(
        near(g.1, 0.0) && near(p.2, 10.0) && p.0[0] > 0.9,
        "{g:?} {p:?}"
    );
    // 0.5 s: the group halfway (20 units), the polygon about to grow.
    let (g, p) = at(5);
    assert!(near(g.1, 20.0) && near(p.2, 10.0), "{g:?} {p:?}");
    // 1 s: the group frozen at 40; the polygon halfway (20).
    let (g, p) = at(10);
    assert!(near(g.1, 40.0) && near(p.2, 20.0), "{g:?} {p:?}");
    // 1.5 s: Grow ends at 30 and freezes; its end turns the fill blue.
    let (g, p) = at(15);
    assert!(
        near(g.1, 40.0) && near(p.2, 30.0) && p.0[2] > 0.9,
        "{g:?} {p:?}"
    );
    let (g, p) = at(24);
    assert!(
        near(g.1, 40.0) && near(p.2, 30.0) && p.0[2] > 0.9,
        "{g:?} {p:?}"
    );
    // Reduced motion: each shows its frozen end from the first frame
    // (the set has no end, so no effect).
    let still = run(true, 2);
    let (g, p) = (still[1][0], still[1][1]);
    assert!(
        near(g.1, 40.0) && near(p.2, 30.0) && p.0[0] > 0.9,
        "{still:?}"
    );
}

const STAMP: &str = r##"
<meta name="component" content="stamp">
<div id="Root">
  <div id="Box" class:plain={$plain}></div>
  <svg id="Defs" width="1" height="1">
    <defs>
      <g id="Star">
        <rect id="Body" width="10" height="10" fill={$color}/>
        {#if $on}<circle id="Dot" cx="5" cy="5" r="2" fill="#0000ff"/>{/if}
      </g>
      {#each $rows as row (row.id)}<rect id="Cell" width="4" height="4" fill={row.fill}/>{/each}
      <filter id="Stamp" x="0" y="0" width="1" height="1">
        <feImage href="#Star" x="4" y="6" result="star"/>
        <feComposite in="star" in2="SourceGraphic" operator="over"/>
      </filter>
      <filter id="Row" x="0" y="0" width="1" height="1"><feImage href="#Cell"/></filter>
    </defs>
  </svg>
  <div id="Lost" class:on={$lost}></div>
</div>
<style>
#Box { width: 40px; height: 40px; background: #202020; filter: url(#Stamp); }
#Box.plain { filter: none; }
#Box:empty { width: 60px; }
#Lost { width: 8px; height: 8px; }
#Lost.on { filter: url(#Row); }
</style>"##;

/// [`STAMP`] mounted over its stores, and what its mount holds: its
/// elements, its bindings and its element images.
fn stamp(plain: bool, lost: bool) -> Ui {
    use crate::reactive::session::Command;
    use serde_json::json;
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(UiDocument::parse("stamp.svelte", STAMP).unwrap());
    let mut inputs = BTreeMap::new();
    for (name, initial) in [
        ("color", json!("#ff0000")),
        ("on", json!(false)),
        ("plain", json!(plain)),
        ("lost", json!(lost)),
    ] {
        let source = SourceId::new("test", name);
        ui.docs
            .state_mut()
            .memory
            .reactive
            .execute(Command::Signal {
                source: source.clone(),
                initial,
                readable: false,
            })
            .unwrap();
        inputs.insert(name.to_string(), source);
    }
    let rows = SourceId::new("test", "rows");
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Collection {
            source: rows.clone(),
            key: "id".into(),
            items: vec![
                json!({ "id": "a", "fill": "#00ff00" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ],
        })
        .unwrap();
    inputs.insert("rows".to_string(), rows);
    ui.mount_document(
        SourceId::new("test", "stamp"),
        "stamp",
        inputs,
        BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    ui
}

fn stamp_set(ui: &mut Ui, name: &str, value: serde_json::Value) {
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(crate::reactive::session::Command::Set {
            source: SourceId::new("test", name),
            value,
            force: false,
        })
        .unwrap();
}

fn stamp_tick(ui: &mut Ui) -> crate::element::LayoutStats {
    ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
    ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
    ui.end().stats
}

fn stamp_held(ui: &Ui) -> (usize, usize, usize) {
    let host = &ui.docs.state().memory.retained;
    (host.elements.len(), host.inputs.len(), host.images.len())
}

/// An `<feImage href="#id">` draws the element mounted: its bound fill
/// follows its store, its `{#if}` block comes and goes, and the image is
/// unmounted with the filter.
///
/// Ways it could fail: the image built from what is written (a bound fill
/// reads nothing, a block is never built); the image counted among the
/// filtered element's children (`:empty` stops matching); the image's
/// bindings and elements left behind when the filter goes; the image built
/// again every frame.
#[test]
fn an_fe_image_of_an_element_draws_it_mounted() {
    use serde_json::json;
    let mut ui = stamp(false, false);
    let (set, tick, held) = (stamp_set, stamp_tick, stamp_held);
    // What the mount holds before any image is mounted.
    let bare = held(&ui);
    let mut stats = Default::default();
    for _ in 0..4 {
        stats = tick(&mut ui);
    }
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    // Solid shapes by colour: their paths' centres from the box's top left.
    let solid = |ui: &Ui, rgb: [f32; 3]| -> Vec<Vec2> {
        let b = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.path == "test:stamp/Box")
            .unwrap();
        ui.frame()
            .items
            .iter()
            .filter_map(|it| match &it.cmd {
                DrawCmd::Shape { shape, .. } => match &shape.paint {
                    og_draw::UiPaint::Solid(c) if (0..3).all(|k| (c[k] - rgb[k]).abs() < 0.05) => {
                        let og_draw::UiGeometry::Path(p) = &shape.geometry else {
                            panic!("a path: {:?}", shape.geometry);
                        };
                        let (lo, hi) = p.bounds().unwrap();
                        Some(shape.center + (lo + hi) / 2.0 - Vec2::new(b.x, b.y))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect()
    };
    let paths = |ui: &Ui| -> Vec<String> {
        ui.probe_elements()
            .into_iter()
            .map(|e| e.path)
            .filter(|p| p.contains("~image"))
            .collect()
    };
    // The body, red, 10 px square at (4, 6) of the box: centre (9, 11).
    let red = solid(&ui, [1.0, 0.0, 0.0]);
    assert_eq!(red.len(), 1, "{red:?} {:?}", paths(&ui));
    assert!((red[0] - Vec2::new(9.0, 11.0)).length() < 0.6, "{red:?}");
    assert!(solid(&ui, [0.0, 0.0, 1.0]).is_empty(), "the dot is off");
    // The image is no child of the box: `:empty` still sizes it.
    let width = |ui: &Ui| {
        ui.probe_elements()
            .into_iter()
            .find(|e| e.path == "test:stamp/Box")
            .unwrap()
            .width
    };
    assert_eq!(width(&ui), 60.0);
    assert_eq!(held(&ui).2, 1);
    // Its fill follows the store, its block is built.
    set(&mut ui, "color", json!("#00ff00"));
    set(&mut ui, "on", json!(true));
    tick(&mut ui);
    assert!(solid(&ui, [1.0, 0.0, 0.0]).is_empty());
    assert_eq!(solid(&ui, [0.0, 1.0, 0.0]).len(), 1);
    let blue = solid(&ui, [0.0, 0.0, 1.0]);
    assert_eq!(blue.len(), 1, "{:?}", paths(&ui));
    assert!((blue[0] - Vec2::new(9.0, 11.0)).length() < 0.6, "{blue:?}");
    assert_eq!(width(&ui), 60.0);
    set(&mut ui, "on", json!(false));
    tick(&mut ui);
    assert!(solid(&ui, [0.0, 0.0, 1.0]).is_empty());
    for _ in 0..3 {
        stats = tick(&mut ui);
    }
    assert_eq!(
        (stats.bindings, stats.reconciled),
        (0, 0),
        "idle: {stats:?}"
    );
    // Without the filter the image is unmounted, with all it held.
    set(&mut ui, "plain", json!(true));
    tick(&mut ui);
    assert!(solid(&ui, [0.0, 1.0, 0.0]).is_empty());
    assert!(paths(&ui).is_empty(), "{:?}", paths(&ui));
    assert_eq!(held(&ui), bare);
    // And mounted again with it.
    set(&mut ui, "plain", json!(false));
    tick(&mut ui);
    assert_eq!(solid(&ui, [0.0, 1.0, 0.0]).len(), 1);
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
}

/// An element that cannot be mounted where the filter is (it reads the row
/// of an `{#each}` the filtered element is not in) draws nothing, says so,
/// and leaves nothing behind: its half-made bindings would otherwise be
/// refreshed with no element to take them.
#[test]
fn an_fe_image_of_an_element_out_of_its_row_is_refused_whole() {
    use crate::reactive::session::Command;
    use serde_json::json;
    let mut ui = stamp(true, true);
    let bare = stamp_held(&ui);
    stamp_tick(&mut ui);
    let warnings = ui.frame().warnings.clone();
    assert!(
        warnings.len() == 1
            && warnings[0].contains("<feImage href=\"#Cell\"> cannot draw the element here"),
        "{warnings:?}"
    );
    assert_eq!(stamp_held(&ui), bare);
    // The stores it would have read change: nothing is left to refresh.
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Insert {
            source: SourceId::new("test", "rows"),
            row: json!({ "id": "b", "fill": "#0000ff" })
                .as_object()
                .unwrap()
                .clone(),
            before: None,
        })
        .unwrap();
    stamp_set(&mut ui, "color", json!("#00ff00"));
    for _ in 0..2 {
        stamp_tick(&mut ui);
    }
    assert_eq!(stamp_held(&ui).2, 0);
}
