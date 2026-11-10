//! Material, geometry and immediate rendering binding calls.

use super::*;

// ── render ──────────────────────────────────────────────────────────────────

/// A StandardMaterialDesc applied over `base` (fields the desc omits keep
/// their values): og-render's StandardMaterial carries every field under
/// the same name; texture fields are Texture handles and texture-array
/// fields TextureArray handles (null clears them).
pub(crate) fn material(
    base: og_render::StandardMaterial,
    desc: &Value,
    textures: &[og_render::TextureHandle],
    arrays: &[super::assets::LoadedArray],
) -> Result<og_render::StandardMaterial, String> {
    let mut m = base;
    let num = |k: &str| desc[k].as_f64().map(|v| v as f32);
    let flag = |k: &str| desc[k].as_bool().map(u32::from);
    let rgb = |k: &str| {
        let c = &desc[k];
        c.is_object().then(|| {
            glam::Vec3::new(
                c["r"].as_f64().unwrap_or(0.0) as f32,
                c["g"].as_f64().unwrap_or(0.0) as f32,
                c["b"].as_f64().unwrap_or(0.0) as f32,
            )
        })
    };
    let tex = |k: &str| -> Result<Option<u32>, String> {
        match desc.get(k) {
            None => Ok(None),
            Some(Value::Null) => Ok(Some(0)),
            Some(v) => {
                let t = v.as_f64().unwrap_or(-1.0);
                index_of(t)
                    .and_then(|i| textures.get(i))
                    .map(|h| Some(h.slot()))
                    .ok_or_else(|| format!("{k}: texture {t} is not alive"))
            }
        }
    };
    macro_rules! set {
        ($field:ident, $v:expr) => {
            if let Some(v) = $v {
                m.$field = v;
            }
        };
    }
    set!(color, rgb("color"));
    set!(map, tex("map")?);
    set!(metalness, num("metalness"));
    set!(metalnessMap, tex("metalnessMap")?);
    set!(roughness, num("roughness"));
    set!(roughnessMap, tex("roughnessMap")?);
    set!(normalMap, tex("normalMap")?);
    set!(aoMap, tex("aoMap")?);
    set!(emissive, rgb("emissive"));
    set!(emissiveIntensity, num("emissiveIntensity"));
    set!(emissiveMap, tex("emissiveMap")?);
    set!(opacity, num("opacity"));
    set!(transparent, flag("transparent"));
    set!(alphaTest, num("alphaTest"));
    set!(
        side,
        desc["side"].as_str().map(|s| match s {
            "back" => 1,
            "double" => 2,
            _ => 0,
        })
    );
    set!(flatShading, flag("flatShading"));
    set!(wireframe, flag("wireframe"));
    set!(alphaToCoverage, flag("alphaToCoverage"));
    set!(paintDetail, num("paintDetail"));
    set!(weathering, num("weathering"));
    set!(wireframeAngle, num("wireframeAngle"));
    set!(wireframeColor, rgb("wireframeColor"));
    if let Some(v) = desc["foliage"]["translucency"].as_f64() {
        m.foliage.x = v as f32;
    }
    if let Some(v) = desc["foliage"]["domeNormals"].as_f64() {
        m.foliage.y = v as f32;
    }
    set!(envMapIntensity, num("envMapIntensity"));
    let vec2 = |k: &str, fallback: glam::Vec2| {
        let v = &desc[k];
        v.is_object().then(|| {
            glam::Vec2::new(
                v["x"].as_f64().map_or(fallback.x, |x| x as f32),
                v["y"].as_f64().map_or(fallback.y, |y| y as f32),
            )
        })
    };
    set!(normalScale, vec2("normalScale", glam::Vec2::ONE));
    set!(aoMapIntensity, num("aoMapIntensity"));
    set!(unlit, flag("unlit"));
    set!(styled, flag("styled"));
    set!(toon, flag("toon"));
    set!(gradientMap, tex("gradientMap")?);
    set!(rimColor, rgb("rimColor"));
    set!(rimIntensity, num("rimIntensity"));
    set!(rimPower, num("rimPower"));
    set!(flashColor, rgb("flashColor"));
    set!(flash, num("flash"));
    set!(dissolve, num("dissolve"));
    set!(dissolveMap, tex("dissolveMap")?);
    set!(dissolveEdgeColor, rgb("dissolveEdgeColor"));
    set!(dissolveEdgeWidth, num("dissolveEdgeWidth"));
    set!(dissolveScale, num("dissolveScale"));
    set!(outlineColor, rgb("outlineColor"));
    set!(outlineWidth, num("outlineWidth"));
    set!(clearcoat, num("clearcoat"));
    set!(clearcoatRoughness, num("clearcoatRoughness"));
    set!(clearcoatMap, tex("clearcoatMap")?);
    set!(clearcoatRoughnessMap, tex("clearcoatRoughnessMap")?);
    set!(clearcoatNormalMap, tex("clearcoatNormalMap")?);
    set!(
        clearcoatNormalScale,
        vec2("clearcoatNormalScale", glam::Vec2::ONE)
    );
    set!(sheen, num("sheen"));
    set!(sheenColor, rgb("sheenColor"));
    set!(sheenRoughness, num("sheenRoughness"));
    set!(sheenColorMap, tex("sheenColorMap")?);
    set!(sheenRoughnessMap, tex("sheenRoughnessMap")?);
    set!(transmission, num("transmission"));
    set!(transmissionMap, tex("transmissionMap")?);
    set!(thickness, num("thickness"));
    set!(thicknessMap, tex("thicknessMap")?);
    set!(attenuationDistance, num("attenuationDistance"));
    set!(attenuationColor, rgb("attenuationColor"));
    set!(ior, num("ior"));
    set!(iridescence, num("iridescence"));
    set!(iridescenceIOR, num("iridescenceIOR"));
    set!(
        iridescenceThicknessRange,
        vec2("iridescenceThicknessRange", glam::Vec2::new(100.0, 400.0))
    );
    set!(iridescenceMap, tex("iridescenceMap")?);
    set!(iridescenceThicknessMap, tex("iridescenceThicknessMap")?);
    set!(anisotropy, num("anisotropy"));
    set!(anisotropyRotation, num("anisotropyRotation"));
    set!(anisotropyMap, tex("anisotropyMap")?);
    // Each slot's uv transform (KHR_texture_transform, three.js Texture.offset/repeat/rotation).
    set!(mapOffset, vec2("mapOffset", glam::Vec2::ZERO));
    set!(mapRepeat, vec2("mapRepeat", glam::Vec2::ONE));
    set!(mapRotation, num("mapRotation"));
    set!(
        metalnessMapOffset,
        vec2("metalnessMapOffset", glam::Vec2::ZERO)
    );
    set!(
        metalnessMapRepeat,
        vec2("metalnessMapRepeat", glam::Vec2::ONE)
    );
    set!(metalnessMapRotation, num("metalnessMapRotation"));
    set!(normalMapOffset, vec2("normalMapOffset", glam::Vec2::ZERO));
    set!(normalMapRepeat, vec2("normalMapRepeat", glam::Vec2::ONE));
    set!(normalMapRotation, num("normalMapRotation"));
    set!(
        emissiveMapOffset,
        vec2("emissiveMapOffset", glam::Vec2::ZERO)
    );
    set!(
        emissiveMapRepeat,
        vec2("emissiveMapRepeat", glam::Vec2::ONE)
    );
    set!(emissiveMapRotation, num("emissiveMapRotation"));
    set!(aoMapOffset, vec2("aoMapOffset", glam::Vec2::ZERO));
    set!(aoMapRepeat, vec2("aoMapRepeat", glam::Vec2::ONE));
    set!(aoMapRotation, num("aoMapRotation"));
    let array = |k: &str| -> Result<Option<u32>, String> {
        match desc.get(k) {
            None => Ok(None),
            Some(Value::Null) => Ok(Some(0)),
            Some(v) => {
                let a = v.as_f64().unwrap_or(-1.0);
                let loaded = index_of(a)
                    .and_then(|i| arrays.get(i))
                    .ok_or_else(|| format!("{k}: texture array {a} is not loaded in this world"))?;
                let h = loaded.handle.ok_or_else(|| {
                    format!(
                        "{k}: {}'s streaming group is not resident (the array loads with it)",
                        loaded.id
                    )
                })?;
                Ok(Some(h.slot()))
            }
        }
    };
    let layer = |k: &str| desc[k].as_f64().map(|v| v.max(0.0) as u32);
    set!(mapArray, array("mapArray")?);
    set!(mapLayer, layer("mapLayer"));
    set!(normalMapArray, array("normalMapArray")?);
    set!(normalMapLayer, layer("normalMapLayer"));
    Ok(m)
}

/// Material handles are the render state's material index with generation 1.
pub(crate) fn material_handle(index: usize) -> f64 {
    assets::handle(index) as f64
}

pub fn og_render_standard_material(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        // three.js MeshStandardMaterial defaults: white, metalness 0, roughness 1.
        let mut rs = eng.render.borrow_mut();
        let m = material(
            og_render::StandardMaterial::default(),
            &desc,
            &rs.textures,
            &rs.texture_arrays,
        )?;
        let handle = rs.scene.add_standard_material(m);
        rs.materials.push(handle);
        Ok(json!(material_handle(rs.materials.len() - 1)))
    })())
}

pub fn og_render_update_material(world: u32, handle: f64, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let mut rs = eng.render.borrow_mut();
        let h = index_of(handle)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {handle} is not alive"))?;
        let current = *rs
            .scene
            .standard_material(h)
            .ok_or("material is not alive")?;
        let m = material(current, &desc, &rs.textures, &rs.texture_arrays)?;
        rs.scene.set_standard_material(h, m);
        Ok(())
    })())
}

pub fn og_render_capabilities(world: u32) -> String {
    let caps = engine(world).ok().and_then(|eng| {
        let rs = eng.render.borrow();
        rs.backend.as_ref().map(|b| {
            let tier = b.tier();
            let (w, h) = b.size();
            let info = b.info();
            json!({
                "tier": tier,
                "compute": true,
                "antialias": b.antialias(),
                "hdrSupported": info.as_ref().is_some_and(|i| i.hdr_supported),
                "hdr": info.as_ref().is_some_and(|i| i.hdr),
                "maxLights": og_render::MAX_LIGHTS,
                "maxShadowedLights": info.as_ref().map_or(0, |i| i.shadowed_lights),
                "maxMsaaSamples": info.as_ref().map_or(0, |i| i.msaa_samples),
                "pointShadows": info.as_ref().map_or("cube", |i| crate::point_shadows_name(i.point_shadows)),
                "width": w, "height": h,
                "stylise": og_render::STYLISE_DRAWN,
                "ssr": true,
                "reflectionProbes": og_render::MAX_REFLECTION_PROBES,
                "volumetricFog": match info.as_ref().map(|i| i.volumetric) {
                    Some(og_render::VolumetricMethod::Froxel) => "froxel",
                    Some(og_render::VolumetricMethod::Raymarch) => "raymarch",
                    None => "froxel",
                },
                "volumetricLights": info.as_ref().map_or(
                    og_render::MAX_CLUSTER_LIGHTS as u32,
                    |i| i.volumetric_lights,
                ),
                "maxLightCookies": og_render::MAX_COOKIE_LAYERS,
                "gpuTimestamps": b.gpu_timing_absent().is_none(),
                "planarReflections": info.as_ref().map_or(0, |i| i.planar_reflections),
                "without": info.as_ref().map_or(Vec::new(), |i| i.without.names()),
                "upscalers": crate::extract::UPSCALERS,
                "transparency": info.as_ref().map_or("weighted", |i| crate::transparency_name(i.transparency)),
                "gpuCrowds": info.as_ref().is_some_and(|i| i.gpu_crowds),
                "visualSimulation": info.as_ref().is_some_and(|i| i.visual_simulation),
                "world3d": og_render::WORLD_3D,
                "exrDecode": crate::uncooked::EXR_DECODE,
                "visibilityBuffer": info.is_some() && b.visibility_buffer_refused().is_none(),
                "visibilityBufferRefused": b.visibility_buffer_refused(),
            })
        })
        .map(|mut c| {
            if let (Value::Object(c), Value::Object(v)) = (&mut c, super::views::capabilities(&rs)) {
                c.extend(v);
            }
            c
        })
    });
    caps.unwrap_or_else(|| json!({ "tier": "none", "compute": false, "antialias": "off", "hdrSupported": false, "hdr": false, "maxLights": 0, "maxShadowedLights": 0, "maxMsaaSamples": 0, "pointShadows": "cube", "width": 0, "height": 0, "stylise": [], "ssr": false, "reflectionProbes": 0, "renderTargetDepth": false, "maxRenderTargetSize": 0, "maxCustomPosts": 0, "volumetricFog": "raymarch", "volumetricLights": 0, "maxLightCookies": 0, "gpuTimestamps": false, "planarReflections": 0, "without": [], "visualSimulation": false, "upscalers": [], "transparency": "weighted", "gpuCrowds": false, "world3d": og_render::WORLD_3D, "exrDecode": crate::uncooked::EXR_DECODE, "visibilityBuffer": false, "visibilityBufferRefused": null }))
        .to_string()
}

/// `og.render.snap`: `entity` drawn at its tick until the next tick
/// (`super::interpolate`).
pub fn og_render_snap(world: u32, entity: f64) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let (e, tick) = match running(world) {
            Some(f) => {
                // SAFETY: the frame is published while its system runs.
                let ctx = unsafe { f.ctx() };
                let e = live(&eng, "render.snap", entity, |e| Ok(ctx.is_alive(e)))?;
                (e, ctx.time().tick)
            }
            None => {
                let w = eng.world()?;
                let e = live(&eng, "render.snap", entity, |e| Ok(w.is_alive(e)))?;
                (e, w.time().tick)
            }
        };
        eng.render.borrow_mut().interp.snap(e.to_bits(), tick);
        Ok(())
    })())
}

/// `og.render.snapCamera`.
pub fn og_render_snap_camera(world: u32) {
    if let Ok(eng) = engine(world) {
        let tick = match running(world) {
            // SAFETY: the frame is published while its system runs.
            Some(f) => unsafe { f.ctx() }.time().tick,
            None => eng.world().map_or(0, |w| w.time().tick),
        };
        eng.render.borrow_mut().interp.snap_camera(tick);
    }
}

pub fn og_render_text(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let Some(font) = d["font"].as_f64() else {
            return Ok(());
        };
        let fi = index_of(font).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let tick = og_world_get_tick(world) as u64;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let c = &d["color"];
        let mut rs = eng.render.borrow_mut();
        new_frame(&mut rs, tick);
        let text = d["text"].as_str().unwrap_or_default().to_string();
        // Its missing glyphs warned of now, on the CPU, renderer or not.
        if let Some(missing) = crate::glyphs::lacking(&mut rs, render, &text) {
            drop(rs);
            crate::glyphs::warn_missing(&eng, tick, vec![missing]);
            rs = eng.render.borrow_mut();
        }
        let size = num(&d, "size", 1.0);
        let rotation = num(&d, "rotation", 0.0);
        let position = vec3(&d["position"]);
        let color = [
            num(c, "r", 1.0),
            num(c, "g", 1.0),
            num(c, "b", 1.0),
            num(&d, "opacity", 1.0),
        ];
        let layer = d["layer"].as_i64().unwrap_or(0) as i32;
        let screen = d["screen"].as_bool().unwrap_or(false);
        let clip = clip_of(&d["clip"]);
        let outline = num(&d, "outline", 0.0).clamp(0.0, 1.0);
        let oc = &d["outlineColor"];
        let outline_color = [
            num(oc, "r", 0.0),
            num(oc, "g", 0.0),
            num(oc, "b", 0.0),
            color[3],
        ];
        let facing = super::feel::facing_named(d["facing"].as_str());
        let shift = match d["align"].as_str() {
            Some("center") => 0.5,
            Some("end") => 1.0,
            _ => 0.0,
        };
        // Unaligned, or a faced world text, which the renderer aligns about
        // its point in its own plane (`og_render::TextFacing::align`).
        let faced = facing != og_render::SpriteFacing::None && !screen;
        if shift == 0.0 || faced {
            rs.texts.push(super::extract::QueuedText {
                font: render,
                text,
                position,
                size,
                rotation,
                color,
                layer,
                screen,
                clip,
                outline,
                outline_color,
                facing,
                align: shift,
            });
            return Ok(());
        }
        // Aligned: each line is placed on its own, moved left along the
        // text's baseline by its advance times `shift`.
        let (metrics, line_height) = font_metrics(&rs, render)?;
        let (sin, cos) = og_core::math::sin_cos(rotation);
        let along = glam::Vec3::new(cos, sin, 0.0);
        // The next line is below: -y in the world, +y on screen (y down,
        // rotation clockwise).
        let down = if screen {
            glam::Vec3::new(-sin, cos, 0.0)
        } else {
            glam::Vec3::new(sin, -cos, 0.0)
        };
        for (i, line) in text.split('\n').enumerate() {
            let w = metrics.measure(line, size);
            rs.texts.push(super::extract::QueuedText {
                font: render,
                text: line.to_string(),
                position: position - along * (w * shift) + down * (i as f32 * line_height * size),
                size,
                rotation,
                color,
                layer,
                screen,
                clip,
                outline,
                outline_color,
                facing,
                align: 0.0,
            });
        }
        Ok(())
    })())
}

/// Adds a generated geometry; `physics` also registers it as a physics mesh
/// under its handle.
pub(crate) fn add_geometry(
    world: u32,
    mesh: og_render::MeshData,
    physics: bool,
) -> Result<Value, String> {
    let eng = engine(world)?;
    let handle = {
        let mut rs = eng.render.borrow_mut();
        let h = rs.scene.add_mesh(mesh.clone());
        rs.geometries.push(h);
        assets::handle(rs.geometries.len() - 1)
    };
    if physics {
        super::modules::set_mesh(&eng, handle, super::modules::mesh_of(&mesh))?;
    }
    Ok(json!(handle as f64))
}

/// A geometry desc's `uvRepeat` applied (og_render::shapes::repeat_uv).
pub(crate) fn uv_repeat(d: &Value, mesh: &mut og_render::MeshData) -> Result<(), String> {
    let r = &d["uvRepeat"];
    let axis = |k: &str| -> Result<f32, String> {
        let v = r[k].as_f64().unwrap_or(1.0);
        if !v.is_finite() {
            return Err(format!("uvRepeat.{k} must be finite, got {v}"));
        }
        Ok(v as f32)
    };
    og_render::shapes::repeat_uv(mesh, glam::Vec2::new(axis("x")?, axis("y")?));
    Ok(())
}

fn size_of(d: &Value, key: &str, default: f64) -> Result<f32, String> {
    let v = d[key].as_f64().unwrap_or(default);
    if !(v.is_finite() && v >= 0.0) {
        return Err(format!("{key} must be finite and at least 0, got {v}"));
    }
    Ok(v as f32)
}

pub fn og_render_box_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let size = glam::Vec3::new(
            size_of(&d, "width", 1.0)?,
            size_of(&d, "height", 1.0)?,
            size_of(&d, "depth", 1.0)?,
        );
        let mut mesh = og_render::shapes::cuboid(size);
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_plane_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let mut mesh =
            og_render::shapes::plane_xy(size_of(&d, "width", 1.0)?, size_of(&d, "height", 1.0)?);
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_sphere_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d = parse(desc.as_deref())?;
        let radius = size_of(&d, "radius", 1.0)?;
        let seg = d["widthSegments"].as_u64().unwrap_or(32).clamp(3, 512) as u32;
        let rings = d["heightSegments"].as_u64().unwrap_or(16).clamp(2, 512) as u32;
        let mut mesh = og_render::shapes::sphere(seg, rings);
        for v in &mut mesh.vertices {
            for c in &mut v.position {
                *c *= radius * 2.0;
            }
        }
        uv_repeat(&d, &mut mesh)?;
        add_geometry(world, mesh, false)
    })())
}

/// Starts the queued draws of a new tick.
fn new_frame(rs: &mut super::extract::RenderState, tick: u64) {
    if rs.texts_tick != tick {
        rs.texts.clear();
        rs.fx.sprites.clear();
        rs.fx.ribbons.clear();
        rs.texts_tick = tick;
    }
}

/// A `clip` field: `[x, y, width, height]`, or none.
fn clip_of(v: &Value) -> Option<[f32; 4]> {
    let a = v.as_array().filter(|a| a.len() == 4)?;
    let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
    Some([n(0), n(1), n(2), n(3)])
}

/// A loaded font's layout metrics (its first page carries every advance)
/// and its line height in em.
fn font_metrics(
    rs: &super::extract::RenderState,
    render: usize,
) -> Result<(og_ui::FontMetrics, f32), String> {
    let first = rs
        .fonts
        .get(render)
        .and_then(|p| p.pages.first().copied())
        .ok_or("the font has no pages")?;
    let msdf = rs
        .scene
        .font(first)
        .ok_or("the font's first page is missing")?;
    Ok((og_ui::FontMetrics::from_msdf(msdf), msdf.line_height))
}

pub fn og_render_sprite(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d["texture"].as_f64() {
            None => None,
            Some(t) => Some(
                index_of(t)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        // A layer of a texture array in place of the texture (world or screen).
        let array = match d["array"].as_f64() {
            None => None,
            Some(a) => {
                let loaded = index_of(a)
                    .and_then(|i| rs.texture_arrays.get(i))
                    .ok_or_else(|| format!("texture array {a} is not loaded in this world"))?;
                let handle = loaded.handle.ok_or_else(|| {
                    format!(
                        "{}: its streaming group is not resident (the array loads with it)",
                        loaded.id
                    )
                })?;
                let layer = d["arrayLayer"].as_f64().unwrap_or(0.0);
                if !(layer >= 0.0 && layer.fract() == 0.0 && layer < f64::from(loaded.layers)) {
                    return Err(format!(
                        "arrayLayer {layer}: {} has layers 0 to {}",
                        loaded.id,
                        loaded.layers - 1
                    ));
                }
                Some((handle, layer as u32, loaded.size))
            }
        };
        let uv = d["uv"]
            .as_array()
            .filter(|a| a.len() == 4)
            .map(|a| {
                let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
                [n(0), n(1), n(2), n(3)]
            })
            .unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let c = &d["color"];
        let anchor = &d["anchor"];
        new_frame(&mut rs, tick);
        let fc = &d["flashColor"];
        let flash = [
            if fc.is_object() {
                num(fc, "r", 1.0)
            } else {
                1.0
            },
            if fc.is_object() {
                num(fc, "g", 1.0)
            } else {
                1.0
            },
            if fc.is_object() {
                num(fc, "b", 1.0)
            } else {
                1.0
            },
            num(&d, "flash", 0.0).clamp(0.0, 1.0),
        ];
        let sprite = og_render::Sprite {
            texture,
            position: vec3(&d["position"]),
            size: glam::Vec2::new(num(&d, "width", 1.0), num(&d, "height", 1.0)),
            anchor: glam::Vec2::new(num(anchor, "x", 0.5), num(anchor, "y", 0.5)),
            rotation: num(&d, "rotation", 0.0),
            color: [
                num(c, "r", 1.0),
                num(c, "g", 1.0),
                num(c, "b", 1.0),
                num(&d, "opacity", 1.0),
            ],
            uv_rect: uv,
            layer: d["layer"].as_i64().unwrap_or(0) as i32,
            clip: None,
        };
        // Flips swap the region's edges; a nine-slice becomes its pieces.
        let mut sprite = sprite;
        if d["flipX"].as_bool().unwrap_or(false) {
            sprite.uv_rect.swap(0, 2);
        }
        if d["flipY"].as_bool().unwrap_or(false) {
            sprite.uv_rect.swap(1, 3);
        }
        let screen = d["screen"].as_bool().unwrap_or(false);
        let order = if d["sortY"].as_bool().unwrap_or(false) {
            if screen {
                sprite.position.y
            } else {
                -sprite.position.y
            }
        } else {
            0.0
        };
        let size = match array {
            Some((.., size)) => Some(size),
            None => texture.and_then(|t| rs.scene.texture_size(t)),
        };
        let pieces = match (d["slice"].as_array(), size) {
            (Some(a), Some(size)) if a.len() == 4 => {
                let n = |i: usize| a[i].as_f64().unwrap_or(0.0) as f32;
                og_render::nine_slice(
                    &sprite,
                    [n(0), n(1), n(2), n(3)],
                    size,
                    num(&d, "sliceScale", 0.0625),
                )
            }
            _ => vec![sprite],
        };
        let facing = if screen {
            og_render::SpriteFacing::None
        } else {
            super::feel::facing_named(d["facing"].as_str())
        };
        // A faced nine-slice turns as one panel about the sprite's anchor.
        let one_panel = facing != og_render::SpriteFacing::None && pieces.len() > 1;
        for (i, piece) in pieces.into_iter().enumerate() {
            rs.fx.sprites.push(super::fx::QueuedSprite {
                sprite: if one_panel {
                    piece.anchored_at(sprite.position)
                } else {
                    piece
                },
                flash,
                glow: num(&d, "glow", 0.0).max(0.0),
                additive: d["blend"].as_str() == Some("additive"),
                cast_shadow: i == 0 && d["castShadow"].as_bool().unwrap_or(false),
                screen,
                clip: clip_of(&d["clip"]),
                order,
                array: array.map(|(h, layer, _)| (h, layer)),
                drop_shadow: num(&d, "dropShadow", 0.0).clamp(0.0, 1.0),
                drop_shadow_height: num(&d, "dropShadowHeight", 0.0),
                tone_mapped: d["toneMapped"].as_bool().unwrap_or(true),
                facing,
            });
        }
        Ok(())
    })())
}

pub fn og_render_ribbon(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let num = |v: &Value, k: &str, def: f64| v[k].as_f64().unwrap_or(def) as f32;
        let points: Vec<og_render::RibbonPoint> = d["points"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|p| {
                        let c = &p["color"];
                        let rgb = |k: &str| if c.is_object() { num(c, k, 1.0) } else { 1.0 };
                        og_render::RibbonPoint {
                            position: vec3(&p["position"]),
                            width: num(p, "width", 0.1),
                            color: [rgb("r"), rgb("g"), rgb("b"), num(p, "opacity", 1.0)],
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if points.len() < 2 {
            return Err(format!(
                "a ribbon needs at least two points, got {}",
                points.len()
            ));
        }
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d["texture"].as_f64() {
            None => None,
            Some(t) => Some(
                index_of(t)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        let tile = num(&d, "uvTile", 0.0);
        new_frame(&mut rs, tick);
        rs.fx.ribbons.push(og_render::Ribbon {
            points,
            texture,
            uv: if tile > 0.0 {
                og_render::RibbonUv::Tile { length: tile }
            } else {
                og_render::RibbonUv::Stretch
            },
            blend: if d["blend"].as_str() == Some("alpha") {
                og_render::ParticleBlend::Alpha
            } else {
                og_render::ParticleBlend::Additive
            },
            effect: None,
        });
        Ok(())
    })())
}

pub fn og_render_measure_text(world: u32, desc: Option<String>) -> String {
    result((|| {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let font = d["font"]
            .as_f64()
            .ok_or("TextMeasureDesc.font is null: load one with og.load.font")?;
        let fi = index_of(font).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let size = d["size"].as_f64().unwrap_or(1.0) as f32;
        let text = d["text"].as_str().unwrap_or_default();
        let rs = eng.render.borrow();
        let (metrics, line_height) = font_metrics(&rs, render)?;
        let lines: Vec<&str> = text.split('\n').collect();
        let width = lines
            .iter()
            .map(|l| metrics.measure(l, size))
            .fold(0.0f32, f32::max);
        Ok(
            json!({ "width": width, "height": lines.len() as f32 * line_height * size, "lines": lines.len() }),
        )
    })())
}
