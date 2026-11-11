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
    let t: wire::StandardMaterialDesc = wire::value(desc, "desc")?;
    // Patch: only the fields the desc names (not null) change.
    let given = |k: &str| desc.get(k).is_some_and(|v| !v.is_null());
    let num = |k: &str, v: f32| given(k).then_some(v);
    let flag = |k: &str, v: bool| given(k).then_some(u32::from(v));
    let rgb = |k: &str, c: &wire::Color| given(k).then(|| glam::Vec3::new(c.r, c.g, c.b));
    let vec2 = |k: &str, v: glam::Vec2| given(k).then_some(v);
    // A texture field the desc names: null clears it (slot 0).
    let tex = |k: &str, h: Option<u64>| -> Result<Option<u32>, String> {
        match (desc.get(k), h) {
            (None, _) => Ok(None),
            (Some(_), None) => Ok(Some(0)),
            (Some(_), Some(t)) => index_of(t as f64)
                .and_then(|i| textures.get(i))
                .map(|h| Some(h.slot()))
                .ok_or_else(|| format!("{k}: texture {t} is not alive")),
        }
    };
    macro_rules! set {
        ($field:ident, $v:expr) => {
            if let Some(v) = $v {
                m.$field = v;
            }
        };
    }
    set!(color, rgb("color", &t.color));
    set!(map, tex("map", t.map)?);
    set!(metalness, num("metalness", t.metalness));
    set!(metalnessMap, tex("metalnessMap", t.metalness_map)?);
    set!(roughness, num("roughness", t.roughness));
    set!(roughnessMap, tex("roughnessMap", t.roughness_map)?);
    set!(normalMap, tex("normalMap", t.normal_map)?);
    set!(aoMap, tex("aoMap", t.ao_map)?);
    set!(emissive, rgb("emissive", &t.emissive));
    set!(
        emissiveIntensity,
        num("emissiveIntensity", t.emissive_intensity)
    );
    set!(emissiveMap, tex("emissiveMap", t.emissive_map)?);
    set!(opacity, num("opacity", t.opacity));
    set!(transparent, flag("transparent", t.transparent));
    set!(alphaTest, num("alphaTest", t.alpha_test));
    set!(side, given("side").then_some(t.side as u32));
    set!(flatShading, flag("flatShading", t.flat_shading));
    set!(wireframe, flag("wireframe", t.wireframe));
    set!(
        alphaToCoverage,
        flag("alphaToCoverage", t.alpha_to_coverage)
    );
    set!(paintDetail, num("paintDetail", t.paint_detail));
    set!(weathering, num("weathering", t.weathering));
    set!(wireframeAngle, num("wireframeAngle", t.wireframe_angle));
    set!(wireframeColor, rgb("wireframeColor", &t.wireframe_color));
    // Patch within foliage too: only the members it names.
    let foliage = |k: &str| desc["foliage"].get(k).is_some_and(|v| !v.is_null());
    if foliage("translucency") {
        m.foliage.x = t.foliage.translucency;
    }
    if foliage("domeNormals") {
        m.foliage.y = t.foliage.dome_normals;
    }
    set!(envMapIntensity, num("envMapIntensity", t.env_map_intensity));
    set!(normalScale, vec2("normalScale", t.normal_scale));
    set!(aoMapIntensity, num("aoMapIntensity", t.ao_map_intensity));
    set!(unlit, flag("unlit", t.unlit));
    set!(styled, flag("styled", t.styled));
    set!(toon, flag("toon", t.toon));
    set!(gradientMap, tex("gradientMap", t.gradient_map)?);
    set!(rimColor, rgb("rimColor", &t.rim_color));
    set!(rimIntensity, num("rimIntensity", t.rim_intensity));
    set!(rimPower, num("rimPower", t.rim_power));
    set!(flashColor, rgb("flashColor", &t.flash_color));
    set!(flash, num("flash", t.flash));
    set!(dissolve, num("dissolve", t.dissolve));
    set!(dissolveMap, tex("dissolveMap", t.dissolve_map)?);
    set!(
        dissolveEdgeColor,
        rgb("dissolveEdgeColor", &t.dissolve_edge_color)
    );
    set!(
        dissolveEdgeWidth,
        num("dissolveEdgeWidth", t.dissolve_edge_width)
    );
    set!(dissolveScale, num("dissolveScale", t.dissolve_scale));
    set!(outlineColor, rgb("outlineColor", &t.outline_color));
    set!(outlineWidth, num("outlineWidth", t.outline_width));
    set!(clearcoat, num("clearcoat", t.clearcoat));
    set!(
        clearcoatRoughness,
        num("clearcoatRoughness", t.clearcoat_roughness)
    );
    set!(clearcoatMap, tex("clearcoatMap", t.clearcoat_map)?);
    set!(
        clearcoatRoughnessMap,
        tex("clearcoatRoughnessMap", t.clearcoat_roughness_map)?
    );
    set!(
        clearcoatNormalMap,
        tex("clearcoatNormalMap", t.clearcoat_normal_map)?
    );
    set!(
        clearcoatNormalScale,
        vec2("clearcoatNormalScale", t.clearcoat_normal_scale)
    );
    set!(sheen, num("sheen", t.sheen));
    set!(sheenColor, rgb("sheenColor", &t.sheen_color));
    set!(sheenRoughness, num("sheenRoughness", t.sheen_roughness));
    set!(sheenColorMap, tex("sheenColorMap", t.sheen_color_map)?);
    set!(
        sheenRoughnessMap,
        tex("sheenRoughnessMap", t.sheen_roughness_map)?
    );
    set!(transmission, num("transmission", t.transmission));
    set!(transmissionMap, tex("transmissionMap", t.transmission_map)?);
    set!(thickness, num("thickness", t.thickness));
    set!(thicknessMap, tex("thicknessMap", t.thickness_map)?);
    set!(
        attenuationDistance,
        num("attenuationDistance", t.attenuation_distance)
    );
    set!(
        attenuationColor,
        rgb("attenuationColor", &t.attenuation_color)
    );
    set!(ior, num("ior", t.ior));
    set!(iridescence, num("iridescence", t.iridescence));
    set!(iridescenceIOR, num("iridescenceIOR", t.iridescence_ior));
    set!(
        iridescenceThicknessRange,
        vec2("iridescenceThicknessRange", t.iridescence_thickness_range)
    );
    set!(iridescenceMap, tex("iridescenceMap", t.iridescence_map)?);
    set!(
        iridescenceThicknessMap,
        tex("iridescenceThicknessMap", t.iridescence_thickness_map)?
    );
    set!(anisotropy, num("anisotropy", t.anisotropy));
    set!(
        anisotropyRotation,
        num("anisotropyRotation", t.anisotropy_rotation)
    );
    set!(anisotropyMap, tex("anisotropyMap", t.anisotropy_map)?);
    // Each slot's uv transform (KHR_texture_transform, three.js Texture.offset/repeat/rotation).
    set!(mapOffset, vec2("mapOffset", t.map_offset));
    set!(mapRepeat, vec2("mapRepeat", t.map_repeat));
    set!(mapRotation, num("mapRotation", t.map_rotation));
    set!(
        metalnessMapOffset,
        vec2("metalnessMapOffset", t.metalness_map_offset)
    );
    set!(
        metalnessMapRepeat,
        vec2("metalnessMapRepeat", t.metalness_map_repeat)
    );
    set!(
        metalnessMapRotation,
        num("metalnessMapRotation", t.metalness_map_rotation)
    );
    set!(
        normalMapOffset,
        vec2("normalMapOffset", t.normal_map_offset)
    );
    set!(
        normalMapRepeat,
        vec2("normalMapRepeat", t.normal_map_repeat)
    );
    set!(
        normalMapRotation,
        num("normalMapRotation", t.normal_map_rotation)
    );
    set!(
        emissiveMapOffset,
        vec2("emissiveMapOffset", t.emissive_map_offset)
    );
    set!(
        emissiveMapRepeat,
        vec2("emissiveMapRepeat", t.emissive_map_repeat)
    );
    set!(
        emissiveMapRotation,
        num("emissiveMapRotation", t.emissive_map_rotation)
    );
    set!(aoMapOffset, vec2("aoMapOffset", t.ao_map_offset));
    set!(aoMapRepeat, vec2("aoMapRepeat", t.ao_map_repeat));
    set!(aoMapRotation, num("aoMapRotation", t.ao_map_rotation));
    // A texture-array field the desc names: null clears it (slot 0).
    let array = |k: &str, h: Option<u64>| -> Result<Option<u32>, String> {
        match (desc.get(k), h) {
            (None, _) => Ok(None),
            (Some(_), None) => Ok(Some(0)),
            (Some(_), Some(a)) => {
                let loaded = index_of(a as f64)
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
    set!(mapArray, array("mapArray", t.map_array)?);
    set!(mapLayer, given("mapLayer").then_some(t.map_layer));
    set!(normalMapArray, array("normalMapArray", t.normal_map_array)?);
    set!(
        normalMapLayer,
        given("normalMapLayer").then_some(t.normal_map_layer)
    );
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
        let d: wire::TextDesc = wire::arg(desc.as_deref(), "desc")?;
        let Some(font) = d.font else {
            return Ok(());
        };
        let fi = index_of(font as f64).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        new_frame(&mut rs, tick);
        let text = d.text;
        // Its missing glyphs warned of now, on the CPU, renderer or not.
        if let Some(missing) = crate::glyphs::lacking(&mut rs, render, &text) {
            drop(rs);
            crate::glyphs::warn_missing(&eng, tick, vec![missing]);
            rs = eng.render.borrow_mut();
        }
        let size = d.size;
        let rotation = d.rotation;
        let position = d.position;
        let color = [d.color.r, d.color.g, d.color.b, d.opacity];
        let layer = d.layer;
        let screen = d.screen;
        let clip = d.clip;
        let outline = d.outline.clamp(0.0, 1.0);
        let oc = &d.outline_color;
        let outline_color = [oc.r, oc.g, oc.b, color[3]];
        let facing = super::feel::facing_named(Some(d.facing.as_str()));
        let shift = match d.align {
            wire::TextAlign::Start => 0.0,
            wire::TextAlign::Center => 0.5,
            wire::TextAlign::End => 1.0,
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
    let axis = |k: &str| r[k].as_f64().unwrap_or(1.0) as f32;
    repeat_uv(glam::Vec2::new(axis("x"), axis("y")), mesh)
}

/// A geometry desc's typed `uvRepeat` applied (og_render::shapes::repeat_uv).
fn repeat_uv(r: glam::Vec2, mesh: &mut og_render::MeshData) -> Result<(), String> {
    for (k, v) in [("x", r.x), ("y", r.y)] {
        if !v.is_finite() {
            return Err(format!("uvRepeat.{k} must be finite, got {v}"));
        }
    }
    og_render::shapes::repeat_uv(mesh, r);
    Ok(())
}

/// A geometry desc's size field `v`, checked.
fn size_of(key: &str, v: f32) -> Result<f32, String> {
    if !(v.is_finite() && v >= 0.0) {
        return Err(format!("{key} must be finite and at least 0, got {v}"));
    }
    Ok(v)
}

pub fn og_render_box_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d: wire::BoxGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let size = glam::Vec3::new(
            size_of("width", d.width)?,
            size_of("height", d.height)?,
            size_of("depth", d.depth)?,
        );
        let mut mesh = og_render::shapes::cuboid(size);
        repeat_uv(d.uv_repeat, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_plane_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d: wire::PlaneGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let mut mesh =
            og_render::shapes::plane_xy(size_of("width", d.width)?, size_of("height", d.height)?);
        repeat_uv(d.uv_repeat, &mut mesh)?;
        add_geometry(world, mesh, true)
    })())
}

pub fn og_render_sphere_geometry(world: u32, desc: Option<String>) -> String {
    result((|| {
        let d: wire::SphereGeometryDesc = wire::arg(desc.as_deref(), "desc")?;
        let radius = size_of("radius", d.radius)?;
        let seg = d.width_segments.clamp(3, 512);
        let rings = d.height_segments.clamp(2, 512);
        let mut mesh = og_render::shapes::sphere(seg, rings);
        for v in &mut mesh.vertices {
            for c in &mut v.position {
                *c *= radius * 2.0;
            }
        }
        repeat_uv(d.uv_repeat, &mut mesh)?;
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
        let d: wire::SpriteDesc = wire::arg(desc.as_deref(), "desc")?;
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d.texture {
            None => None,
            Some(t) => Some(
                index_of(t as f64)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        // A layer of a texture array in place of the texture (world or screen).
        let array = match d.array {
            None => None,
            Some(a) => {
                let loaded = index_of(a as f64)
                    .and_then(|i| rs.texture_arrays.get(i))
                    .ok_or_else(|| format!("texture array {a} is not loaded in this world"))?;
                let handle = loaded.handle.ok_or_else(|| {
                    format!(
                        "{}: its streaming group is not resident (the array loads with it)",
                        loaded.id
                    )
                })?;
                let layer = d.array_layer;
                if layer >= loaded.layers {
                    return Err(format!(
                        "arrayLayer {layer}: {} has layers 0 to {}",
                        loaded.id,
                        loaded.layers - 1
                    ));
                }
                Some((handle, layer, loaded.size))
            }
        };
        let uv = d.uv.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let c = &d.color;
        new_frame(&mut rs, tick);
        let fc = &d.flash_color;
        let flash = [fc.r, fc.g, fc.b, d.flash.clamp(0.0, 1.0)];
        let sprite = og_render::Sprite {
            texture,
            position: d.position,
            size: glam::Vec2::new(d.width, d.height),
            anchor: d.anchor,
            rotation: d.rotation,
            color: [c.r, c.g, c.b, d.opacity],
            uv_rect: uv,
            layer: d.layer,
            clip: None,
        };
        // Flips swap the region's edges; a nine-slice becomes its pieces.
        let mut sprite = sprite;
        if d.flip_x {
            sprite.uv_rect.swap(0, 2);
        }
        if d.flip_y {
            sprite.uv_rect.swap(1, 3);
        }
        let screen = d.screen;
        let order = if d.sort_y {
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
        let pieces = match (d.slice, size) {
            (Some(slice), Some(size)) => og_render::nine_slice(&sprite, slice, size, d.slice_scale),
            _ => vec![sprite],
        };
        let facing = if screen {
            og_render::SpriteFacing::None
        } else {
            super::feel::facing_named(Some(d.facing.as_str()))
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
                glow: d.glow.max(0.0),
                additive: d.blend == wire::SpriteBlend::Additive,
                cast_shadow: i == 0 && d.cast_shadow,
                screen,
                clip: d.clip,
                order,
                array: array.map(|(h, layer, _)| (h, layer)),
                drop_shadow: d.drop_shadow.clamp(0.0, 1.0),
                drop_shadow_height: d.drop_shadow_height,
                tone_mapped: d.tone_mapped,
                facing,
            });
        }
        Ok(())
    })())
}

pub fn og_render_ribbon(world: u32, desc: Option<String>) -> String {
    result_void((|| {
        let eng = engine(world)?;
        let d: wire::RibbonDesc = wire::arg(desc.as_deref(), "desc")?;
        let points: Vec<og_render::RibbonPoint> = d
            .points
            .iter()
            .map(|p| og_render::RibbonPoint {
                position: p.position,
                width: p.width,
                color: [p.color.r, p.color.g, p.color.b, p.opacity],
            })
            .collect();
        if points.len() < 2 {
            return Err(format!(
                "a ribbon needs at least two points, got {}",
                points.len()
            ));
        }
        let tick = og_world_get_tick(world) as u64;
        let mut rs = eng.render.borrow_mut();
        let texture = match d.texture {
            None => None,
            Some(t) => Some(
                index_of(t as f64)
                    .and_then(|i| rs.textures.get(i).copied())
                    .ok_or_else(|| format!("texture {t} is not alive"))?,
            ),
        };
        let tile = d.uv_tile;
        new_frame(&mut rs, tick);
        rs.fx.ribbons.push(og_render::Ribbon {
            points,
            texture,
            uv: if tile > 0.0 {
                og_render::RibbonUv::Tile { length: tile }
            } else {
                og_render::RibbonUv::Stretch
            },
            blend: if d.blend == wire::StripBlend::Alpha {
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
        let d: wire::TextMeasureDesc = wire::arg(desc.as_deref(), "desc")?;
        let font = d
            .font
            .ok_or("TextMeasureDesc.font is null: load one with og.load.font")?;
        let fi = index_of(font as f64).ok_or_else(|| format!("{font} is not a font handle"))?;
        let render = eng
            .assets
            .borrow()
            .fonts
            .get(fi)
            .map(|f| f.render)
            .ok_or_else(|| format!("font {font} is not alive"))?;
        let size = d.size;
        let text = d.text.as_str();
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
