//! Asset loading and look binding calls.

use super::*;

pub fn og_load_texture(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_texture(&eng, &path).map(asset_value)))
}

// ── load ────────────────────────────────────────────────────────────────────

fn asset_value(i: usize) -> Value {
    json!(assets::handle(i) as f64)
}

pub fn og_load_bundle(world: u32, path: String) -> String {
    result((|| {
        let eng = engine(world)?;
        let i = eng.assets.borrow_mut().open_bundle(&path);
        Ok(asset_value(i))
    })())
}

pub fn og_load_bundle_info(world: u32, asset: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let e = index_of(asset)
            .and_then(|i| a.entries.get(i))
            .ok_or_else(|| format!("asset {asset} is not alive"))?;
        match (&e.status, &e.payload) {
            (Status::Failed(r), _) => Err(r.clone()),
            (Status::Pending, _) => Err(format!("{} is still loading", e.path)),
            (_, Payload::Bundle { bundle, dir }) => {
                let m = bundle.manifest();
                Ok(json!({ "path": dir, "assets": m.assets.len(), "outputs": m.outputs().count() }))
            }
            _ => Err(format!("{} is not a bundle", e.path)),
        }
    })())
}

pub fn og_load_default_bundle(world: u32) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let i = a
            .default_bundle
            .ok_or("no cooked bundle: the host gave the engine no project root")?;
        match &a.entries[i].status {
            Status::Failed(r) => Err(r.clone()),
            _ => Ok(asset_value(i)),
        }
    })())
}

pub fn og_load_gltf(world: u32, path: String, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let shadows = assets::ModelShadows::from_desc(&parse(desc.as_deref())?)?;
        let index = assets::load_gltf(&eng, &path)?;
        // The flags belong to the asset: this load's, for the models
        // instantiated after it.
        eng.assets.borrow_mut().model_shadows.insert(index, shadows);
        Ok(asset_value(index))
    })())
}

pub fn og_load_scene(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_scene(&eng, &path).map(asset_value)))
}

pub fn og_load_lut(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::fx::load_lut(&eng, &path).map(asset_value)))
}

pub fn og_load_look(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::look::load(&eng, &path).map(asset_value)))
}

pub fn og_load_look_info(world: u32, look: f64) -> String {
    result(engine(world).and_then(|eng| super::look::info(&eng, super::look::index(look)?)))
}

pub fn og_load_palette(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::look::load_palette(&eng, &path).map(asset_value)))
}

pub fn og_render_apply_look(world: u32, look: f64) -> String {
    result_void(engine(world).and_then(|eng| super::look::apply(&eng, super::look::index(look)?)))
}

/// `og.render.applyQuality`: the preset's row of RenderSettings' quality
/// fields (og-render's [`og_render::RenderQuality::preset`], the table in
/// api/render.ogt's doc), written as the game would write them.
pub fn og_render_apply_quality(world: u32, preset: String) -> String {
    let row = match preset.as_str() {
        "low" => og_render::QualityPreset::Low,
        "medium" => og_render::QualityPreset::Medium,
        "high" => og_render::QualityPreset::High,
        other => {
            return refuse(format!(
                "'{other}' is not a graphics preset (low, medium or high)"
            ));
        }
    };
    let q = og_render::RenderQuality::preset(row);
    let fields = json!({
        "quality": preset,
        "rectShadows": q.rect_shadows,
        "shadowedLights": q.shadowed_lights,
        "shadowMapScale": q.shadow_map_scale,
        "volumetricFog": q.volumetric_fog,
        "softParticles": q.soft_particles,
        "screenSpaceReflections": q.ssr,
        "ambientOcclusion": q.ambient_occlusion,
        "aoResolution": crate::extract::ao_resolution_setting(q.ao_resolution),
        "ssrStepScale": q.ssr_step_scale,
        "volumetricStepScale": q.volumetric_step_scale,
        "msaaSamples": q.msaa_samples,
    });
    result_void(
        engine(world).and_then(|eng| {
            super::engine::write_resource_value(eng.id, "og.RenderSettings", &fields)
        }),
    )
}

pub fn og_render_look(world: u32) -> String {
    let info = engine(world).ok().and_then(|eng| {
        let applied = eng.render.borrow().fx.look.applied;
        applied.and_then(|i| super::look::info(&eng, i).ok())
    });
    info.unwrap_or(Value::Null).to_string()
}

pub fn og_load_texture_array(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_texture_array(&eng, &path)))
}

pub fn og_load_environment(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::environment::load(&eng, &path).map(asset_value)))
}

pub fn og_load_emitter(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| super::fx::load_emitter(&eng, &path).map(asset_value)))
}

pub fn og_load_font(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| assets::load_font(&eng, &path).map(asset_value)))
}

pub fn og_load_font_info(world: u32, font: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        index_of(font)
            .and_then(|i| a.fonts.get(i))
            .map(|f| f.info.clone())
            .ok_or_else(|| format!("font {font} is not alive"))
    })())
}

pub fn og_load_sound(world: u32, path: String) -> String {
    result(engine(world).and_then(|eng| {
        let i = assets::load_sound(&eng, &path)?;
        let handle = eng.assets.borrow().sound_handles[i];
        Ok(json!(handle as f64))
    }))
}

pub fn og_load_sound_info(world: u32, sound: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let i = assets::sound_index(&eng, sound);
        let a = eng.assets.borrow();
        let s = i
            .and_then(|i| a.sounds.get(i))
            .ok_or_else(|| format!("sound {sound} is not alive"))?;
        let m = &s.meta;
        Ok(json!({
            "rate": m.sample_rate, "channels": m.channels, "frames": m.frames, "chunks": m.chunk_count,
            "seconds": f64::from(m.frames) / f64::from(m.sample_rate.max(1)),
        }))
    })())
}

pub fn og_load_status(world: u32, asset: f64) -> String {
    let status = engine(world).ok().and_then(|eng| {
        let a = eng.assets.borrow();
        index_of(asset)
            .and_then(|i| a.entries.get(i))
            .map(|e| e.status.clone())
    });
    match status {
        Some(Status::Ready) => "ready",
        Some(Status::Pending) => "pending",
        _ => "failed",
    }
    .into()
}

pub fn og_load_error(world: u32, asset: f64) -> String {
    let reason = engine(world).ok().and_then(|eng| {
        let a = eng.assets.borrow();
        match index_of(asset).and_then(|i| a.entries.get(i)) {
            Some(e) => match &e.status {
                Status::Failed(r) => Some(r.clone()),
                _ => None,
            },
            None => Some(format!("{asset} is not an asset handle")),
        }
    });
    json!(reason).to_string()
}

/// A model's clip names from its `#anim` blob, in glTF order; none when the
/// cook wrote no `#anim` (no skins, clips or morph targets).
fn clip_names(a: &assets::Assets, model: &str) -> Result<Vec<String>, String> {
    let id = og_bundle::runtime::anim_id(model);
    if a.bundle_for(&id).is_err() {
        return Ok(Vec::new());
    }
    let (bytes, bundle) = a.blob(&id)?;
    bundle
        .clip_names(&id, bytes.bytes())
        .map_err(|e| e.to_string())
}

pub fn og_load_gltf_info(world: u32, asset: f64) -> String {
    result((|| {
        let eng = engine(world)?;
        let a = eng.assets.borrow();
        let e = index_of(asset)
            .and_then(|i| a.entries.get(i))
            .ok_or_else(|| format!("asset {asset} is not alive"))?;
        match &e.payload {
            Payload::Gltf {
                scenes,
                geometries,
                materials,
                variants,
                lights,
            } => Ok(json!({
                "variants": variants,
                "lights": lights.len(),
                "scenes": scenes,
                "geometries": geometries.iter().map(|g| *g as f64).collect::<Vec<_>>(),
                "materials": materials.iter().map(|m| *m as f64).collect::<Vec<_>>(),
                "animations": clip_names(&a, &e.path)?,
            })),
            _ => Err(format!("{} is not a glTF", e.path)),
        }
    })())
}
