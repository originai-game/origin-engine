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
<div id="Root"><img id="Pic" src={1} alt="" /><img id="Plain" src={1} alt="" /><img id="Smooth" src={1} alt="" /></div>
<style>
#Root { image-rendering: pixelated; }
#Pic { width: 100px; height: 50px; object-fit: contain; object-position: right 10px top; }
#Plain { width: 10px; height: 10px; image-rendering: auto; }
#Smooth { width: 40px; height: 40px; object-fit: cover; image-rendering: smooth; }
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
    // stretched image (not a shape); `smooth` under the same pixelated
    // parent samples smoothly again (CSS Images 3 §5.3).
    assert_eq!(images.len(), 2, "{images:?}");
    let im = images[0];
    assert_eq!(im.fit, og_draw::UiFit::Contain);
    assert_eq!(
        (im.focal, im.offset),
        (Vec2::new(1.0, 0.0), Vec2::new(-10.0, 0.0))
    );
    assert!(im.nearest);
    assert_eq!(images[1].fit, og_draw::UiFit::Cover);
    assert!(!images[1].nearest, "smooth resets the inherited pixelated");
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

/// A hidden backface and the children composited into its layer take no
/// pointer; a visible backface and a front-facing hidden-backface box do.
#[test]
fn hidden_backfaces_pass_the_pointer_through_their_layer() {
    let mut ui = mounted(
        "ui/ix.svelte",
        r#"<meta name="component" content="ix">
<div id="Root">
  <div class="plane hidden"><button id="Hidden">Hidden child</button></div>
  <div class="plane visible"><button id="Visible">Visible back</button></div>
  <div class="plane front"><button id="Front">Front</button></div>
</div>
<style>
#Root { width: 600px; height: 300px; cursor: help; }
.plane { position: absolute; width: 100px; height: 100px; top: 20px; transform: rotateY(180deg); }
.hidden { left: 20px; backface-visibility: hidden; cursor: wait; }
.visible { left: 160px; backface-visibility: visible; }
.front { left: 300px; backface-visibility: hidden; transform: rotateY(0deg); }
button { width: 100px; height: 100px; cursor: copy; }
button:hover { height: 80px; }
</style>"#,
    );
    for (id, x, cursor, height) in [
        ("Hidden", 70.0, Cursor::Help, 100.0),
        ("Visible", 210.0, Cursor::Copy, 80.0),
        ("Front", 350.0, Cursor::Copy, 80.0),
        ("Hidden", 70.0, Cursor::Help, 100.0),
    ] {
        for _ in 0..2 {
            assert_eq!(frame(&mut ui, Vec2::new(x, 70.0)), cursor);
        }
        assert_eq!(
            ui.probe_elements()
                .iter()
                .find(|e| e.id == id)
                .unwrap()
                .height,
            height,
            "{id} hover follows hit testing"
        );
    }
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

/// CSS Scroll Anchoring 1: a scrolled container keeps what it shows in
/// place when content above it changes. In a 100px scroller of 40px rows
/// scrolled to 100, the anchor is R2 (rows 80–120: the first box at least
/// partly in view, with nothing inside it). A 50px block inserted above
/// moves the offset to exactly 150 and leaves R3 drawn where it was, and
/// removing it brings the offset back to 100. Not adjusted: under
/// `overflow-anchor: none` on the container (R3 moves down 50), at offset 0
/// (§2.1 step 2), or when the anchor grows below its own top (R3 moves
/// down 40). With `overflow-anchor: none` on R2 the anchor is R3, so R2
/// growing by 40 moves the offset by 40 instead. Ways it could fail: no
/// anchoring, the adjustment applied twice or a frame late, the anchor's
/// growth counted as movement, offset 0 anchored, or `none` ignored on
/// either side.
#[test]
fn scroll_anchoring_keeps_the_view_when_content_above_changes() {
    let doc = |scroller: &str, r2: &str| {
        let text = format!(
            r#"<meta name="component" content="t">
<div><div id="S" style="{scroller}"><div id="Ins" class={{$ins}}></div><div id="R0" class="r"></div><div id="R1" class="r"></div><div id="R2" class={{$r2}} style="{r2}"></div><div id="R3" class="r"></div><div id="R4" class="r"></div><div id="R5" class="r"></div><div id="R6" class="r"></div><div id="R7" class="r"></div></div></div>
<style>
#S {{ width: 200px; height: 100px; overflow: auto; scrollbar-width: none; }}
#Ins {{ height: 0px; }}
#Ins.on {{ height: 50px; }}
.r, #R2 {{ height: 40px; flex-shrink: 0; }}
#R2.big {{ height: 80px; }}
</style>"#
        );
        let mut d = crate::support::Doc::new(
            &text,
            &[
                ("ins", serde_json::json!("")),
                ("r2", serde_json::json!("")),
            ],
        );
        d.step(InputFrame::default());
        d
    };
    let wheel = |d: &mut crate::support::Doc, by: f32| {
        let s = d.get("S");
        d.step(InputFrame {
            pointer: Vec2::new(s.x + 50.0, s.y + 50.0),
            pointer_inside: true,
            wheel: Vec2::new(0.0, by),
            ..InputFrame::default()
        });
        d.step(InputFrame::default());
    };
    let offset = |d: &crate::support::Doc| {
        let path = d.get("S").path;
        d.ui.scroll_state(&path).unwrap().offset_y
    };
    // Anchored: the view stays put both ways.
    let mut d = doc("", "");
    wheel(&mut d, 100.0);
    assert_eq!(offset(&d), 100.0);
    let r3 = d.get("R3").y;
    d.set("ins", serde_json::json!("on"));
    assert_eq!(
        offset(&d),
        150.0,
        "follows the anchor down by the insertion"
    );
    assert_eq!(d.get("R3").y, r3, "R3 stays where it was drawn");
    d.set("ins", serde_json::json!(""));
    assert_eq!(offset(&d), 100.0, "and back up when it goes");
    assert_eq!(d.get("R3").y, r3);
    // The anchor growing below its top moves nothing above it.
    d.set("r2", serde_json::json!("big"));
    assert_eq!(offset(&d), 100.0, "the anchor's own growth is not movement");
    assert_eq!(d.get("R3").y, r3 + 40.0);
    // overflow-anchor: none on the container.
    let mut d = doc("overflow-anchor: none", "");
    wheel(&mut d, 100.0);
    let r3 = d.get("R3").y;
    d.set("ins", serde_json::json!("on"));
    assert_eq!(offset(&d), 100.0, "none: no anchoring");
    assert_eq!(d.get("R3").y, r3 + 50.0);
    // At offset 0 nothing is anchored.
    let mut d = doc("", "");
    let r1 = d.get("R1").y;
    d.set("ins", serde_json::json!("on"));
    assert_eq!(offset(&d), 0.0, "at its start the content moves");
    assert_eq!(d.get("R1").y, r1 + 50.0);
    // overflow-anchor: none on R2 makes R3 the anchor.
    let mut d = doc("", "overflow-anchor: none");
    wheel(&mut d, 100.0);
    let r3 = d.get("R3").y;
    d.set("r2", serde_json::json!("big"));
    assert_eq!(offset(&d), 140.0, "R3 anchors past an excluded R2");
    assert_eq!(d.get("R3").y, r3);
}

/// CSS Overflow 5 §3 scroll markers and buttons, against the spec's
/// numbers. A 200px scroller of four 100px sections: their scroll-into-view
/// positions 0, 100, 200 and 300 over a 200px range are redistributed
/// (§3.1.8 step 5, over min(200/8, 200/2) = 25px at each end) to 0, 100,
/// 180 and 200, so at offset 190 section 2 is current (section 1 without
/// the redistribution) and at 200 section 3. A marker press scrolls its
/// section to the start (section 1: 100) and keeps it current while the
/// offset stays there, even where the algorithm would pick another
/// (section 2 at 200, where section 3 is otherwise current). A button
/// pages by 0.85 of the scrollport (170px) and matches `:disabled` at the
/// end it cannot scroll past. The markers stand in a group before the
/// scroller, in tree order, then the buttons (block-start, block-end). `scroll-target-group: auto`
/// gives `<a href="#id">` links the same states. Ways it could fail: no
/// redistribution, the last-reached rather than the pressed target kept
/// current, a full page, `:disabled` never set or never cleared, markers
/// out of tree order, the states missing from the links, or a restored
/// checkpoint forgetting the pressed target or the states.
#[test]
fn scroll_markers_and_buttons_follow_the_scroll_position() {
    let text = r##"<meta name="component" content="t">
<div>
  <div id="S"><div id="A" class="sec"></div><div id="B" class="sec"></div><div id="C" class="sec"></div><div id="D" class="sec"></div></div>
  <div id="Toc"><a id="LA" href="#A">a</a><a id="LC" href="#C">c</a></div>
</div>
<style>
#S { width: 100px; height: 200px; overflow-y: auto; scrollbar-width: none; scroll-marker-group: before; }
.sec { height: 100px; flex-shrink: 0; }
.sec::scroll-marker { content: ""; width: 10px; height: 10px; }
.sec::scroll-marker:target-before { background: #0000ff; }
.sec::scroll-marker:target-current { background: #00ff00; }
.sec::scroll-marker:target-after { background: #ff0000; }
#S::scroll-button(up) { content: ""; width: 20px; height: 20px; background: #00ffff; }
#S::scroll-button(down) { content: ""; width: 20px; height: 20px; background: #00ffff; }
#S::scroll-button(*):disabled { background: #ffff00; }
#Toc { scroll-target-group: auto; }
#Toc a { display: block; width: 30px; height: 30px; }
#Toc a:target-before { background: #0000ff; }
#Toc a:target-current { background: #00ff00; }
#Toc a:target-after { background: #ff0000; }
</style>"##;
    let mut d = crate::support::Doc::new(text, &[]);
    let settle = |d: &mut crate::support::Doc| {
        for _ in 0..3 {
            d.step(InputFrame::default());
        }
    };
    settle(&mut d);
    let path = d.get("S").path;
    let offset = |d: &crate::support::Doc| d.ui.scroll_state(&path).unwrap().offset_y;
    // The fills of a size, in reading order, as letters: b(efore), c(urrent),
    // a(fter), d(isabled), e(nabled).
    let fills = |d: &crate::support::Doc, size: f32| -> String {
        let mut v: Vec<(f32, char)> =
            d.ui.frame()
                .items
                .iter()
                .filter_map(|i| i.cmd.filled())
                .filter(|(r, _)| r.width == size && r.height == size)
                .filter_map(|(r, c)| {
                    let l = match [c[0], c[1], c[2]] {
                        [0.0, 0.0, 1.0] => 'b',
                        [0.0, 1.0, 0.0] => 'c',
                        [1.0, 0.0, 0.0] => 'a',
                        [1.0, 1.0, 0.0] => 'd',
                        [0.0, 1.0, 1.0] => 'e',
                        _ => return None,
                    };
                    Some((r.y * 1e4 + r.x, l))
                })
                .collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        v.into_iter().map(|(_, l)| l).collect()
    };
    let s = d.get("S");
    assert_eq!(
        s.y, 50.0,
        "the group (a 10px row) and two 20px buttons before it"
    );
    assert_eq!(fills(&d, 10.0), "caaa", "at 0: the first section");
    assert_eq!(fills(&d, 20.0), "de", "up disabled at the top, down not");
    assert_eq!(fills(&d, 30.0), "ca", "the links: A current");
    let wheel = |d: &mut crate::support::Doc, by: f32| {
        d.step(InputFrame {
            pointer: Vec2::new(s.x + 50.0, s.y + 50.0),
            pointer_inside: true,
            wheel: Vec2::new(0.0, by),
            ..InputFrame::default()
        });
        settle(d);
    };
    wheel(&mut d, 100.0);
    assert_eq!(offset(&d), 100.0);
    assert_eq!(fills(&d, 10.0), "bcaa", "at 100: the second");
    assert_eq!(fills(&d, 20.0), "ee", "both ways open");
    assert_eq!(
        fills(&d, 30.0),
        "ca",
        "A, 0, is the largest at or before 100"
    );
    wheel(&mut d, 90.0);
    assert_eq!(offset(&d), 190.0);
    assert_eq!(
        fills(&d, 10.0),
        "bbca",
        "at 190: the third, at its redistributed 180"
    );
    assert_eq!(
        fills(&d, 30.0),
        "bc",
        "C at 200: its smaller neighbour 0 is over half a port back, and 200 within half ahead"
    );
    wheel(&mut d, 10.0);
    assert_eq!(offset(&d), 200.0);
    assert_eq!(fills(&d, 10.0), "bbbc", "at the end: the last, at 200");
    assert_eq!(fills(&d, 20.0), "ed", "down disabled at the end");
    // A marker press: the section scrolls to the start and is current.
    d.click_at(Vec2::new(15.0, 5.0));
    settle(&mut d);
    assert_eq!(
        offset(&d),
        100.0,
        "the second marker scrolls B to the start"
    );
    assert_eq!(fills(&d, 10.0), "bcaa");
    d.click_at(Vec2::new(25.0, 5.0));
    settle(&mut d);
    assert_eq!(offset(&d), 200.0, "C is 200 down");
    assert_eq!(
        fills(&d, 10.0),
        "bbca",
        "the pressed C stays current where D would be"
    );
    // A checkpoint keeps the pressed target current and the states drawn
    // on the first frame after it is restored.
    let saved = d.ui.save_state();
    d.click_at(Vec2::new(5.0, 5.0));
    settle(&mut d);
    d.ui.restore_state(&saved).unwrap();
    d.step(InputFrame::default());
    assert_eq!(offset(&d), 200.0);
    assert_eq!(
        (fills(&d, 10.0).as_str(), fills(&d, 20.0).as_str()),
        ("bbca", "ed"),
        "restored"
    );
    // The buttons: up a page of 0.85 × 200.
    d.click_at(Vec2::new(10.0, 20.0));
    settle(&mut d);
    assert_eq!(offset(&d), 30.0, "200 − 170");
    assert_eq!(fills(&d, 10.0), "caaa", "the pin let go: at 30, A");
    d.click_at(Vec2::new(10.0, 40.0));
    settle(&mut d);
    assert_eq!(offset(&d), 200.0, "30 + 170, clamped to the range");
    assert_eq!(fills(&d, 20.0), "ed");
}

/// CSS Overflow 5 §4 `scroll-axis-lock`: a drag on a scroller with room
/// both ways keeps to the axis its first move runs along when that move is
/// within a 2:1 slope of it, the other axis's delta dropped for the whole
/// press, even once the drag turns; a steeper diagonal stays free, as does
/// every drag under `scroll-axis-lock: none`. Ways it could fail: no lock, the lock decided
/// again each tick (so the turn moves the other axis), the slope test
/// inverted, `none` ignored, or the lock kept into the next press.
#[test]
fn scroll_axis_lock_keeps_a_drag_to_the_axis_it_starts_along() {
    let text = r#"<meta name="component" content="t">
<div id="Root">
  <div id="Lock" class="s"><div class="big"></div></div>
  <div id="Free" class="s" style="scroll-axis-lock: none"><div class="big"></div></div>
</div>
<style>
#Root { width: 800px; display: flex; flex-direction: row; align-items: start; gap: 10px; }
.s { width: 200px; height: 200px; overflow: auto; scrollbar-width: none; }
.big { width: 1000px; height: 1000px; }
</style>"#;
    // The scroller's offset after a pressed pointer follows `path` (each
    // point relative to the press, one per tick), read while still held.
    let drag = |d: &mut crate::support::Doc, id: &str, path: &[Vec2]| {
        let e = d.get(id);
        let from = Vec2::new(e.x + 100.0, e.y + 100.0);
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
        let path_of = d.get(id).path;
        let before = d.ui.scroll_state(&path_of).unwrap();
        at(d, from, true, true, false);
        for &p in path {
            at(d, from + p, true, false, false);
        }
        let s = d.ui.scroll_state(&path_of).unwrap();
        at(d, from + *path.last().unwrap(), false, false, true);
        // Let any fling settle out of the next drag's way.
        for _ in 0..120 {
            d.step(InputFrame::default());
        }
        (s.offset_x - before.offset_x, s.offset_y - before.offset_y)
    };
    let line = |to: Vec2| (1..=8).map(|k| to * (k as f32 / 8.0)).collect::<Vec<_>>();
    let fresh = || {
        let mut d = crate::support::Doc::new(text, &[]);
        d.step(InputFrame::default());
        d
    };
    // A pointer moving up and left: the content's offset grows.
    // 40 across, 19 down: a slope under 1:2, so it keeps to x.
    let mut d = fresh();
    assert_eq!(
        drag(&mut d, "Lock", &line(Vec2::new(-40.0, -19.0))),
        (40.0, 0.0),
        "mostly across"
    );
    let mut d = fresh();
    assert_eq!(
        drag(&mut d, "Lock", &line(Vec2::new(-19.0, -40.0))),
        (0.0, 40.0),
        "mostly down"
    );
    // 40 by 21: past 2:1, so free.
    let mut d = fresh();
    assert_eq!(
        drag(&mut d, "Lock", &line(Vec2::new(-40.0, -21.0))),
        (40.0, 21.0),
        "diagonal"
    );
    // Locked to x by its first move, then turning straight down: y never moves.
    let mut d = fresh();
    let turn = [
        Vec2::new(-10.0, 0.0),
        Vec2::new(-20.0, 0.0),
        Vec2::new(-20.0, -20.0),
        Vec2::new(-20.0, -50.0),
    ];
    assert_eq!(
        drag(&mut d, "Lock", &turn),
        (20.0, 0.0),
        "the lock holds through a turn"
    );
    // The next press decides afresh: the same document, now down first.
    assert_eq!(
        drag(&mut d, "Lock", &line(Vec2::new(0.0, -30.0))),
        (0.0, 30.0),
        "a new press is not kept to the last one's axis"
    );
    // scroll-axis-lock: none never locks.
    let mut d = fresh();
    assert_eq!(
        drag(&mut d, "Free", &line(Vec2::new(-40.0, -10.0))),
        (40.0, 10.0),
        "none"
    );
    let mut d = fresh();
    assert_eq!(
        drag(&mut d, "Free", &turn),
        (20.0, 50.0),
        "none follows the turn"
    );
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

/// Media Queries 5 §6.5 `video-dynamic-range`: `standard` matches on
/// every surface, `high` only where the surface presents extended range
/// (`Viewport::high_range`), and a change of surface restyles at once.
/// Ways it could fail: `high` matching a standard surface, `standard`
/// failing on a high one (it is a minimum, not an exact match), or the
/// query kept from the first frame.
#[test]
fn video_dynamic_range_follows_the_surface() {
    let text = r#"<meta name="component" content="t">
<div id="Root"><div id="Std"></div><div id="High"></div></div>
<style>
#Std, #High { width: 10px; height: 10px; }
@media (video-dynamic-range: standard) { #Std { width: 20px; } }
@media (video-dynamic-range: high) { #High { width: 30px; } }
</style>"#;
    let mut d = crate::support::Doc::new(text, &[]);
    let widths = |d: &crate::support::Doc| (d.get("Std").width, d.get("High").width);
    assert_eq!(widths(&d), (20.0, 10.0), "standard surface");
    d.viewport.high_range = true;
    d.step(InputFrame::default());
    assert_eq!(widths(&d), (20.0, 30.0), "extended-range surface");
    d.viewport.high_range = false;
    d.step(InputFrame::default());
    assert_eq!(widths(&d), (20.0, 10.0), "back to standard");
}

/// `::placeholder { color }` inks an empty field's placeholder text and
/// not its value, whatever colour the field itself sets (the placeholder
/// takes its pseudo-element's colour, not the field's). Ways it could
/// fail: the field's `color` repainting the placeholder (the bug this
/// fixes), the typed value drawn in the placeholder's colour, or a text
/// area treated otherwise.
#[test]
fn placeholder_takes_its_pseudo_elements_color() {
    let text = r#"<meta name="component" content="t">
<div id="Root">
  <input id="Empty" placeholder="Name" />
  <input id="Full" value="Ada" placeholder="Name" />
  <textarea id="Area" placeholder="Notes"></textarea>
</div>
<style>
input, textarea { width: 200px; color: #0000ff; }
input::placeholder, textarea::placeholder { color: #ff0000; }
</style>"#;
    let d = crate::support::Doc::with_font(text, &[], test_font());
    let ink = |id: &str| {
        let e = d.get(id);
        d.ui.frame()
            .items
            .iter()
            .map(|i| match &i.cmd {
                DrawCmd::StyledText { text, .. } => text.as_ref(),
                cmd => cmd,
            })
            .filter_map(|cmd| match cmd {
                DrawCmd::Text { color, bounds, .. }
                    if bounds.y >= e.y - 1.0 && bounds.y < e.y + e.height =>
                {
                    Some(*color)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let (red, blue) = (
        Srgba::rgb(255, 0, 0).linear(),
        Srgba::rgb(0, 0, 255).linear(),
    );
    assert_eq!(ink("Empty"), [red], "the placeholder");
    assert_eq!(ink("Full"), [blue], "the value");
    assert_eq!(ink("Area"), [red], "a text area's placeholder");
}

/// CSS UI 4 §6.1: `interactivity: inert` makes an element and its subtree
/// inert, so a button inside it sends nothing when pressed; HTML's `inert`
/// attribute is the UA rule `[inert] { interactivity: inert }`, so a
/// declared `interactivity: auto` on an `[inert]` box makes its button
/// answer again, and a button outside both answers as ever.
///
/// Ways it could fail: the property refused or ignored, the box inert but
/// not its descendants, the attribute winning over a declaration, or every
/// button going quiet.
#[test]
fn interactivity_inert_stops_a_subtree_and_overrides_the_attribute() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <div id="Locked" class="locked"><button id="Buy" onclick={buy}>Buy</button></div>
  <div id="Back" inert><button id="Sell" onclick={sell}>Sell</button></div>
  <div id="Gone" inert><button id="Drop" onclick={drop}>Drop</button></div>
  <button id="Free" onclick={free}>Free</button>
</div>
<style>
#Page { padding: 20px; gap: 20px; }
.locked { interactivity: inert; }
#Back { interactivity: auto; }
</style>"#;
    let mut d = crate::support::Doc::new(text, &[]);
    d.step(InputFrame::default());
    let sent = |d: &mut crate::support::Doc, id: &str| {
        d.click(id)
            .into_iter()
            .filter(|e| e.kind == "emit")
            .map(|e| e.name)
            .collect::<Vec<_>>()
    };
    assert_eq!(sent(&mut d, "Buy"), Vec::<String>::new(), "inert by CSS");
    assert_eq!(sent(&mut d, "Sell"), vec!["sell"], "auto overrides [inert]");
    assert_eq!(sent(&mut d, "Drop"), Vec::<String>::new(), "[inert] alone");
    assert_eq!(sent(&mut d, "Free"), vec!["free"]);
}

/// CSS Scroll Snap 2 §3: a scroll container starts scrolled to its first
/// `scroll-initial-target: nearest` descendant, aligned as a scroll into
/// view with `start`: in a 100px-tall scroller of 40px rows, `Six` starts
/// at 200px, less its 10px `scroll-margin-top` and the container's 5px
/// `scroll-padding-top`, so the offset is 185 and `Six` sits 15px into
/// the view from the very first frame; `Eight`, later in tree order,
/// loses. `End`'s target near the bottom clamps to the range (300). A
/// target that appears in a container already shown does not move it
/// (here one whose -40px scroll-margin would ask for 40): the position is
/// only an initial one.
///
/// Ways it could fail: the property refused or ignored, the first frame
/// drawn at 0 and corrected a frame later, the last target winning, the
/// margin or padding dropped, no clamp, or a later target scrolling an
/// existing container.
#[test]
fn a_scroller_starts_at_its_initial_target() {
    let text = r#"<meta name="component" content="t">
<div id="Page">
  <div id="List" class="list">
    <div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div>
    <div id="Six" class="row target"></div><div class="row"></div><div id="Eight" class="row target"></div><div class="row"></div><div class="row"></div>
  </div>
  <div id="Tail" class="list">
    <div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div>
    <div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div><div id="End" class="row target"></div>
  </div>
  <div id="Later" class="list">
    {#if $more}<div id="Added" class="row target"></div>{/if}
    <div class="row"></div><div class="row"></div><div class="row"></div><div class="row"></div>
    <div class="row"></div><div class="row"></div><div class="row"></div>
  </div>
</div>
<style>
#Page { padding: 20px; gap: 20px; }
.list { height: 100px; width: 200px; overflow-y: auto; scroll-padding-top: 5px; }
.row { height: 40px; flex-shrink: 0; }
.target { scroll-initial-target: nearest; scroll-margin-top: 10px; }
#Later { scroll-padding-top: 0; }
#Added { scroll-margin-top: -40px; }
</style>"#;
    let text = text
        .replace("<div id=\"Page\">", "<div>{#if $show}<div id=\"Page\">")
        .replace("<style>", "{/if}</div><style>");
    for (screen, later) in [(false, false), (true, false), (true, true)] {
        let text = if screen {
            text.replace(
                "name=\"component\" content=\"t\"",
                "name=\"screen\" content=\"t\" data-layer=\"menu\"",
            )
        } else {
            text.to_owned()
        };
        let mut d = crate::support::Doc::new(
            &text,
            &[
                ("more", serde_json::json!(false)),
                ("show", serde_json::json!(!later)),
            ],
        );
        if later {
            d.put("show", serde_json::json!(true));
            d.step(InputFrame::default());
        }
        let offset = |d: &crate::support::Doc, id: &str| {
            let path = d.get(id).path;
            d.ui.scroll_state(&path)
                .expect("a scroll container")
                .offset_y
        };
        // The first frame shown, in flow or in a screen layer.
        assert_eq!(offset(&d, "List"), 185.0, "screen={screen}, later={later}");
        let (list, six) = (d.get("List"), d.get("Six"));
        assert_eq!(six.y - list.y, 15.0, "drawn there on its first frame");
        assert_eq!(offset(&d, "Tail"), 300.0, "clamped to its range");
        assert_eq!(offset(&d, "Later"), 0.0);
        d.set("more", serde_json::json!(true));
        d.step(InputFrame::default());
        assert!(d.el("Added").is_some());
        assert_eq!(offset(&d, "Later"), 0.0, "only an initial position");
    }
}
