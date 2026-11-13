//! The engine's renderer: installed, warmed and described.

use super::*;

/// Attach (or detach, with `None`) a renderer to an engine; the probe's
/// `capture` draws through it.
pub fn set_render_backend(id: u32, backend: Option<Box<dyn RenderBackend>>) -> Result<(), String> {
    let eng = engine(id)?;
    rprobe::attach(&eng, backend);
    Ok(())
}

/// Make one pipeline of the engine's renderer that no frame has used yet
/// (og-render's `Renderer::warm`), when the host has idle time: the web
/// page behind its play gate. False when every one is made, or without a
/// renderer.
pub fn warm_renderer(id: u32) -> bool {
    engine(id).is_ok_and(|e| {
        e.render
            .try_borrow_mut()
            .is_ok_and(|mut rs| rs.backend.as_mut().is_some_and(|b| b.warm()))
    })
}

/// The game's RenderOptions (`GameDesc.render`), for a host creating a renderer.
pub fn render_options(id: u32) -> Value {
    engine(id).map_or(Value::Null, |e| e.render.borrow().options.clone())
}

/// `base` with the game's RenderOptions applied (GameDesc.render):
/// antialias, shadowMapSize, pointShadows, taaHistory, taaSharpen,
/// planarReflections, transparency, transparencyLayers and without; antialias left
/// out (or tier) is 4x MSAA in a 2D project (options.profile). Both hosts
/// create their renderer from it, so the options mean the same on each.
pub fn render_desc(id: u32, base: og_render::RenderDesc) -> og_render::RenderDesc {
    use crate::host::wire::{Antialias, PointShadows, RenderOptions, TaaHistory, Transparency};
    let given = render_options(id);
    // Options og.game refused (it refuses the engine) read as none given.
    let options: RenderOptions = crate::host::wire::value(&given, "render").unwrap_or_default();
    // A NaN reads as left out.
    let number = |v: f32, or: f32| if v.is_nan() { or } else { v };
    let mut desc = base;
    desc.hdr = options.hdr;
    desc.paper_white_nits = number(options.paper_white_nits, 0.0);
    desc.peak_nits = number(options.peak_nits, 0.0);
    let profile = engine(id).map_or(og_physics::PROFILE_3D, |e| e.state.borrow().profile);
    desc.antialias = match options.antialias {
        Antialias::Off => og_render::Antialias::Off,
        Antialias::Msaa4 => og_render::Antialias::Msaa4,
        Antialias::Taa => og_render::Antialias::Taa,
        Antialias::Fxaa => og_render::Antialias::Fxaa,
        Antialias::Smaa => og_render::Antialias::Smaa,
        Antialias::SmaaT2x => og_render::Antialias::SmaaT2x,
        // A 2D project's default is 4x MSAA: TAA's jitter and history
        // blend soften nearest-sampled pixel art.
        Antialias::Tier if profile == og_physics::PROFILE_2D => og_render::Antialias::Msaa4,
        Antialias::Tier => og_render::Antialias::Tier,
    };
    // Left out, the host's size stays (a phone's is smaller than the
    // option's default).
    if given["shadowMapSize"].is_u64() {
        desc.shadow_map_size = options.shadow_map_size.clamp(256, 4096);
    }
    desc.point_shadows = match options.point_shadows {
        PointShadows::Cube => og_render::PointShadows::Cube,
        PointShadows::Paraboloid => og_render::PointShadows::Paraboloid,
        PointShadows::Auto => og_render::PointShadows::Auto,
    };
    desc.taa_history = match options.taa_history {
        TaaHistory::CatmullRom => og_render::TaaHistory::CatmullRom,
        TaaHistory::Bilinear => og_render::TaaHistory::Bilinear,
    };
    desc.taa_sharpen = number(options.taa_sharpen, 0.0).clamp(0.0, 1.0);
    desc.planar_reflections = options
        .planar_reflections
        .clamp(1, og_render::MAX_PLANAR_REFLECTIONS);
    desc.transparency = match options.transparency {
        Transparency::LinkedList => og_render::Transparency::LinkedList,
        Transparency::Weighted => og_render::Transparency::Weighted,
    };
    desc.transparency_layers = options
        .transparency_layers
        .min(og_render::MAX_TRANSPARENCY_LAYERS);
    desc.without = options
        .without
        .iter()
        .filter_map(|f| og_render::LitFeatures::named(f.as_str()))
        .fold(og_render::LitFeatures::NONE, |a, f| a | f);
    // og.motion times its compute slices where the adapter can (its
    // `Gpu::set_frame_budget`): a publish build asks only for a game whose
    // bundle declares the model.
    desc.timestamps = engine(id).is_ok_and(|e| crate::bind::motion::present::declares(&e));
    desc
}

/// og-render's Antialias by its IDL name (RenderCapabilities.antialias).
pub fn antialias_name(a: og_render::Antialias) -> &'static str {
    match a {
        og_render::Antialias::Msaa4 => "msaa4",
        og_render::Antialias::Taa => "taa",
        og_render::Antialias::Fxaa => "fxaa",
        og_render::Antialias::Smaa => "smaa",
        og_render::Antialias::SmaaT2x => "smaaT2x",
        og_render::Antialias::Off | og_render::Antialias::Tier => "off",
    }
}

/// og-render's Transparency by its IDL name (RenderCapabilities.transparency).
pub fn transparency_name(t: og_render::Transparency) -> &'static str {
    match t {
        og_render::Transparency::Weighted => "weighted",
        og_render::Transparency::LinkedList => "linkedList",
    }
}

/// og-render's PointShadows by its IDL name (RenderCapabilities.pointShadows).
pub fn point_shadows_name(p: og_render::PointShadows) -> &'static str {
    match p {
        og_render::PointShadows::Cube => "cube",
        og_render::PointShadows::Paraboloid => "paraboloid",
        og_render::PointShadows::Auto => "auto",
    }
}
