//! Mounted screens as layers (docs/ui.md §2.1): a HUD component and a
//! pause screen mounted with a store input, the pause menu a modal layer
//! over the dimmed HUD, and the same menu leaving under its CSS view
//! transition's `-og-iris` cover on `::view-transition-old(root)`. The frames match
//! `references/vulkan/mounted-layers-{1280,390}.png` and
//! `mounted-layers-iris-{1280,390}.png` (`OG_UI_CAPTURE=1` records them).
//!
//! A third frame, `mounted-layers-morph-{1280,390}.png`, is a shared
//! element halfway through its view transition: a list card named `hero`
//! morphing into the detail screen's larger card, both images cross-faded
//! (plus-lighter) on the group's box, the live cards hidden.
//!
//! Ways it could fail: the mounted screen drawn as a plain overlay (no dim
//! under it), its store input unread (the volume line empty), or the cover
//! drawn over nothing because the leaving layer was removed at once.

use crate::support;

use og_ui::*;
use serde_json::json;
use std::collections::BTreeMap;
use support::*;

const HUD: &str = r#"<meta name="component" content="hud">
<row id="Hud"><text id="Gold">1,250 gold</text><spacer/><text id="Day">Day 3</text></row>
<style>#Hud { padding: 16px; gap: 12px; }</style>"#;

const PAUSE: &str = r#"<meta name="screen" content="pause" data-layer="modal" data-pause="true" data-focus="Menu/Resume">
<column id="Center">
<panel id="Menu">
  <h2 id="Title">Paused</h2>
  <p id="Vol">Volume {$volume}</p>
  <button id="Resume" variant="primary">Resume</button>
  <button id="Quit">Quit to title</button>
</panel>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Menu { width: 320px; padding: 24px; gap: 12px; }
#Title, #Vol { margin: 0; }
@view-transition { navigation: auto; }
::view-transition-old(root), ::view-transition-new(root) { animation: 400ms -og-iris; }
::view-transition { background-color: #0b1220; }
</style>"#;

const LIST: &str = r#"<meta name="screen" content="list">
<column id="Page"><panel id="Card"><h3 id="Name">Ember blade</h3><p id="Kind">Sword, rare</p></panel></column>
<style>
@view-transition { navigation: auto; }
#Page { width: 100%; height: 100%; padding: 32px; }
#Card { width: 240px; padding: 16px; gap: 4px; view-transition-name: hero; }
#Name, #Kind { margin: 0; }
::view-transition-group(hero) { animation-duration: 600ms; animation-timing-function: linear; }
</style>"#;

const DETAIL: &str = r#"<meta name="screen" content="detail">
<column id="Page"><panel id="Card"><h2 id="Name">Ember blade</h2><p id="Kind">Sword, rare, 42 attack</p></panel></column>
<style>
@view-transition { navigation: auto; }
#Page { width: 100%; height: 100%; padding: 32px; align-items: center; justify-content: center; }
#Card { width: 320px; padding: 32px; gap: 12px; view-transition-name: hero; }
#Name, #Kind { margin: 0; }
::view-transition-group(hero) { animation-duration: 600ms; animation-timing-function: linear; }
</style>"#;

fn send(ui: &mut Ui, command: serde_json::Value) {
    let reply = reactive::wire::decode(
        &ui.apply_reactive(&reactive::wire::encode(&json!([command])).unwrap()),
    )
    .unwrap();
    assert_eq!(reply[0]["ok"], true, "{reply}");
}

#[test]
#[ignore = "requires GPU"]
fn mounted_layers_match_their_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font.clone());
        ui.define_document(UiDocument::parse("ui/hud.svelte", HUD).unwrap());
        ui.define_document(UiDocument::parse("ui/pause.svelte", PAUSE).unwrap());
        let volume = reactive::SourceId::new("test", "volume");
        send(
            &mut ui,
            json!({"op": "signal", "source": volume, "initial": "80%"}),
        );
        let mount = |ui: &mut Ui, id: &str, doc: &str| {
            ui.mount_document(
                reactive::SourceId::new("test", id),
                doc,
                BTreeMap::from([("volume".to_string(), volume.clone())]),
                Default::default(),
                &bind::NoEnv,
            )
            .unwrap();
        };
        mount(&mut ui, "hud", "hud");
        mount(&mut ui, "pause", "pause");
        let vp = Viewport::new(width as f32, height as f32);
        let step = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.build_layers(&bind::NoEnv);
            ui.end();
        };
        // The join's cover has opened.
        for _ in 0..60 {
            step(&mut ui);
        }
        assert_eq!(ui.layer_stack().len(), 1);
        let mut frames = vec![("mounted-layers", capture(&ui, &mut scene, width, height))];
        // Leaving: the iris closes over the menu, a third of the way.
        ui.unmount_document(&reactive::SourceId::new("test", "pause"));
        for _ in 0..5 {
            step(&mut ui);
        }
        assert_eq!(
            ui.layer_stack().len(),
            1,
            "the leaving menu stays under its cover"
        );
        frames.push((
            "mounted-layers-iris",
            capture(&ui, &mut scene, width, height),
        ));
        // A shared element halfway from the list's card to the detail's.
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/list.svelte", LIST).unwrap());
        ui.define_document(UiDocument::parse("ui/detail.svelte", DETAIL).unwrap());
        ui.push_screen("list", serde_json::Value::Null).unwrap();
        for _ in 0..30 {
            step(&mut ui);
        }
        ui.replace_screen("detail", serde_json::Value::Null)
            .unwrap();
        for _ in 0..=ui.ticks_for(300.0) {
            step(&mut ui);
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        frames.push((
            "mounted-layers-morph",
            capture(&ui, &mut scene, width, height),
        ));
        for (stem, (pixels, info)) in frames {
            let name = format!("{stem}-{width}.png");
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
    scene.overlay = Some(overlay);
    let bg = ui.theme().palette.backdrop.linear();
    scene.environment.background = Background::Color(og_core::glam::Vec3::new(bg[0], bg[1], bg[2]));
    scene.post.tonemap = Tonemap::None;
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
