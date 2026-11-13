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
use og_ui::*;
use serde_json::json;
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

fn frame(ui: &mut Ui, p: Vec2) -> Cursor {
    ui.begin(
        InputFrame {
            pointer: p,
            pointer_inside: true,
            ..InputFrame::default()
        },
        Viewport::new(800.0, 600.0),
    );
    ui.component("T", "ix", &mut json!({}), &bind::NoEnv);
    ui.end();
    assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    ui.frame().cursor
}

#[test]
fn the_pointer_passes_through_none_and_hidden() {
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/ix.svelte", DOC).expect("parses"));
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
    for retained in [false, true] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/cursors.svelte", doc).unwrap());
        if retained {
            ui.mount_document(
                reactive::SourceId::new("test", "cursor"),
                "cursors",
                Default::default(),
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        }
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
                if retained {
                    ui.build_retained_documents(&bind::NoEnv).unwrap();
                } else {
                    ui.component("T", "cursors", &mut json!({}), &bind::NoEnv);
                }
                ui.end();
            }
            assert_eq!(
                ui.frame().cursor,
                expected,
                "retained={retained}, ({x}, {y})"
            );
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
    for retained in [false, true] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("ui/cuts.svelte", doc).unwrap());
        if retained {
            ui.mount_document(
                reactive::SourceId::new("test", "cuts"),
                "cuts",
                Default::default(),
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        }
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
                if retained {
                    ui.build_retained_documents(&bind::NoEnv).unwrap();
                } else {
                    ui.component("T", "cuts", &mut json!({}), &bind::NoEnv);
                }
                ui.end();
            }
            assert_eq!(
                ui.frame().cursor,
                expected,
                "retained={retained}, ({x}, {y})"
            );
            assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        }
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
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
        ui.component("T", "im", &mut json!({}), &Env(t));
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
    let mut ui = ui_with_font(test_font_metrics());
    ui.define_document(UiDocument::parse("ui/ix.svelte", PIN).expect("parses"));
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
    for retained in [false, true] {
        let mut ui = ui_with_font(test_font_metrics());
        ui.define_document(UiDocument::parse("@ui/test/ix.svelte", text).unwrap());
        if retained {
            ui.mount_document(
                reactive::SourceId::new("test", "none"),
                "ix",
                Default::default(),
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        }
        ui.begin(InputFrame::default(), Viewport::new(400.0, 400.0));
        if retained {
            ui.build_retained_documents(&bind::NoEnv).unwrap();
        } else {
            ui.component("T", "ix", &mut json!({}), &bind::NoEnv);
        }
        ui.end();
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        for id in ["Percent", "Length"] {
            let node = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.id == id)
                .unwrap();
            assert_eq!(
                (node.x, node.y),
                (100.0, 150.0),
                "{id}, retained={retained}"
            );
        }
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
    let mut d = crate::support::Doc::new(text, &[], true);
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
