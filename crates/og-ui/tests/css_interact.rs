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
#Under { width: 200px; height: 60px; cursor: url(arrow) 3 4, url(7), crosshair; }
#Over { position: absolute; top: 0; inset-inline-start: 0; width: 300px; height: 60px; pointer-events: none; cursor: wait; }
#Again { position: absolute; top: 0; inset-inline-start: 220px; width: 60px; height: 60px; pointer-events: auto; cursor: copy; }
#Ghost { visibility: hidden; cursor: no-drop; }
#Lost { width: 100px; height: 40px; }
#Shown { width: 100px; height: 40px; visibility: visible; cursor: url(grip), auto; }
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
        [("arrow".into(), Some([3.0, 4.0])), ("7".into(), None)]
    );
    // Its child that says auto takes the pointer again.
    frame(&mut ui, again);
    assert_eq!(frame(&mut ui, again), Cursor::Copy);
    // A hidden button takes nothing: the pointer is on the root.
    frame(&mut ui, lost);
    assert_eq!(frame(&mut ui, lost), Cursor::Default);
    // A visible child of a hidden parent takes it. Images over `auto` are
    // its own value: the keyword is its kind's, not its parent's no-drop.
    frame(&mut ui, shown);
    assert_eq!(frame(&mut ui, shown), Cursor::Default);
    assert_eq!(urls(&ui), [("grip".into(), None)]);
    let a11y = format!("{:?}", ui.a11y());
    assert!(a11y.contains("Shown") && !a11y.contains("Lost"), "{a11y}");
}

#[test]
fn images_take_object_fit_position_and_inherited_image_rendering() {
    struct Env(og_render::TextureHandle);
    impl bind::Env for Env {
        fn texture(&self, _: f64) -> Option<og_render::TextureHandle> {
            Some(self.0)
        }
    }
    let mut scene = og_render::RenderScene::new();
    let t = scene.add_texture(og_render::TextureData {
        width: 2,
        height: 2,
        rgba: vec![255; 16],
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    });
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
                og_render::UiPaint::Image(im) => Some(*im),
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
    assert_eq!(im.fit, og_render::UiFit::Contain);
    assert_eq!(
        (im.focal, im.offset),
        (Vec2::new(1.0, 0.0), Vec2::new(-10.0, 0.0))
    );
    assert!(im.nearest);
}
