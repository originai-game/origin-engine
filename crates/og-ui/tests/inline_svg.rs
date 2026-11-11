//! Inline SVG in documents (docs/ui.md §3.3): a cooldown ring whose
//! `stroke-dashoffset` binds a store over `pathLength="1"`, a bar chart
//! from a keyed `{#each}` with a `url(#…)` gradient, a line chart's path
//! data bound to a store, a star `<polygon>` and an even-odd ring, each in
//! a box CSS sizes. The frames match `references/vulkan/inline-svg-{1280,390}.png`
//! (`OG_UI_CAPTURE=1` records them).
//!
//! Ways it could fail: the view box not fitted (shapes at their user-space
//! size), strokes not scaled with it, the dash offset read as px instead of
//! of `pathLength`, a gradient placed in the wrong space, or `fill-rule`
//! lost (the ring's hole filled).

use crate::support;

use og_ui::*;
use serde_json::json;
use std::collections::BTreeMap;
use support::*;

const CARD: &str = r##"<meta name="component" content="card">
<column id="Center">
<panel id="Card">
  <h2 id="Title">Inline SVG</h2>
  <row id="Row">
    <svg id="Ring" viewBox="0 0 40 40">
      <circle id="Track" cx="20" cy="20" r="17" fill="none" stroke="var(--line)" stroke-width="4"/>
      <circle id="Ready" cx="20" cy="20" r="17" fill="none" stroke="var(--accent)" stroke-width="4"
        stroke-linecap="round" pathLength="1" stroke-dasharray="1 1" stroke-dashoffset={1 - $ready}
        transform="rotate(-90 20 20)"/>
    </svg>
    <svg id="Bars" viewBox="0 0 100 40">
      <linearGradient id="Up" x1="0" y1="1" x2="0" y2="0">
        <stop offset="0" stop-color="#1f6feb"/><stop offset="1" stop-color="#7ee787"/>
      </linearGradient>
      {#each $bars as b (b.id)}<rect id="Bar" x={b.x} y={40 - b.h} width="10" height={b.h} rx="2" fill="url(#Up)"/>{/each}
    </svg>
  </row>
  <row id="Row2">
    <svg id="Line" viewBox="0 0 100 40">
      <path id="Area" d={$area} fill="var(--accent)" fill-opacity="0.25"/>
      <path id="Stroke" d={$line} fill="none" stroke="var(--accent)" stroke-width="2" stroke-linejoin="round"/>
    </svg>
    <svg id="Badge" viewBox="0 0 40 40">
      <path id="Hole" fill-rule="evenodd" fill="#f0b429"
        d="M20 2a18 18 0 1 1 0 36a18 18 0 1 1 0-36zM20 10a10 10 0 1 0 0 20a10 10 0 1 0 0-20z"/>
      <polygon id="Star" points="20,12 22.4,17.2 28,17.6 23.6,21.2 25,26.6 20,23.6 15,26.6 16.4,21.2 12,17.6 17.6,17.2" fill="#fff"/>
    </svg>
  </row>
</panel>
</column>
<style>
#Center { width: 100%; height: 100%; align-items: center; justify-content: center; }
#Card { padding: 24px; gap: 16px; }
#Title { margin: 0; }
#Row, #Row2 { gap: 16px; align-items: center; }
#Ring, #Badge { width: 96px; height: 96px; }
#Bars, #Line { width: 220px; height: 96px; }
@media (max-width: 500px) {
  #Ring, #Badge { width: 72px; height: 72px; }
  #Bars, #Line { width: 180px; height: 72px; }
}
</style>"##;

fn send(ui: &mut Ui, command: serde_json::Value) {
    let reply = reactive::wire::decode(
        &ui.apply_reactive(&reactive::wire::encode(&json!([command])).unwrap()),
    )
    .unwrap();
    assert_eq!(reply[0]["ok"], true, "{reply}");
}

#[test]
#[ignore = "requires GPU"]
fn inline_svg_matches_its_reference_frames() {
    use og_render::*;
    for (width, height) in [(1280u32, 720u32), (390, 844)] {
        let mut scene = RenderScene::new();
        let font = add_test_font(&mut scene);
        let mut ui = ui_with_font(font);
        ui.define_document(UiDocument::parse("ui/card.svelte", CARD).unwrap());
        let src = |n: &str| reactive::SourceId::new("test", n);
        send(
            &mut ui,
            json!({"op": "signal", "source": src("ready"), "initial": 0.7}),
        );
        let ys = [12.0, 30.0, 18.0, 26.0, 8.0, 22.0];
        let pts: Vec<String> = ys
            .iter()
            .enumerate()
            .map(|(i, y)| format!("{} {y}", i as f32 * 20.0))
            .collect();
        let line = format!("M{}", pts.join(" L"));
        send(
            &mut ui,
            json!({"op": "signal", "source": src("line"), "initial": line}),
        );
        send(
            &mut ui,
            json!({"op": "signal", "source": src("area"), "initial": format!("{line} L100 40 L0 40 Z")}),
        );
        let bars: Vec<_> = [14.0, 26.0, 20.0, 34.0, 10.0, 30.0, 22.0]
            .iter()
            .enumerate()
            .map(|(i, h)| json!({"id": format!("b{i}"), "x": 2.0 + i as f32 * 14.0, "h": h}))
            .collect();
        send(
            &mut ui,
            json!({"op": "collection", "source": src("bars"), "key": "id", "items": bars}),
        );
        ui.mount_document(
            src("card"),
            "card",
            ["ready", "line", "area", "bars"]
                .iter()
                .map(|n| (n.to_string(), src(n)))
                .collect::<BTreeMap<_, _>>(),
            Default::default(),
            &bind::NoEnv,
        )
        .unwrap();
        let vp = Viewport::new(width as f32, height as f32);
        for _ in 0..4 {
            ui.begin(InputFrame::default(), vp);
            ui.build_retained_documents(&bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        let (pixels, info) = capture(&ui, &mut scene, width, height);
        let name = format!("inline-svg-{width}.png");
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
