//! RenderOptions.without reaches the renderer's description: left out or
//! empty it leaves nothing out (every lit shader carries every optional
//! feature, as before the option), and it leaves out exactly the
//! LitFeatures it names. RenderCapabilities.without reports what the
//! renderer made from that description was made without.
//!
//! Ways this could fail: render_desc stops reading it (a game that asked
//! for lean lit shaders compiles every feature anyway), reads only its
//! first entry, or maps a name to the wrong feature (a game would lose a
//! feature it uses, with only a log line to say so); the renderer forgets
//! what it was made without, or the capabilities do not say it (a game
//! could not tell why a feature it turned on does not draw).

use og_render::{LitFeatures, RenderDesc};
use og_runtime::{
    RenderBackend, og_engine_create, og_game_game, og_host_begin_bundle, og_render_capabilities,
    render_desc, set_render_backend,
};
use serde_json::{Value, json};

/// An engine for a game with these RenderOptions.
fn engine(render: Value) -> u32 {
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({ "seed": 1, "render": render }).to_string()));
    let v: Value = serde_json::from_str(&og_engine_create(
        game,
        json!({ "profile": "3d" }).to_string(),
    ))
    .unwrap();
    v["value"].as_u64().unwrap_or_else(|| panic!("{v}")) as u32
}

fn resolved(render: Value) -> LitFeatures {
    render_desc(engine(render), RenderDesc::default()).without
}

/// A backend around a renderer made from the engine's own description, as
/// both hosts make theirs.
struct Made(og_render::Renderer);

impl RenderBackend for Made {
    fn tier(&self) -> String {
        self.0.info().backend_tier()
    }
    fn size(&self) -> (u32, u32) {
        self.0.size()
    }
    fn info(&self) -> Option<og_render::RenderInfo> {
        Some(self.0.info().clone())
    }
    fn render(&mut self, _: &og_render::RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(
        &mut self,
        _: &og_render::RenderScene,
        _: u32,
        _: u32,
    ) -> Option<og_render::RgbaImage> {
        None
    }
}

/// RenderCapabilities.without: empty without a renderer, then the features
/// the renderer was made without, in LitFeature's order.
#[test]
#[ignore = "requires GPU"]
fn the_capabilities_report_what_the_renderer_was_made_without() {
    let without = |e: u32| {
        let caps: Value = serde_json::from_str(&og_render_capabilities(e)).unwrap();
        caps["without"].clone()
    };
    let e = engine(json!({ "without": ["decals", "areaLights"] }));
    assert_eq!(without(e), json!([]), "without a renderer");
    let desc = render_desc(
        e,
        RenderDesc {
            width: 64,
            height: 64,
            ..RenderDesc::default()
        },
    );
    let renderer = og_render::Renderer::new_blocking(&desc).expect("a headless adapter");
    set_render_backend(e, Some(Box::new(Made(renderer)))).unwrap();
    assert_eq!(without(e), json!(["areaLights", "decals"]));
    // Each renderer goes before the test ends, its compiles finished, not
    // with the process while a driver still compiles.
    og_render::settle_pipelines();
    set_render_backend(e, None).unwrap();
    let e = engine(json!({}));
    let desc = render_desc(
        e,
        RenderDesc {
            width: 64,
            height: 64,
            ..RenderDesc::default()
        },
    );
    let renderer = og_render::Renderer::new_blocking(&desc).expect("a headless adapter");
    set_render_backend(e, Some(Box::new(Made(renderer)))).unwrap();
    assert_eq!(without(e), json!([]), "nothing left out by default");
    og_render::settle_pipelines();
    set_render_backend(e, None).unwrap();
}

#[test]
fn without_leaves_out_only_the_features_it_names() {
    assert_eq!(resolved(json!({})), LitFeatures::NONE);
    assert_eq!(resolved(json!({ "without": [] })), LitFeatures::NONE);
    assert_eq!(
        resolved(json!({ "without": ["decals"] })),
        LitFeatures::DECALS
    );
    assert_eq!(
        resolved(json!({ "without": ["gi", "areaLights"] })),
        LitFeatures::GI | LitFeatures::AREA_LIGHTS
    );
    assert_eq!(
        resolved(json!({ "without": ["surfaceWeather", "gi", "areaLights", "decals"] })),
        LitFeatures::SURFACE_WEATHER
            | LitFeatures::GI
            | LitFeatures::AREA_LIGHTS
            | LitFeatures::DECALS
    );
}
