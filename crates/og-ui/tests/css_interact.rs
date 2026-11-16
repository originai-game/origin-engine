//! CSS interaction properties (docs/ui.md §2): `pointer-events`,
//! `visibility` and the cursor keywords.
//!
//! Ways these could fail, each covered below:
//! - `pointer-events: none` taking the pointer anyway, or not handing it
//!   to what is under it, or a child saying `auto` inside it not taking
//!   it → `the_pointer_passes_through_none_and_hidden`
//! - `visibility: hidden` drawing or taking the pointer, or not letting a
//!   child that says `visible` take it → the same test
//! - `cursor: url()` images lost before the host, or images over `auto`
//!   taking the parent's keyword → the same test
//! - a cursor applying outside an element's `clip-path` (its own, or an
//!   ancestor's cutting a child), or a fill rule or `url(#id)` clip
//!   ignored → `cursors_apply_only_inside_clip_paths`
//! - `object-fit` or `object-position` not reaching an `<image>`, or
//!   `image-rendering` not inherited (or `auto` not resetting it) →
//!   `images_take_object_fit_position_and_inherited_image_rendering`

use crate::support;

use og_core::glam::Vec2;
use og_ui::reactive::SourceId;
use og_ui::*;
use support::*;

const DOC: &str = r#"<meta name="component" content="ix">
<div id="Root">
  <button id="Under">Under</button>
  <div id="Over" onpointerdown={over}><button id="Again">Again</button></div>
  <div id="Ghost"><button id="Lost">Lost</button><button id="Shown">Shown</button></div>
</div>
<style>
#Root { width: 600px; height: 400px; cursor: help; }
#Under { width: 200px; height: 60px; cursor: url(arrow.svg) 3 4, url(fallback.svg), crosshair; }
#Over { position: absolute; top: 0; inset-inline-start: 0; width: 300px; height: 60px; pointer-events: none; cursor: wait; }
#Again { position: absolute; top: 0; inset-inline-start: 220px; width: 60px; height: 60px; pointer-events: auto; cursor: copy; }
#Ghost { visibility: hidden; cursor: no-drop; }
#Lost { width: 100px; height: 40px; }
#Shown { width: 100px; height: 40px; visibility: visible; cursor: url(grip.svg), auto; }
</style>"#;

/// A UI with `text` (the component `ix`) mounted.
fn mounted(source: &str, text: &str) -> Ui {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse(source, text).expect("parses"));
    mount_stores(&mut ui, SourceId::new("test", "t"), "ix", &[]);
    ui
}

/// One frame of the mounted document with the pointer at `p`.
fn frame(ui: &mut Ui, p: Vec2) -> Cursor {
    ui.begin(
        InputFrame {
            pointer: p,
            pointer_inside: true,
            ..InputFrame::default()
        },
        Viewport::new(800.0, 600.0),
    );
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    ui.frame().cursor
}

#[test]
fn the_pointer_passes_through_none_and_hidden() {
    let mut ui = mounted("ui/ix.svelte", DOC);
    frame(&mut ui, Vec2::ZERO);
    let el = |ui: &Ui, id: &str| {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap();
        Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
    };
    let (under, again, lost, shown) = (
        el(&ui, "Under"),
        el(&ui, "Again"),
        el(&ui, "Lost"),
        el(&ui, "Shown"),
    );
    // Over the button the overlay covers: the overlay passes it on. Its
    // images come with its keyword, in order, for the host to try.
    frame(&mut ui, under);
    assert_eq!(frame(&mut ui, under), Cursor::Crosshair);
    let urls = |ui: &Ui| {
        let f = ui.frame();
        f.cursor_images
            .iter()
            .map(|c| (c.url.clone(), c.hotspot))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        urls(&ui),
        [
            ("/ui/arrow.svg".into(), Some([3.0, 4.0])),
            ("/ui/fallback.svg".into(), None)
        ]
    );
    // Its child that says auto takes the pointer again.
    frame(&mut ui, again);
    assert_eq!(frame(&mut ui, again), Cursor::Copy);
    // A hidden button takes nothing: the pointer is on the root.
    frame(&mut ui, lost);
    assert_eq!(frame(&mut ui, lost), Cursor::Help);
    // A visible child of a hidden parent takes it. Images over `auto` are
    // its own value: the keyword is its kind's, not its parent's no-drop.
    frame(&mut ui, shown);
    assert_eq!(frame(&mut ui, shown), Cursor::Default);
    assert_eq!(urls(&ui), [("/ui/grip.svg".into(), None)]);
    let a11y = format!("{:?}", ui.a11y());
    assert!(a11y.contains("Shown") && !a11y.contains("Lost"), "{a11y}");
}

#[test]
fn plain_boxes_choose_cursors_in_clipped_transformed_paint_order() {
    let doc = r#"<meta name="component" content="cursors" />
<section id="Root">
  <button id="Under">Under</button>
  <div id="Cover"></div><div id="Pass"></div>
  <div id="Clip"><div id="Crop"></div></div>
  <div id="Turn"></div>
  <div id="Ghost"><div id="Shown"></div><div id="Hidden"></div></div>
  <div id="Inherit"></div>
</section>
<style>
#Root { width: 240px; height: 200px; cursor: help; }
#Under, #Cover, #Pass { position: absolute; left: 10px; top: 10px; width: 80px; height: 40px; }
#Under { cursor: crosshair; }
#Cover { z-index: 1; cursor: url(hand.svg) 2 3, wait; }
#Pass { z-index: 2; pointer-events: none; cursor: copy; }
#Clip { position: absolute; left: 120px; top: 10px; width: 40px; height: 40px; overflow: hidden; }
#Crop { width: 80px; height: 40px; cursor: grab; }
#Turn { position: absolute; left: 100px; top: 100px; width: 80px; height: 20px;
  transform-origin: top left; transform: rotate(90deg); cursor: copy; }
#Ghost { position: absolute; left: 190px; top: 50px; width: 40px; height: 40px;
  visibility: hidden; cursor: progress; }
#Shown, #Hidden { position: absolute; top: 0; width: 20px; height: 20px; }
#Shown { left: 0; visibility: visible; cursor: zoom-in; }
#Hidden { left: 20px; }
#Inherit { position: absolute; left: 200px; top: 150px; width: 20px; height: 20px; }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/cursors.svelte", doc).unwrap());
    mount_stores(&mut ui, SourceId::new("test", "cursor"), "cursors", &[]);
    for (x, y, expected) in [
        (20.0, 20.0, Cursor::Wait),
        (130.0, 20.0, Cursor::Grab),
        (175.0, 20.0, Cursor::Help),
        (90.0, 130.0, Cursor::Copy),
        (125.0, 105.0, Cursor::Help),
        (195.0, 55.0, Cursor::ZoomIn),
        (225.0, 55.0, Cursor::Help),
        (205.0, 155.0, Cursor::Help),
        (230.0, 190.0, Cursor::Help),
        (260.0, 190.0, Cursor::Default),
    ] {
        for _ in 0..2 {
            ui.begin(
                InputFrame {
                    pointer: Vec2::new(x, y),
                    pointer_inside: true,
                    ..Default::default()
                },
                Viewport::new(300.0, 240.0),
            );
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        assert_eq!(ui.frame().cursor, expected, "({x}, {y})");
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        if expected == Cursor::Wait {
            assert_eq!(
                ui.frame().cursor_images,
                [CursorImage {
                    url: "/ui/hand.svg".into(),
                    hotspot: Some([2.0, 3.0])
                }]
            );
        }
    }
}

#[test]
fn cursors_apply_only_inside_clip_paths() {
    let doc = r##"<meta name="component" content="cuts" />
<section id="Root">
  <svg id="Defs" viewBox="0 0 1 1" width="0" height="0">
    <clipPath id="Half" clipPathUnits="objectBoundingBox"><path d="M0 0 L1 0 L0 1 Z"/></clipPath>
  </svg>
  <button id="Under">Under</button>
  <div id="Round"></div>
  <div id="Wedge"></div>
  <div id="Ring"></div>
  <div id="Drawn"></div>
  <div id="Url"></div>
  <div id="Parent"><div id="Child"></div></div>
</section>
<style>
#Root { position: relative; width: 300px; height: 200px; cursor: help; }
#Under { position: absolute; left: 0; top: 0; width: 80px; height: 80px; cursor: crosshair; }
#Round { position: absolute; left: 0; top: 0; width: 80px; height: 80px;
  clip-path: circle(30px at 40px 40px); cursor: wait; }
#Wedge { position: absolute; left: 100px; top: 0; width: 80px; height: 80px;
  clip-path: polygon(0 0, 100% 0, 0 100%) content-box; padding: 0; cursor: grab; }
#Ring { position: absolute; left: 200px; top: 0; width: 80px; height: 80px; cursor: copy;
  clip-path: path(evenodd, "M0 0 H80 V80 H0 Z M20 20 H60 V60 H20 Z"); }
#Drawn { position: absolute; left: 0; top: 100px; width: 80px; height: 80px; cursor: move;
  clip-path: shape(from 0 0, line to 50% 0, line to 50% 100%, line to 0 100%, close); }
#Url { position: absolute; left: 100px; top: 100px; width: 80px; height: 80px;
  clip-path: url(#Half); cursor: zoom-in; }
#Parent { position: absolute; left: 200px; top: 100px; width: 80px; height: 80px;
  clip-path: inset(0 40px 0 0); }
#Child { width: 80px; height: 80px; cursor: progress; }
</style>"##;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/cuts.svelte", doc).unwrap());
    mount_stores(&mut ui, SourceId::new("test", "cuts"), "cuts", &[]);
    for (x, y, expected) in [
        // circle(): inside it the overlay's, in the box's corner the
        // button under it.
        (40.0, 40.0, Cursor::Wait),
        (4.0, 4.0, Cursor::Crosshair),
        // polygon(): the top-left triangle only.
        (110.0, 10.0, Cursor::Grab),
        (170.0, 70.0, Cursor::Help),
        // path() with evenodd: the ring, not its hole.
        (205.0, 5.0, Cursor::Copy),
        (240.0, 40.0, Cursor::Help),
        // shape(): the left half.
        (20.0, 140.0, Cursor::Move),
        (60.0, 140.0, Cursor::Help),
        // url(#id) in objectBoundingBox units: the top-left triangle.
        (110.0, 110.0, Cursor::ZoomIn),
        (170.0, 170.0, Cursor::Help),
        // An ancestor's clip cuts its child's cursor too.
        (220.0, 140.0, Cursor::Progress),
        (260.0, 140.0, Cursor::Help),
    ] {
        for _ in 0..2 {
            ui.begin(
                InputFrame {
                    pointer: Vec2::new(x, y),
                    pointer_inside: true,
                    ..Default::default()
                },
                Viewport::new(320.0, 220.0),
            );
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        assert_eq!(ui.frame().cursor, expected, "({x}, {y})");
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }
}

#[test]
fn images_take_object_fit_position_and_inherited_image_rendering() {
    struct Env(og_draw::TextureHandle);
    impl bind::Env for Env {
        fn texture(&self, _: f64) -> Option<og_draw::TextureHandle> {
            Some(self.0)
        }
    }
    let mut handles = Handles::default();
    let t = handles.texture();
    let doc = r#"<meta name="component" content="im">
<div id="Root"><img id="Pic" src={1} alt="" /><img id="Plain" src={1} alt="" /></div>
<style>
#Root { image-rendering: pixelated; }
#Pic { width: 100px; height: 50px; object-fit: contain; object-position: right 10px top; }
#Plain { width: 10px; height: 10px; image-rendering: auto; }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/im.svelte", doc).expect("parses"));
    ui.mount_document(
        SourceId::new("test", "t"),
        "im",
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
    let images: Vec<_> = ui
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
    // The fitted one, sampled nearest by its parent's image-rendering and
    // placed 10 px in from the right edge; the one saying auto is a plain
    // stretched image (not a shape).
    assert_eq!(images.len(), 1, "{images:?}");
    let im = images[0];
    assert_eq!(im.fit, og_draw::UiFit::Contain);
    assert_eq!(
        (im.focal, im.offset),
        (Vec2::new(1.0, 0.0), Vec2::new(-10.0, 0.0))
    );
    assert!(im.nearest);
}

/// `translate`'s percentages are of the element's own box, as CSS's: a
/// pin at `left: 300px; top: 200px` with `translate: -50% -50%` is centred
/// on that point for the probe and for the pointer, and the place its box
/// would have without the shift takes nothing. A px translate that wins
/// the cascade replaces an earlier percentage.
#[test]
fn translate_percentages_move_by_the_elements_own_box() {
    const PIN: &str = r#"<meta name="component" content="ix">
<div id="Root">
  <button id="Pin">Pin</button>
  <button id="Back" class="centred">Back</button>
</div>
<style>
#Root { width: 600px; height: 400px; }
#Pin { position: absolute; left: 300px; top: 200px; width: 80px; height: 40px; translate: -50% -50%; cursor: copy; }
.centred { translate: -50% -50%; }
#Back { position: absolute; left: 100px; top: 300px; width: 80px; height: 40px; translate: 10px 0; }
</style>"#;
    let mut ui = mounted("ui/ix.svelte", PIN);
    frame(&mut ui, Vec2::ZERO);
    let pin = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.id == "Pin")
        .unwrap();
    assert_eq!(
        (pin.x + pin.width / 2.0, pin.y + pin.height / 2.0),
        (300.0, 200.0)
    );
    // A winning px translate replaces a percentage, as CSS's does.
    let back = ui
        .probe_elements()
        .into_iter()
        .find(|e| e.id == "Back")
        .unwrap();
    assert_eq!((back.x, back.y), (110.0, 300.0));
    let centre = Vec2::new(300.0, 200.0);
    frame(&mut ui, centre);
    assert_eq!(frame(&mut ui, centre), Cursor::Copy);
    // Inside the unshifted box (300..380, 200..240), outside the shifted
    // one: the pointer is on the root, as over a hidden button.
    let unshifted = Vec2::new(370.0, 232.0);
    frame(&mut ui, unshifted);
    assert_eq!(frame(&mut ui, unshifted), Cursor::Default);
}

#[test]
fn translate_none_clears_percentage_and_length_offsets_in_kits() {
    let text = r#"<meta name="component" content="ix">
<div><div id="Percent"></div><div id="Length"></div></div>
<style>
#Percent, #Length { position: absolute; left: 100px; top: 150px; width: 80px; height: 40px; }
#Percent { translate: -50% -50%; }
#Length { translate: 12px -24px; }
#Percent, #Length { translate: none; }
</style>"#;
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("@ui/test/ix.svelte", text).unwrap());
    mount_stores(&mut ui, SourceId::new("test", "none"), "ix", &[]);
    ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
    ui.build_retained_documents(&bind::NoEnv).unwrap();
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    for id in ["Percent", "Length"] {
        let node = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap();
        assert_eq!((node.x, node.y), (100.0, 150.0), "{id}");
    }
}

#[test]
fn focus_within_follows_real_focus_between_retained_ancestors() {
    let text = r#"<meta name="component" content="t">
<div id="Root">
  <div id="Left"><div id="Nested"><button id="First">First</button><button id="Second">Second</button></div></div>
  <div id="Right"><button id="Third">Third</button></div>
</div>
<style>
#Root { width: 400px; }
#Left, #Right { width: 120px; }
#Nested { width: 100px; }
#Left:focus-within { width: 180px; }
#Right:focus-within { width: 220px; }
#Nested:focus-within { width: 160px; }
button { width: 80px; height: 40px; }
</style>"#;
    let mut d = crate::support::Doc::new(text, &[]);
    let widths = |d: &crate::support::Doc| {
        [
            d.get("Left").width,
            d.get("Nested").width,
            d.get("Right").width,
        ]
    };
    d.click("First");
    assert_eq!(widths(&d), [180.0, 160.0, 120.0]);
    d.click("Second");
    assert_eq!(
        widths(&d),
        [180.0, 160.0, 120.0],
        "shared ancestors remain focused"
    );
    d.click("Third");
    assert_eq!(
        widths(&d),
        [120.0, 100.0, 220.0],
        "old ancestors lose focus-within"
    );
    d.click("First");
    assert_eq!(widths(&d), [180.0, 160.0, 120.0]);
}

/// CSS Transforms 1 and 2 place the box and its hit test by the
/// matrix the spec composes: `transform-origin` (default the centre, here a
/// corner), `matrix()`, `skewX()`, `transform-box: content-box` moving the
/// reference box, and in 3D `perspective` with `perspective-origin`,
/// `translateZ` and `rotateY` projected through the viewer's distance. Expected
/// bounds are worked from the spec's matrices (comments), not read back.
/// Ways it could fail: the origin taken as the top-left by default (or the
/// centre when a corner is written), matrix() arguments read in another
/// order, skew applied to y, the content box ignored, perspective scaling
/// about the box's own centre instead of the perspective origin, a sign
/// flipped in rotateY's depth, the probe or the pointer keeping the
/// untransformed box.
#[test]
fn transforms_place_and_hit_by_the_specs_matrices() {
    const BOXES: &str = r#"<meta name="component" content="ix">
<div id="Root">
  <div id="Rot" class="b"></div>
  <div id="Corner" class="b"></div>
  <div id="Mat" class="b"></div>
  <div id="Skew" class="b"></div>
  <div id="Persp"><div id="Near" class="b"></div><div id="Turn" class="b"></div></div>
  <div id="Content" class="b pad"></div>
  <div id="Border" class="b pad"></div>
</div>
<style>
#Root { width: 800px; height: 900px; cursor: help; }
.b { position: absolute; width: 100px; height: 40px; background: #808080; cursor: copy; }
#Rot { left: 100px; top: 100px; transform: rotate(90deg); }
#Corner { left: 300px; top: 100px; transform-origin: 0 0; transform: rotate(90deg); }
#Mat { left: 500px; top: 100px; transform: matrix(2, 0, 0, 0.5, 10, 20); }
#Skew { left: 100px; top: 250px; transform: skewX(45deg); }
#Persp { position: absolute; left: 0; top: 400px; width: 800px; height: 200px; perspective: 200px; perspective-origin: 150px 20px; }
#Near { left: 100px; top: 0; transform: translateZ(100px); }
#Turn { left: 300px; top: 0; transform: rotateY(60deg); }
.pad { top: 300px; padding: 20px; box-sizing: border-box; transform-origin: 0 0; transform: scale(2); }
#Content { left: 300px; transform-box: content-box; }
#Border { left: 550px; transform-box: border-box; }
</style>"#;
    let mut ui = mounted("ui/ix.svelte", BOXES);
    frame(&mut ui, Vec2::ZERO);
    let bounds = |ui: &Ui, id: &str| {
        let e = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.id == id)
            .unwrap_or_else(|| {
                panic!(
                    "no {id}: {:?}",
                    ui.probe_elements()
                        .iter()
                        .map(|e| e.id.clone())
                        .collect::<Vec<_>>()
                )
            });
        [e.x, e.y, e.width, e.height]
    };
    let near = |a: [f32; 4], b: [f32; 4]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3);
    let cases: [(&str, [f32; 4]); 7] = [
        // Centre (150, 120); (x, y) → (−y, x) about it.
        ("Rot", [130.0, 70.0, 40.0, 100.0]),
        // About (300, 100): x 300−40..300, y 100..200.
        ("Corner", [260.0, 100.0, 40.0, 100.0]),
        // (±50, ±20) about (550, 120) → (±100, ±10) + (10, 20).
        ("Mat", [460.0, 130.0, 200.0, 20.0]),
        // x' = x + y·tan 45° over y ∈ ±20 about the centre.
        ("Skew", [80.0, 250.0, 140.0, 40.0]),
        // d / (d − z) = 200 / 100 = 2 about the perspective origin
        // (150, 420): x 150 ± 2·50 → 50..250, y 420 + 2·(−20..20).
        ("Near", [50.0, 380.0, 200.0, 80.0]),
        // Content box origin (320, 320): x 320 + 2·(300 − 320) = 280.
        ("Content", [280.0, 280.0, 200.0, 80.0]),
        // Border box origin (550, 300).
        ("Border", [550.0, 300.0, 200.0, 80.0]),
    ];
    for (id, expected) in cases {
        assert!(
            near(bounds(&ui, id), expected),
            "{id}: {:?}",
            bounds(&ui, id)
        );
    }
    // rotateY(60°) about its centre (350, 420): x' = x·cos θ, z = −x·sin θ,
    // then projected about the origin (150, 420) by d / (d − z).
    let project = |x: f32, y: f32| {
        let (s, c) = 60f32.to_radians().sin_cos();
        let (px, z) = (350.0 + x * c, -x * s);
        let k = 200.0 / (200.0 - z);
        (150.0 + (px - 150.0) * k, 420.0 + y * k)
    };
    let corners =
        [(-50.0, -20.0), (50.0, -20.0), (-50.0, 20.0), (50.0, 20.0)].map(|(x, y)| project(x, y));
    let (x0, x1) = corners
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), c| (a.min(c.0), b.max(c.0)));
    let (y0, y1) = corners
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), c| (a.min(c.1), b.max(c.1)));
    assert!(
        near(bounds(&ui, "Turn"), [x0, y0, x1 - x0, y1 - y0]),
        "Turn: {:?} vs {:?}",
        bounds(&ui, "Turn"),
        [x0, y0, x1 - x0, y1 - y0]
    );
    // The pointer follows the transformed box, not the laid-out one.
    for (at, cursor) in [
        // Inside the rotated box, above the laid-out one.
        (Vec2::new(150.0, 80.0), Cursor::Copy),
        // Inside the laid-out box, left of the rotated one.
        (Vec2::new(110.0, 120.0), Cursor::Help),
        // Laid out at 300..400, 400..440 but projected onto 335..373; the
        // near edge is taller than the laid-out box.
        (Vec2::new(330.0, 420.0), Cursor::Help),
        (Vec2::new(380.0, 420.0), Cursor::Help),
        (Vec2::new(345.0, 420.0), Cursor::Copy),
        (Vec2::new(368.0, 397.0), Cursor::Copy),
    ] {
        frame(&mut ui, at);
        assert_eq!(frame(&mut ui, at), cursor, "at {at}");
    }
}

/// Pointer Events 3 §9 `touch-action` and CSSOM View's `scroll-behavior`.
/// A diagonal touch drag over a two-axis scroller pans only what the
/// element it started on allows: `pan-y` drops the x move, `pan-left`
/// keeps only a finger moving left (content offset growing), and a child's
/// `pan-y` inside a `pan-x` parent intersects to none; a fine pointer pans
/// freely. A scroll into view glides under `smooth` (more than one tick,
/// monotonic, landing on the target and staying) and jumps under `auto`.
/// Ways it could fail: an axis kept that the value drops, the direction
/// of pan-left read as the content's, an ancestor's value ignored (a
/// union instead of an intersection), touch-action applied to a mouse,
/// a touch-action read off the scroll container instead of the element
/// the touch landed on, a smooth scroll that jumps or stops short, or
/// `auto` gliding.
#[test]
fn touch_action_filters_pans_and_scroll_behavior_glides() {
    let text = r#"<meta name="component" content="t">
<div id="Root">
  <div id="Pan" class="s"><div class="big"><div id="Y" class="hit"></div><div id="L" class="hit"></div><div id="X" class="hit"><div id="XY" class="hit"></div></div><div id="Free" class="hit"></div></div></div>
  <div id="Smooth" class="s v"><div id="SmoothTall" class="tall"><button id="SmoothEnd" class="end">S</button></div></div>
  <div id="Jump" class="s v"><div id="JumpTall" class="tall"><button id="JumpEnd" class="end">J</button></div></div>
</div>
<style>
#Root { width: 800px; display: flex; flex-direction: row; align-items: start; gap: 10px; }
.s { width: 200px; height: 200px; overflow: auto; scrollbar-width: none; }
.big { position: relative; width: 1000px; height: 1000px; }
.hit { position: absolute; width: 60px; height: 60px; }
#Y { left: 10px; top: 10px; touch-action: pan-y; }
#L { left: 100px; top: 10px; touch-action: pan-left; }
#X { left: 10px; top: 100px; width: 80px; height: 80px; touch-action: pan-x; }
#XY { left: 10px; top: 10px; touch-action: pan-y; }
#Free { left: 100px; top: 100px; }
.tall { height: 2000px; }
.end { position: absolute; top: 1500px; width: 40px; height: 10px; }
.tall { position: relative; }
#Smooth { scroll-behavior: smooth; }
</style>"#;
    let run = |coarse: bool, id: &str, by: Vec2| {
        let mut d = crate::support::Doc::new(text, &[]);
        d.viewport.coarse_pointer = coarse;
        d.step(InputFrame::default());
        let e = d.get(id);
        let from = Vec2::new(e.x + 30.0, e.y + 30.0);
        let at = |d: &mut crate::support::Doc, p: Vec2, down, pressed, released| {
            d.step(InputFrame {
                pointer: p,
                pointer_inside: true,
                down,
                pressed,
                released,
                ..InputFrame::default()
            });
        };
        at(&mut d, from, true, true, false);
        for k in 1..=8 {
            at(&mut d, from + by * (k as f32 / 8.0), true, false, false);
        }
        // Read while still held: the release may fling on.
        let path = d.get("Pan").path;
        let s = d.ui.scroll_state(&path).unwrap();
        at(&mut d, from + by, false, false, true);
        (s.offset_x, s.offset_y)
    };
    // A finger moving up and left by 40: the content's offset grows by 40
    // on each axis it may pan.
    let finger = Vec2::new(-40.0, -40.0);
    assert_eq!(run(true, "Free", finger), (40.0, 40.0), "auto pans both");
    assert_eq!(run(true, "Y", finger), (0.0, 40.0), "pan-y");
    assert_eq!(
        run(true, "L", finger),
        (40.0, 0.0),
        "pan-left, finger moving left"
    );
    assert_eq!(
        run(true, "L", Vec2::new(40.0, -40.0)).0,
        0.0,
        "pan-left drops a finger moving right"
    );
    assert_eq!(
        run(true, "XY", finger),
        (0.0, 0.0),
        "pan-y inside pan-x is none"
    );
    assert_eq!(
        run(false, "Y", finger),
        (40.0, 40.0),
        "a mouse drag ignores touch-action"
    );
    // scroll-behavior: the same scroll into view, smooth and auto.
    let glide = |end: &str, scroller: &str| {
        let mut d = crate::support::Doc::new(text, &[]);
        d.step(InputFrame::default());
        let path = format!("{}/{scroller}Tall/{end}", d.get(scroller).path);
        d.ui.scroll_into_view_path(&path, false);
        let scroller = d.get(scroller).path;
        (0..180)
            .map(|_| {
                d.step(InputFrame::default());
                d.ui.scroll_state(&scroller).unwrap().offset_y
            })
            .collect::<Vec<_>>()
    };
    // `auto` is at its end on the first tick: the button's bottom (1510)
    // at the view's (200 down), with the focus ring's room.
    let jump = glide("JumpEnd", "Jump");
    let target = jump[0];
    assert!(
        (1310.0..1320.0).contains(&target) && jump.iter().all(|&y| y == target),
        "auto jumps: {:?}",
        &jump[..4]
    );
    let smooth = glide("SmoothEnd", "Smooth");
    assert!(
        smooth[0] > 0.0 && smooth[0] < target * 0.5,
        "smooth starts gliding: {:?}",
        &smooth[..4]
    );
    assert!(
        smooth.windows(2).all(|w| w[1] >= w[0]),
        "monotonic: {smooth:?}"
    );
    // CSSOM View leaves the curve to the UA: it lands (to f32 rounding)
    // within three seconds and stays.
    let end = *smooth.last().unwrap();
    assert!((end - target).abs() < 0.01, "lands: {end} vs {target}");
    assert!(
        smooth[150..].iter().all(|&y| y == end),
        "settles: {:?}",
        &smooth[150..]
    );
}
