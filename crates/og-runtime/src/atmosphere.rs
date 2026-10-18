//! The E5 round's rendering from the world (`og.atmosphere` and the
//! drawing of the physics that has a look): og.Fog, og.Sky, og.DayNight
//! and og.Weather into the scene's fog, sky, light and particles; og.Water
//! (with an og.WaterSurface's heights), og.Water2D over an og.WaterLine,
//! og.Fluid2D as metaballs, visible og.Rope and og.Cloth; and the material
//! shaders (`og.load.shader`, `og.render.shaderMaterial`,
//! `og.render.setShaderParam`). Like the rest of extraction it reads the
//! world (and the physics' last step) and never writes either.

use super::codec::Layout;
use super::engine::engine;
use super::extract::{Hierarchy, RenderState};
use super::fx::{color, f, on, resource, rows, u, vec2};
use super::{ok, ok_void, parse, refuse};
use glam::{Mat4, Vec2, Vec3};
use og_ecs::{Entity, World};
use og_render::{
    Background, Fog, MaterialHandle, MeshData, MeshHandle, Metaballs, ParticleSystem,
    ProceduralSky, Projection, Ribbon, RibbonPoint, RibbonUv, SkyLook, Sprite, Vertex, Water,
    Water2d, WaterHeights,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The layouts of the records this module reads, and the physics whose
/// last step it draws.
#[derive(Default)]
pub(crate) struct AtmoLayouts {
    fog: Option<Layout>,
    sky: Option<Layout>,
    day_night: Option<Layout>,
    weather: Option<Layout>,
    water: Option<Layout>,
    water_2d: Option<Layout>,
    water_line: Option<Layout>,
    fluid: Option<Layout>,
    rope: Option<Layout>,
    cloth: Option<Layout>,
    pub ragdoll: Option<Layout>,
    pub physics: Option<og_physics::PhysicsSystem>,
}

impl AtmoLayouts {
    pub fn from(
        get: impl Fn(&str) -> Option<Layout>,
        physics: Option<og_physics::PhysicsSystem>,
    ) -> AtmoLayouts {
        AtmoLayouts {
            fog: get("og.Fog"),
            sky: get("og.Sky"),
            day_night: get("og.DayNight"),
            weather: get("og.Weather"),
            water: get("og.Water"),
            water_2d: get("og.Water2D"),
            water_line: get("og.WaterLine"),
            fluid: get("og.Fluid2D"),
            rope: get("og.Rope"),
            cloth: get("og.Cloth"),
            ragdoll: get("og.Ragdoll"),
            physics,
        }
    }
}

/// What this module keeps between frames: the weather's emitter, the
/// cloths' meshes, the loaded shaders and the shader materials.
#[derive(Default)]
pub(crate) struct AtmoState {
    weather: Option<(Vec<u32>, og_render::EmitterHandle)>,
    cloths: BTreeMap<u64, MeshHandle>,
    cloth_material: Option<MaterialHandle>,
    /// `og.load.shader` handles by load index, and loads by path.
    pub shaders: Vec<og_render::ShaderHandle>,
    pub shader_paths: BTreeMap<String, usize>,
    /// Ragdoll rigs by entity (`og.physics.setRagdollRig`), replayed into
    /// a reloaded world's physics, and this frame's palette of each posed
    /// model by entity.
    pub rigs: BTreeMap<u64, super::physics_ext::KeptRig>,
    pub palettes: BTreeMap<u64, u32>,
}

/// The particle system id of the weather (no entity has it).
const WEATHER_ID: u64 = u64::MAX - 0x5745;

/// The DayNight clock at `tick` (hours, 0..24), from its row.
fn hours(d: &Value, tick: u64, dt: f32) -> f32 {
    let start = f64::from(f(d, "start", 9.0));
    if !on(d, "enabled", false) {
        return start.rem_euclid(24.0) as f32;
    }
    let speed = f64::from(f(d, "speed", 0.05));
    (start + tick as f64 * f64::from(dt) * speed).rem_euclid(24.0) as f32
}

/// og.Fog, og.Sky, og.DayNight, og.Weather, og.Water, og.Water2D,
/// og.Fluid2D, og.Rope and og.Cloth into `rs.scene`.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy) {
    let RenderState {
        scene, draw, atmo, ..
    } = rs;
    let layouts = &draw.atmo;
    scene.fog = None;
    scene.water.clear();
    scene.water_2d.clear();
    scene.metaballs.clear();
    let time = world.time();
    let tick = time.tick;
    let orthographic = matches!(scene.camera.projection, Projection::Orthographic { .. });

    // The sky: a chosen look, drawn per pixel and lit by.
    if let Some(s) = resource(world, layouts.sky.as_ref())
        && on(&s, "enabled", false)
    {
        let look = SkyLook {
            zenith: color(&s["zenith"], Vec3::new(0.18, 0.36, 0.78)),
            horizon: color(&s["horizon"], Vec3::new(0.78, 0.84, 0.92)),
            ground: color(&s["ground"], Vec3::new(0.16, 0.14, 0.12)),
            sun_size: f(&s, "sunSize", 1.8),
            sun_intensity: f(&s, "sunIntensity", 40.0),
            glow: f(&s, "glow", 0.6),
            glow_power: f(&s, "glowPower", 24.0),
            stars: f(&s, "stars", 0.0),
            brightness: f(&s, "brightness", 1.1),
        };
        let dir = super::fx::vec3(&s["sunDirection"]).normalize_or(Vec3::Y);
        let sun = color(&s["sunColor"], Vec3::new(1.0, 0.95, 0.88));
        scene.environment.sky = ProceduralSky {
            sun_direction: dir,
            sun_color: sun,
            look: Some(look),
        };
        scene.environment.background = Background::Procedural {
            sun_direction: dir,
            sun_color: sun,
        };
    }

    // The day-night cycle: light, sky and ambient from the clock.
    let mut fog_tint = None;
    if let Some(d) = resource(world, layouts.day_night.as_ref())
        && on(&d, "enabled", false)
    {
        let (tilt, azimuth) = (f(&d, "tilt", 25.0), f(&d, "azimuth", 0.0));
        let now = hours(&d, tick, time.dt);
        let look = og_render::day_night(now, tilt, azimuth);
        // The image-based light steps every `step` game minutes.
        let step = f(&d, "step", 10.0).clamp(1.0, 120.0) / 60.0;
        let stepped = og_render::day_night((now / step).floor() * step, tilt, azimuth);
        let scale = f(&d, "lightIntensity", 1.0).max(0.0);
        scene.light.direction_to_light = look.light_direction;
        scene.light.color = look.light_color;
        scene.light.intensity = look.light_intensity * scale;
        scene.environment.intensity = look.ambient;
        scene.environment.sky = ProceduralSky {
            sun_direction: stepped.sun_direction,
            sun_color: stepped.sun_color,
            look: Some(stepped.sky),
        };
        scene.environment.background = Background::Procedural {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
        };
        if on(&d, "tintFog", true) {
            fog_tint = Some(look.fog_color);
        }
    }

    // Fog, declared or from the weather.
    if let Some(v) = resource(world, layouts.fog.as_ref())
        && on(&v, "enabled", false)
    {
        let mut fog = Fog {
            color: color(&v["color"], Vec3::new(0.62, 0.68, 0.76)),
            density: f(&v, "density", 0.02),
            start: f(&v, "start", 0.0),
            end: f(&v, "end", 0.0),
            height_density: f(&v, "heightDensity", 0.0),
            height: f(&v, "height", 0.0),
            falloff: f(&v, "falloff", 0.5),
            sun_color: color(&v["sunColor"], Vec3::new(1.0, 0.86, 0.62)),
            sun_power: f(&v, "sunPower", 8.0),
            max_opacity: f(&v, "maxOpacity", 1.0),
            sky: f(&v, "sky", 1.0),
        };
        if let Some(t) = fog_tint {
            fog.color = fog.color.lerp(t, 0.75);
        }
        scene.fog = Some(fog);
    }
    if let Some(w) = resource(world, layouts.weather.as_ref()) {
        let kind = u(&w, "kind", 0);
        let layer = orthographic.then(|| w["layer"].as_i64().unwrap_or(50) as i32);
        let wind = super::fx::vec3(&w["wind"]);
        let look = og_render::weather(
            kind,
            f(&w, "intensity", 1.0),
            wind,
            f(&w, "radius", 18.0),
            color(&w["tint"], Vec3::ONE),
            f(&w, "dim", 1.0),
            layer,
        );
        if scene.fog.is_none()
            && let Some(mut fog) = look.fog
        {
            if let Some(t) = fog_tint {
                fog.color = fog.color.lerp(t, 0.5);
            }
            scene.fog = Some(fog);
        }
        scene.light.intensity *= look.light;
        scene.environment.intensity *= 0.5 + 0.5 * look.light;
        if let Some(emitter) = &look.emitter {
            // The emitter is remade only when its inputs change.
            let key: Vec<u32> = [
                kind as f32,
                f(&w, "intensity", 1.0),
                wind.x,
                wind.y,
                wind.z,
                f(&w, "radius", 18.0),
                layer.map_or(f32::NAN, |l| l as f32),
            ]
            .iter()
            .map(|x| x.to_bits())
            .chain([
                color(&w["tint"], Vec3::ONE).x.to_bits(),
                color(&w["tint"], Vec3::ONE).y.to_bits(),
                color(&w["tint"], Vec3::ONE).z.to_bits(),
            ])
            .collect();
            let handle = match atmo.weather.as_ref().map(|(k, h)| (k == &key, *h)) {
                Some((true, handle)) => handle,
                Some((false, handle)) => {
                    scene.set_emitter(handle, emitter.clone());
                    atmo.weather = Some((key, handle));
                    handle
                }
                None => {
                    let handle = scene.add_emitter(emitter.clone());
                    atmo.weather = Some((key, handle));
                    handle
                }
            };
            let cam = &scene.camera;
            let at = look.origin(cam.position, cam.target - cam.position);
            let at = if orthographic {
                Vec3::new(cam.position.x, cam.position.y + look.offset.y, 0.0)
            } else {
                at
            };
            scene.particles.push(ParticleSystem::new(
                WEATHER_ID,
                handle,
                Mat4::from_translation(at),
                1,
                tick,
            ));
        }
    }

    // Ragdolls blended into their models' palettes.
    super::physics_ext::draw_ragdolls(world, rs, &mut |e| h.world_of(e));
    let RenderState {
        scene,
        draw,
        atmo,
        materials,
        ..
    } = rs;
    let layouts = &draw.atmo;
    let physics = layouts.physics.as_ref();
    // 3D water, with its wave heights when the entity has them.
    for (e, v) in rows(world, layouts.water.as_ref()) {
        let entity = Entity::from_bits(e);
        let field = physics.and_then(|p| p.water_heights(entity));
        let m = h.world_of(e);
        let transform = if field.is_some() {
            // The wave field is centred on the entity and unrotated.
            Mat4::from_translation(m.transform_point3(Vec3::ZERO))
        } else {
            let (_, r, t) = m.to_scale_rotation_translation();
            Mat4::from_rotation_translation(r, t)
        };
        scene.water.push(Water {
            transform,
            size: vec2(&v["size"], Vec2::splat(16.0)),
            segments: u(&v, "segments", 64),
            shallow: color(&v["shallow"], Vec3::new(0.55, 0.85, 0.82)),
            deep: color(&v["deep"], Vec3::new(0.02, 0.09, 0.14)),
            depth_fade: f(&v, "depthFade", 2.5),
            opacity: f(&v, "opacity", 0.92),
            foam: color(&v["foam"], Vec3::new(0.95, 0.97, 1.0)),
            foam_width: f(&v, "foamWidth", 0.35),
            wave_scale: f(&v, "waveScale", 0.6),
            wave_speed: f(&v, "waveSpeed", 1.0),
            wave_strength: f(&v, "waveStrength", 0.35),
            refraction: f(&v, "refraction", 1.0),
            reflectivity: f(&v, "reflectivity", 1.0),
            heights: field.map(|w| WaterHeights {
                resolution: w.resolution,
                size: w.size,
                heights: Arc::from(w.heights),
            }),
        });
    }
    // 2D water over its spring line.
    let lines: BTreeMap<u64, Value> = rows(world, layouts.water_line.as_ref())
        .into_iter()
        .collect();
    for (e, v) in rows(world, layouts.water_2d.as_ref()) {
        let (Some(line), Some(p)) = (lines.get(&e), physics) else {
            continue;
        };
        let at = h.world_of(e).transform_point3(Vec3::ZERO);
        let width = f(line, "width", 12.0);
        let surface = p
            .water_line(Entity::from_bits(e))
            .unwrap_or_else(|| vec![0.0; u(line, "columns", 96).max(2) as usize]);
        let rgba = |c: &str, a: &str, dc: Vec3, da: f32| color(&v[c], dc).extend(f(&v, a, da));
        scene.water_2d.push(Water2d {
            origin: Vec2::new(at.x - width * 0.5, at.y),
            width,
            depth: f(line, "depth", 3.0),
            surface,
            top: rgba("top", "topOpacity", Vec3::new(0.16, 0.5, 0.72), 0.72),
            bottom: rgba("bottom", "bottomOpacity", Vec3::new(0.02, 0.1, 0.25), 0.92),
            lip: color(&v["lip"], Vec3::new(0.8, 0.95, 1.0)).extend(0.9),
            lip_width: f(&v, "lipWidth", 0.06),
            layer: v["layer"].as_i64().unwrap_or(0) as i32,
            z: at.z,
        });
    }
    let Some(physics) = physics else {
        return;
    };
    // Fluids as metaballs.
    for (e, v) in rows(world, layouts.fluid.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let Some(particles) = physics.fluid_particles(Entity::from_bits(e)) else {
            continue;
        };
        scene.metaballs.push(Metaballs {
            particles: Arc::from(particles),
            radius: f(&v, "radius", 0.18) * f(&v, "blobScale", 1.6),
            threshold: f(&v, "threshold", 0.45),
            // Fluid2D.color is a Vector3 (x, y, z are r, g, b).
            color: super::fx::vec3(&v["color"]).extend(f(&v, "opacity", 0.9)),
            layer: v["layer"].as_i64().unwrap_or(0) as i32,
            z: h.world_of(e).transform_point3(Vec3::ZERO).z,
            ..Metaballs::default()
        });
    }
    // Ropes: a strip through their points in 3D, a sprite per segment in 2D.
    for (e, v) in rows(world, layouts.rope.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let Some(points) = physics.rope_points(Entity::from_bits(e)) else {
            continue;
        };
        // Rope.color is a Vector3 (x, y, z are r, g, b).
        let c = super::fx::vec3(&v["color"]);
        let width = f(&v, "width", 0.08);
        if orthographic {
            for pair in points.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let d = b - a;
                scene.sprites.push(Sprite {
                    position: ((a + b) * 0.5).truncate().extend(a.z),
                    size: Vec2::new(d.truncate().length() + width, width),
                    rotation: d.y.atan2(d.x),
                    color: [c.x, c.y, c.z, 1.0],
                    ..Default::default()
                });
            }
        } else {
            scene.ribbons.push(Ribbon {
                points: points
                    .iter()
                    .map(|&position| RibbonPoint {
                        position,
                        width: width * 1.6,
                        color: [c.x, c.y, c.z, 1.0],
                    })
                    .collect(),
                texture: None,
                uv: RibbonUv::Stretch,
                blend: og_render::ParticleBlend::Alpha,
            });
        }
    }
    // Cloth: a double-sided mesh of its points, replaced every frame.
    for (e, v) in rows(world, layouts.cloth.as_ref()) {
        if !on(&v, "visible", true) {
            continue;
        }
        let Some(c) = physics.cloth_points(Entity::from_bits(e)) else {
            continue;
        };
        let mesh = cloth_mesh(c.columns, c.rows, &c.points);
        let handle = match atmo.cloths.get(&e) {
            Some(&handle) => {
                scene.set_mesh(handle, mesh);
                handle
            }
            None => {
                let handle = scene.add_mesh(mesh);
                atmo.cloths.insert(e, handle);
                handle
            }
        };
        let material = super::fx::handle_index(&v["material"])
            .and_then(|i| materials.get(i).copied())
            .unwrap_or_else(|| {
                *atmo.cloth_material.get_or_insert_with(|| {
                    scene.add_standard_material(og_render::StandardMaterial {
                        color: Vec3::new(0.72, 0.16, 0.14),
                        roughness: 0.8,
                        side: 2,
                        ..Default::default()
                    })
                })
            });
        scene.add_instance(handle, material, Mat4::IDENTITY);
    }
}

/// The mesh of a `columns` x `rows` grid of world points: smooth normals
/// from the neighbouring points, uv over the grid.
fn cloth_mesh(columns: u32, rows: u32, points: &[Vec3]) -> MeshData {
    let (w, ht) = (columns.max(2) as usize, rows.max(2) as usize);
    if points.len() < w * ht {
        return MeshData {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    let at = |x: usize, y: usize| points[y * w + x];
    let mut vertices = Vec::with_capacity(w * ht);
    for y in 0..ht {
        for x in 0..w {
            let dx = at((x + 1).min(w - 1), y) - at(x.saturating_sub(1), y);
            let dy = at(x, (y + 1).min(ht - 1)) - at(x, y.saturating_sub(1));
            let n = dy.cross(dx).normalize_or(Vec3::Z);
            vertices.push(Vertex {
                position: at(x, y).to_array(),
                normal: n.to_array(),
                uv: [x as f32 / (w - 1) as f32, y as f32 / (ht - 1) as f32],
                ..Default::default()
            });
        }
    }
    let mut indices = Vec::with_capacity((w - 1) * (ht - 1) * 6);
    for y in 0..ht - 1 {
        for x in 0..w - 1 {
            let i = (y * w + x) as u32;
            let (r, d) = (i + 1, i + w as u32);
            indices.extend_from_slice(&[i, d, r, r, d, d + 1]);
        }
    }
    MeshData { vertices, indices }
}

// ── og.atmosphere.timeOfDay ─────────────────────────────────────────────

pub fn og_atmosphere_time_of_day(world: u32) -> f64 {
    let Ok(eng) = engine(world) else {
        return 0.0;
    };
    let Some(layout) = eng.state.borrow().layouts.get("og.DayNight").cloned() else {
        return 9.0;
    };
    // Inside a system, through its context (og.DayNight declared among its
    // reads); between ticks, from the world.
    if let Some(frame) = super::engine::running(world) {
        // SAFETY: the frame is published while this system runs.
        let ctx = unsafe { frame.ctx() };
        let time = ctx.time();
        let d = super::api::running_record(&eng, "og.DayNight", super::codec::RecordKind::Resource)
            .ok()
            .and_then(|(l, id)| ctx.resource_bytes(id).ok().map(|b| l.decode(&b)));
        return d.map_or(9.0, |d| f64::from(hours(&d, time.tick, time.dt)));
    }
    let Ok(w) = eng.world() else {
        return 0.0;
    };
    let time = w.time();
    resource(w, Some(&layout)).map_or(9.0, |d| f64::from(hours(&d, time.tick, time.dt)))
}

// ── material shaders ────────────────────────────────────────────────────

/// `og.load.shader`: a cooked shader, added to the scene once per path.
pub fn og_load_shader(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let id = og_cook::runtime::asset_id(&path).to_string();
        if let Some(&i) = eng.render.borrow().atmo.shader_paths.get(&id) {
            return Ok(json!(super::assets::handle(i)));
        }
        let shader = {
            let assets = eng.assets.borrow();
            let (bytes, bundle) = assets.blob(&id)?;
            bundle
                .shader(&id, bytes.bytes())
                .map_err(|e| e.to_string())?
        };
        let mut rs = eng.render.borrow_mut();
        let h = rs.scene.add_shader(shader);
        rs.atmo.shaders.push(h);
        let i = rs.atmo.shaders.len() - 1;
        rs.atmo.shader_paths.insert(id, i);
        Ok(json!(super::assets::handle(i)))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_render_shader_material(world: u32, shader: f64, desc: Option<String>) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let mut rs = eng.render.borrow_mut();
        let s = super::assets::index_of(shader)
            .and_then(|i| rs.atmo.shaders.get(i).copied())
            .ok_or_else(|| format!("shader {shader} is not loaded in this world"))?;
        let m = super::api::material(og_render::StandardMaterial::default(), &desc, &rs.textures)?;
        let handle = rs.scene.add_standard_material(m);
        rs.scene.set_material_shader(handle, Some(s));
        rs.materials.push(handle);
        Ok(json!(super::api::material_handle(rs.materials.len() - 1)))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_render_set_shader_param(
    world: u32,
    material: f64,
    name: String,
    value: String,
) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let value: Vec<f32> = match serde_json::from_str::<Value>(&value) {
            Ok(Value::Array(a)) => a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect(),
            Ok(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
            _ => return Err("value is a list of numbers".into()),
        };
        let mut rs = eng.render.borrow_mut();
        let h = super::assets::index_of(material)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {material} is not alive"))?;
        if rs.scene.material_shader(h).is_none() {
            return Err(format!(
                "material {material} has no shader (make it with og.render.shaderMaterial)"
            ));
        }
        if !rs.scene.set_shader_param(h, &name, &value) {
            return Err(format!(
                "the material's shader declares no parameter '{name}'"
            ));
        }
        Ok(())
    })();
    match r {
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}
