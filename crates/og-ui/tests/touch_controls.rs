//! On-screen controls in documents (docs/ui.md §3.3): a `<joystick>`
//! held by one finger and pushed up and right, a `<dpad>` held right by a
//! second, and two touch `<button data-og-touch>`s CSS sizes, laid
//! out by CSS at the screen's corners, shown on a coarse pointer only. The
//! frames match `references/vulkan/touch-controls-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: the knob not following a second finger (a document
//! element built once), the stick sized by its touch-target minimum
//! instead of its CSS width, `@media (pointer: coarse)` not matching, or
//! the corners placed from the wrong edges.

use crate::support;

use og_core::glam::Vec2;
use og_ui::*;
use support::*;

const HUD: &str = r##"<meta name="component" content="hud">
<div id="Root">
  <joystick id="Stick" action="move"/>
  <dpad id="Pad" action="look"/>
  <button id="Jump" class="touch" data-og-touch="jump">Jump</button>
  <button id="Fire" class="touch" data-og-touch="fire">Fire</button>
</div>
<style>
#Root { padding: 0; width: 100%; height: 100%; }
#Stick, #Pad, .touch { position: absolute; }
#Stick { inset-inline-start: 32px; bottom: 32px; width: 160px; }
#Pad { inset-inline-start: 224px; bottom: 44px; }
.touch { width: 80px; height: 80px; }
#Jump { inset-inline-end: 32px; bottom: 40px; }
#Fire { inset-inline-end: 128px; bottom: 96px; }
@media (pointer: fine) { #Stick, #Pad, .touch { display: none; } }
@media (max-width: 500px) {
  #Stick { width: 132px; inset-inline-start: 20px; }
  #Pad { inset-inline-start: 20px; bottom: 196px; }
  #Jump { inset-inline-end: 20px; }
  #Fire { inset-inline-end: 112px; }
}
</style>"##;

fn held(touches: &[(u32, Vec2)]) -> og_input::Pointers {
    let mut p = og_input::Pointers::default();
    for &(id, at) in touches {
        p.held |= 1 << id;
        let (pos, start) = match id {
            1 => (&mut p.position1, &mut p.start1),
            _ => (&mut p.position2, &mut p.start2),
        };
        (*pos, *start) = (at, at);
    }
    p.pressed = p.held;
    p
}

#[test]
#[ignore = "requires GPU"]
fn touch_controls_match_their_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/hud.svelte", HUD).unwrap());
        ui.mount_document(
            reactive::SourceId::new("test", "hud"),
            "hud",
            Default::default(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let mut vp = Viewport::new(width as f32, height as f32);
        vp.coarse_pointer = true;
        let run = |ui: &mut Ui, p: og_input::Pointers| {
            ui.begin(
                InputFrame {
                    pointers: p,
                    ..Default::default()
                },
                vp,
            );
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        };
        for _ in 0..3 {
            run(&mut ui, og_input::Pointers::default());
        }
        let centre = |ui: &Ui, path: &str| {
            let e = ui
                .probe_elements()
                .into_iter()
                .find(|e| e.path == path)
                .unwrap_or_else(|| panic!("no {path}"));
            Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0)
        };
        let stick = centre(&ui, "test:hud/Stick");
        let pad = centre(&ui, "test:hud/Pad");
        // Down on each, then pushed: the stick up and right, the pad right.
        let down = held(&[(1, stick), (2, pad)]);
        run(&mut ui, down);
        let mut moved = down;
        moved.position1 = stick + Vec2::new(40.0, -30.0);
        moved.position2 = pad + Vec2::new(40.0, 0.0);
        moved.pressed = 0;
        for _ in 0..2 {
            run(&mut ui, moved);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let zones = ui.take_control_zones().expect("zones");
        assert_eq!(zones.len(), 4, "{zones:?}");
        let (pixels, info) = capture(&ui, &mut scene, width, height);
        let name = format!("touch-controls-{width}.png");
        image::save_buffer(
            out_dir().join(&name),
            &pixels,
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
        let own = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("references")
            .join(og_tests::compare::RECORDING_TIER)
            .join(&name);
        if std::env::var_os("OG_UI_CAPTURE").is_some() {
            record_reference(&own, width, height, &pixels);
        }
        let expected = image::open(&own)
            .unwrap_or_else(|e| panic!("{}: {e} (OG_UI_CAPTURE=1 records it)", own.display()))
            .to_rgba8();
        let (rmse, _) = compare(&pixels, expected.as_raw(), &info, &own)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("{name}: RMSE {rmse:.6}");
    }
}

fn capture(
    ui: &Ui,
    scene: &mut og_render::RenderScene,
    width: u32,
    height: u32,
) -> (Vec<u8>, og_render::RenderInfo) {
    use og_render::*;
    let mut overlay = Overlay::new(width as f32, height as f32, 1.0);
    ui.frame().emit(&EmitDesc::default(), &mut overlay);
    scene.ui.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
    scene.look.post.tonemap = Tonemap::None;
    let mut renderer = Renderer::new_blocking(&RenderDesc {
        width,
        height,
        ..Default::default()
    })
    .expect("a headless adapter");
    let stats = renderer.render(scene, &RenderTarget::Offscreen).unwrap();
    assert_eq!(stats.missing_glyphs, 0);
    (
        renderer.read_offscreen().unwrap().pixels,
        renderer.info().clone(),
    )
}
