//! Mounted screens as layers (docs/ui.md §2.1): a HUD component and a
//! pause screen mounted with a store input, the pause menu a modal layer
//! over the dimmed HUD, and the same menu leaving under its CSS view
//! transition's iris cover. The frames match
//! `references/metal/mounted-layers-{1280,390}.png` and
//! `mounted-layers-iris-{1280,390}.png` (`OG_UI_CAPTURE=1` records them).
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
@view-transition { navigation: auto; types: iris; }
::view-transition-group(root) { animation-duration: 800ms; }
::view-transition { background-color: #0b1220; }
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
        let mut ui = ui_with_font(font);
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
                .join("references/metal")
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
