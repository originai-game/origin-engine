//! Composed effects (`type vfx`, og_bundle::vfx): `og.load.vfx`, the
//! `og.vfx` system that runs their gameplay side in the tick (og.VfxEvent,
//! shakes, sounds, the despawn at the end) and their extraction, which
//! expands every playing og.Vfx into the renderer's particle systems,
//! effect meshes, trails, beams, lights and decals from (asset, seed,
//! startTick, the tick) alone. Nothing extraction makes is world state.
//!
//! og.Vfx and og.VfxEvent are lazy (`super::lazy`): a world that never names
//! them keeps its type table and hash, and has no og.vfx system.

use super::codec::{Layout, RecordKind};
use super::extract::{Hierarchy, RenderState};
use super::wire::{FlipbookInfo, FromJson, record};
use glam::{Mat4, Quat, Vec3};
use og_bundle::vfx::{Vfx, VfxKind, VfxShape};
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use og_render::{
    Beam, ColorCurve, Curve, Decal, EffectMaterial, EffectMaterialHandle, EffectMesh,
    EmitterHandle, Light, LightKind, MeshHandle, ParticleBlend, ParticleSystem, RibbonUv, Trail,
    TrailStyle,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(crate) const COMPONENT: &str = "og.Vfx";
pub(crate) const EVENT: &str = "og.VfxEvent";
pub(crate) const RECORDS: [&str; 2] = [COMPONENT, EVENT];
const SYSTEM: &str = "og.vfx";
/// og.Vfx.startTick's and ran's unwritten value.
const UNSET: u64 = u32::MAX as u64;
/// VfxAsset handles: this tag, then the load index plus one.
const TAG: u64 = 0x7d << 32;

fn handle(index: usize) -> f64 {
    (TAG | (index as u64 + 1)) as f64
}

/// A VfxAsset handle's load index.
fn index(h: u64) -> Option<usize> {
    (h & !0xffff_ffff == TAG && h & 0xffff_ffff > 0).then(|| (h & 0xffff_ffff) as usize - 1)
}

/// `og.VfxEvent`, registered from this type (its name is a str64), not
/// from the IDL's declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, og_core::Reflect)]
#[reflect(name = "og.VfxEvent")]
pub struct VfxEvent {
    pub entity: u64,
    pub asset: u64,
    pub name: og_core::Str64,
    pub time: f32,
}

/// What an element draws from or plays, linked at load.
#[derive(Clone, Debug)]
pub(crate) enum Link {
    Emitter(EmitterHandle),
    Mesh(MeshHandle, EffectMaterialHandle),
    Effect(Option<EffectMaterialHandle>),
    /// A sound's handle (og.AudioSource.sound).
    Sound(f64),
    /// A flipbook sheet (og.load.flipbook): its handle, texture, the first
    /// frame's width over height and the first clip's length.
    Flipbook {
        sheet: f64,
        texture: f64,
        aspect: f32,
        clip: f32,
    },
    None,
}

/// A loaded composed effect.
#[derive(Debug)]
pub(crate) struct Loaded {
    pub id: String,
    pub vfx: Vfx,
    /// Per element, in file order.
    pub links: Vec<Link>,
}

/// The effects a world loaded, by load index; shared by the loads, the
/// og.vfx system and extraction.
pub(crate) type Registry = Arc<Mutex<Vec<Arc<Loaded>>>>;

/// og.vfx on one engine: the effects its world loaded.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Registry);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── loading ─────────────────────────────────────────────────────────────────

/// A built-in effect mesh, made once.
fn builtin_mesh(eng: &super::engine::Engine, shape: &VfxShape) -> Result<MeshHandle, String> {
    use og_render::geometry::primitives as p;
    if let VfxShape::Ring(Some(r)) = shape {
        // A ring of its own shape, made once per shape.
        let name = format!("vfx:ring {r:?}");
        let mut rs = eng.render.borrow_mut();
        if let Some(&h) = rs.fx.particle_meshes.get(&name) {
            return Ok(h);
        }
        let h = rs.scene.add_mesh(og_render::effect_ring(r));
        rs.fx.particle_meshes.insert(name, h);
        return Ok(h);
    }
    let (name, make): (&'static str, fn() -> og_render::MeshData) = match shape {
        VfxShape::Sphere => ("vfx:sphere", || og_render::shapes::sphere(32, 20)),
        VfxShape::Quad => ("vfx:quad", || og_render::shapes::plane_xy(1.0, 1.0)),
        VfxShape::Ring(_) => ("vfx:ring", || {
            p::ring(0.5, 1.0, 64, 2, 0.0, std::f32::consts::TAU)
        }),
        VfxShape::Box => ("vfx:box", || og_render::shapes::cuboid(Vec3::ONE)),
        VfxShape::Cylinder => ("vfx:cylinder", og_render::effect_cylinder),
        VfxShape::Model(m) => {
            return super::fx::particle_mesh(eng, og_bundle::fx::ParticleMesh::Asset(m));
        }
    };
    let mut rs = eng.render.borrow_mut();
    if let Some(&h) = rs.fx.particle_meshes.get(name) {
        return Ok(h);
    }
    let h = rs.scene.add_mesh(make());
    rs.fx.particle_meshes.insert(name.to_string(), h);
    Ok(h)
}

/// The flash's material (og_render's glow), made once.
fn flash_material(eng: &super::engine::Engine) -> EffectMaterialHandle {
    let mut rs = eng.render.borrow_mut();
    if let Some(h) = rs.effects.flash {
        return h;
    }
    let h = rs.scene.add_effect_material(EffectMaterial::glow());
    rs.effects.flash = Some(h);
    h
}

fn link(eng: &super::engine::Engine, kind: &VfxKind) -> Result<Link, String> {
    Ok(match kind {
        VfxKind::Emitter { id, .. } => {
            let i = super::fx::load_emitter(eng, id)?;
            Link::Emitter(eng.render.borrow().fx.emitters[i])
        }
        VfxKind::Mesh { shape, effect, .. } => {
            let mesh = builtin_mesh(eng, shape)?;
            Link::Mesh(mesh, super::effect::load_effect(eng, effect)?)
        }
        VfxKind::Trail { effect, .. } | VfxKind::Beam { effect, .. } => Link::Effect(
            effect
                .as_deref()
                .map(|e| super::effect::load_effect(eng, e))
                .transpose()?,
        ),
        VfxKind::Flash { .. } => {
            Link::Mesh(builtin_mesh(eng, &VfxShape::Sphere)?, flash_material(eng))
        }
        VfxKind::Sound { id, .. } => {
            Link::Sound(super::assets::handle(super::assets::load_sound(eng, id)?) as f64)
        }
        VfxKind::Flipbook { id, .. } => {
            let info = FlipbookInfo::from_json(&super::feel::load_flipbook(eng, id)?)?;
            let w = f64::from(info.frame_width);
            let h = f64::from(info.frame_height).max(1.0);
            Link::Flipbook {
                sheet: info.sheet as f64,
                texture: info.texture as f64,
                aspect: (w / h) as f32,
                clip: info.clips.first().map_or(0.0, |c| c.duration),
            }
        }
        VfxKind::Light { .. } | VfxKind::Decal { .. } | VfxKind::Shake { .. } => Link::None,
    })
}

/// `og.load.vfx`: its load index, loading what it names once.
pub(crate) fn load(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let registry = eng.modules.vfx.0.clone();
    if let Some(i) = lock(&registry).iter().position(|l| l.id == id) {
        return Ok(i);
    }
    let text = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .vfx_text(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let vfx = og_bundle::vfx::parse_vfx(&text).map_err(|e| format!("{id}: {e}"))?;
    let links = vfx
        .elements
        .iter()
        .map(|e| link(eng, &e.kind))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{id}: {e}"))?;
    let mut r = lock(&registry);
    r.push(Arc::new(Loaded { id, vfx, links }));
    eng.render.borrow_mut().effects.vfx = Some(registry.clone());
    Ok(r.len() - 1)
}

fn loaded(engine: u32, asset: f64) -> Result<Arc<Loaded>, String> {
    let registry = super::engine::engine(engine).map(|e| e.modules.vfx.0.clone());
    index(asset as u64)
        .and_then(|i| lock(registry.as_ref().ok()?).get(i).cloned())
        .ok_or_else(|| format!("{asset} is not a vfx loaded in this world (og.load.vfx)"))
}

pub fn og_load_vfx(world: u32, path: String) -> Result<serde_json::Value, String> {
    super::engine::engine(world)
        .and_then(|eng| load(&eng, &path))
        .map(|i| json!(handle(i)))
}

pub fn og_vfx_info(world: u32, asset: f64) -> Result<crate::wire::VfxInfo, String> {
    (|| {
        super::engine::engine(world)?;
        let l = loaded(world, asset)?;
        let v = &l.vfx;
        Ok(super::wire::VfxInfo {
            asset: asset as u64,
            duration: v.duration,
            r#loop: v.looping,
            events: v.events.iter().map(|e| e.name.clone()).collect(),
            impact: v
                .events
                .iter()
                .find(|e| e.name == og_bundle::vfx::IMPACT)
                .map_or(-1.0, |e| e.time),
            elements: v
                .elements
                .iter()
                .map(|e| e.kind.name().to_string())
                .collect(),
        })
    })()
}

pub fn og_vfx_spawn(world: u32, desc: Option<String>) -> Result<Value, String> {
    (|| {
        let raw = super::parse(desc.as_deref())?;
        let d: super::wire::VfxSpawnDesc = super::wire::value(&raw, "desc")?;
        let given = |k: &str| raw.get(k).is_some_and(|v| !v.is_null());
        let asset = d.asset as f64;
        loaded(world, asset)?;
        let mut spawn = serde_json::Map::new();
        if given("position") {
            let p = d.position;
            spawn.insert("position".into(), json!({ "x": p.x, "y": p.y, "z": p.z }));
        }
        if given("rotation") {
            let q = d.rotation;
            spawn.insert(
                "quaternion".into(),
                json!({ "x": q.x, "y": q.y, "z": q.z, "w": q.w }),
            );
        }
        if let Some(parent) = d.parent {
            spawn.insert("parent".into(), json!(parent));
        }
        let e = super::api::og_world_spawn(world, Some(Value::Object(spawn).to_string()))?;
        let bits = u64::from_json(&e).map_err(|_| "world.spawn answered no entity")?;
        let seed = match d.seed {
            0 => (splitmix(bits) as u32).max(1),
            s => s,
        };
        let mut v = json!({
            "asset": asset,
            "seed": seed,
            "startTick": super::api::og_world_get_tick(world),
        });
        if given("scale") {
            v["scale"] = json!(d.scale);
        }
        if given("tint") {
            let t = &d.tint;
            v["tint"] = json!({ "r": t.r, "g": t.g, "b": t.b });
        }
        super::api::og_world_insert(world, bits as f64, COMPONENT.into(), Some(v.to_string()))?;
        Ok(e)
    })()
}

// ── shared arithmetic ───────────────────────────────────────────────────────

fn splitmix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// An element's random value, 0 to 1, fixed per instance.
fn random(seed: u32, element: usize) -> f32 {
    (splitmix(u64::from(seed) << 16 | element as u64) >> 40) as f32 / (1u64 << 24) as f32
}

/// The renderer id of an instance's element (a system, a trail, a mesh).
fn element_id(entity: u64, element: usize) -> u64 {
    splitmix(entity ^ ((element as u64 + 1) << 52)) & !(1 << 63)
}

fn ticks(seconds: f32, rate: f32) -> u64 {
    (f64::from(seconds) * f64::from(rate)).round().max(0.0) as u64
}

/// The tick an instance's current cycle started (a looping effect starts
/// over every duration), or None before it starts or after a non-looping
/// one ends.
fn cycle_start(v: &Vfx, start: u64, tick: u64, rate: f32) -> Option<u64> {
    if tick < start {
        return None;
    }
    let len = ticks(v.duration, rate).max(1);
    let since = tick - start;
    if v.looping {
        Some(start + since / len * len)
    } else {
        (since <= len).then_some(start)
    }
}

// ── the tick: events, shakes, sounds and the end ─────────────────────────────

struct Records {
    vfx: (SchemaId, Layout),
    event: (SchemaId, Layout),
    source: (SchemaId, Layout),
    transform: (SchemaId, Layout),
    parent: (SchemaId, Layout),
    sprite: (SchemaId, Layout),
    flipbook: (SchemaId, Layout),
    flip_event: (SchemaId, Layout),
}

fn layout(world: &World, name: &str, kind: RecordKind) -> og_core::Result<(SchemaId, Layout)> {
    let id = world.id_by_name(name)?;
    Ok((id, Layout::from_schema(world.schema(id)?, kind)))
}

/// A component the system spawns from a partial value: the IDL's layout,
/// whose defaults fill the rest (a schema read back has none).
fn spawned(world: &World, name: &str) -> og_core::Result<(SchemaId, Layout)> {
    let id = world.id_by_name(name)?;
    let decl = og_api::host::RECORDS.iter().find(|d| d.schema == name);
    match decl.and_then(|d| Layout::from_decl(d).ok()) {
        Some(l) => Ok((id, l)),
        None => layout(world, name, RecordKind::Component),
    }
}

/// Registers og.vfx on a world that names og.Vfx or og.VfxEvent
/// (`super::lazy`), after every system scheduled so far.
pub(crate) fn enable(world: &mut World, mount: super::engine::Mount<'_>) -> og_core::Result<()> {
    let engine_id = mount.id;
    if world.systems().contains(&SYSTEM) || world.id_by_name(COMPONENT).is_err() {
        return Ok(());
    }
    let Ok(event) = layout(world, EVENT, RecordKind::Event) else {
        // og.VfxEvent registers with og.Vfx (super::lazy's TOGETHER); its
        // partner's enable schedules the system.
        return Ok(());
    };
    let r = Records {
        vfx: layout(world, COMPONENT, RecordKind::Component)?,
        event,
        source: layout(world, "og.AudioSource", RecordKind::Component)?,
        transform: spawned(world, "og.Transform")?,
        parent: spawned(world, "og.Parent")?,
        sprite: spawned(world, "og.Sprite")?,
        flipbook: spawned(world, "og.Flipbook")?,
        flip_event: layout(world, "og.FlipbookEvent", RecordKind::Event)?,
    };
    let feel = super::engine::engine(engine_id)
        .ok()
        .and_then(|e| e.state.borrow().feel.clone())
        .map(|f| f.shared);
    let registry = mount.modules.vfx.0.clone();
    let names = [COMPONENT, "og.Parent"];
    world.add_system(
        SystemDesc {
            name: SYSTEM.into(),
            reads: names
                .iter()
                .map(|n| n.to_string())
                .chain(["og.FlipbookEvent".to_string()])
                .collect(),
            writes: vec![
                COMPONENT.into(),
                EVENT.into(),
                "og.AudioSource".into(),
                "og.Transform".into(),
                "og.Parent".into(),
                "og.Sprite".into(),
                "og.Flipbook".into(),
            ],
            queries: names
                .iter()
                .map(|n| QueryDesc {
                    read: vec![n.to_string()],
                    ..QueryDesc::default()
                })
                .collect(),
            ..Default::default()
        },
        move |ctx| system(ctx, &registry, feel.as_deref(), &r),
    )
}

fn system(
    ctx: &mut SystemContext<'_>,
    registry: &Mutex<Vec<Arc<Loaded>>>,
    feel: Option<&Mutex<super::feel::FeelShared>>,
    r: &Records,
) -> og_core::Result<()> {
    let time = ctx.time();
    let rate = if time.dt > 0.0 { 1.0 / time.dt } else { 60.0 };
    let tick = time.tick;
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    og_core::sort::sort_by_key(&mut rows, |e| e.index());
    // Every entity's og.Parent, for the flipbooks an effect spawned.
    let mut parents: Vec<(Entity, Entity)> = Vec::new();
    for a in ctx.query(1)? {
        for &c in a.entities() {
            let p: record::Parent = r
                .parent
                .1
                .read(&ctx.get_bytes(c, r.parent.0)?)
                .map_err(invalid)?;
            parents.push((c, Entity::from_bits(p.entity)));
        }
    }
    // The events of an effect's flipbooks (sent last tick by og.flipbook),
    // passed on as the effect's: a vfx2d's impact is the effect's impact.
    for ev in ctx.events_bytes(r.flip_event.0)? {
        let f: record::FlipbookEvent = r.flip_event.1.read(&ev).map_err(invalid)?;
        let child = Entity::from_bits(f.entity);
        let Some(&(_, e)) = parents.iter().find(|(c, _)| *c == child) else {
            continue;
        };
        if !rows.contains(&e) {
            continue;
        }
        let v: record::Vfx = r.vfx.1.read(&ctx.get_bytes(e, r.vfx.0)?).map_err(invalid)?;
        let start = u64::from(v.start_tick);
        if start == UNSET || tick < start {
            continue;
        }
        // The frame fired last tick, when og.flipbook ran.
        let at = (tick.saturating_sub(1).max(start) - start) as f64 / f64::from(rate);
        let value = json!({ "entity": e.to_bits(), "asset": v.asset, "name": f.name, "time": at });
        let b = r.event.1.encode(None, Some(&value)).map_err(invalid)?;
        ctx.send_bytes(r.event.0, &b)?;
    }
    let mut ended = Vec::new();
    // Flipbook sprites despawned this tick (despawns apply after the system).
    let mut gone: Vec<Entity> = Vec::new();
    for &e in &rows {
        let bytes = ctx.get_bytes(e, r.vfx.0)?;
        let mut v = r.vfx.1.decode(&bytes);
        let row = record::Vfx::from_json(&v).map_err(invalid)?;
        let start = u64::from(row.start_tick);
        let Some(l) = index(row.asset).and_then(|i| lock(registry).get(i).cloned()) else {
            continue;
        };
        if start == UNSET || tick < start {
            continue;
        }
        let fx = &l.vfx;
        let len = ticks(fx.duration, rate).max(1);
        // The ticks to run: those after the last run (a startTick written
        // again starts over), at most one cycle back.
        let ran = u64::from(row.ran);
        let from = if ran != UNSET && ran >= start {
            if ran >= tick {
                continue;
            }
            ran + 1
        } else {
            start
        };
        let from = from.max(tick.saturating_sub(len));
        let (seed, scale) = (row.seed, row.scale);
        // Whether a time lands on one of the ticks run (its tick in the
        // cycle; a looping effect's end is its next start).
        let due = |at: f32| -> bool {
            let k = ticks(at, rate).min(len);
            let k = if fx.looping { k % len } else { k };
            (from..=tick).any(|t| {
                let since = t - start;
                k == if fx.looping { since % len } else { since }
            })
        };
        for ev in &fx.events {
            if due(ev.time) {
                let value = json!({ "entity": e.to_bits(), "asset": row.asset, "name": ev.name, "time": ev.time });
                let b = r.event.1.encode(None, Some(&value)).map_err(invalid)?;
                ctx.send_bytes(r.event.0, &b)?;
            }
        }
        for (i, el) in fx.elements.iter().enumerate() {
            // A flipbook's sprite goes when its clip (or its life) is over.
            if let Link::Flipbook { sheet, clip, .. } = &l.links[i] {
                let life = if el.life > 0.0 { el.life } else { *clip };
                if due(el.at + life) {
                    for &(c, p) in &parents {
                        if p == e
                            && !gone.contains(&c)
                            && ctx.get_bytes(c, r.flipbook.0).is_ok_and(|b| {
                                r.flipbook
                                    .1
                                    .read::<record::Flipbook>(&b)
                                    .is_ok_and(|fb| fb.sheet as f64 == *sheet)
                            })
                        {
                            ctx.despawn(c)?;
                            gone.push(c);
                        }
                    }
                }
            }
            if !due(el.at) {
                continue;
            }
            match (&el.kind, &l.links[i]) {
                (VfxKind::Shake { trauma }, _) => {
                    if let Some(f) = feel {
                        lock(f)
                            .camera_ops
                            .push(super::feel::CameraOp::Shake(*trauma));
                    }
                }
                (VfxKind::Sound { volume, pitch, .. }, Link::Sound(sound)) => {
                    let p = pitch[0] + (pitch[1] - pitch[0]) * random(seed, i);
                    let o = el.offset * scale;
                    let src = json!({
                        "sound": sound, "volume": volume, "playbackRate": p,
                        "playing": true, "spatial": true,
                    });
                    let at = json!({ "position": { "x": o.x, "y": o.y, "z": o.z } });
                    ctx.spawn_bytes(vec![
                        (
                            r.source.0,
                            r.source.1.encode(None, Some(&src)).map_err(invalid)?,
                        ),
                        (
                            r.transform.0,
                            r.transform.1.encode(None, Some(&at)).map_err(invalid)?,
                        ),
                        (
                            r.parent.0,
                            r.parent
                                .1
                                .encode(None, Some(&json!({ "entity": e.to_bits() })))
                                .map_err(invalid)?,
                        ),
                    ])?;
                }
                (
                    VfxKind::Flipbook {
                        size, color, blend, ..
                    },
                    Link::Flipbook {
                        sheet,
                        texture,
                        aspect,
                        ..
                    },
                ) => {
                    let o = el.offset * scale;
                    let tint = Vec3::new(row.tint.r, row.tint.g, row.tint.b);
                    let sprite = json!({
                        "texture": texture,
                        "width": size * aspect * scale,
                        "height": size * scale,
                        "facing": "camera",
                        "color": { "r": color[0] * tint.x, "g": color[1] * tint.y, "b": color[2] * tint.z },
                        "opacity": color[3],
                        "blend": if *blend == ParticleBlend::Additive { "additive" } else { "alpha" },
                    });
                    let at = json!({ "position": { "x": o.x, "y": o.y, "z": o.z } });
                    ctx.spawn_bytes(vec![
                        (
                            r.sprite.0,
                            r.sprite.1.encode(None, Some(&sprite)).map_err(invalid)?,
                        ),
                        (
                            r.flipbook.0,
                            r.flipbook
                                .1
                                .encode(None, Some(&json!({ "sheet": sheet, "clip": 0 })))
                                .map_err(invalid)?,
                        ),
                        (
                            r.transform.0,
                            r.transform.1.encode(None, Some(&at)).map_err(invalid)?,
                        ),
                        (
                            r.parent.0,
                            r.parent
                                .1
                                .encode(None, Some(&json!({ "entity": e.to_bits() })))
                                .map_err(invalid)?,
                        ),
                    ])?;
                }
                _ => {}
            }
        }
        v["ran"] = json!(tick);
        let out = r.vfx.1.encode(Some(&bytes), Some(&v)).map_err(invalid)?;
        ctx.set_bytes(e, r.vfx.0, &out)?;
        if !fx.looping && tick - start >= len && row.despawn {
            ended.push(e);
        }
    }
    if ended.is_empty() {
        return Ok(());
    }
    // The ended effects and their sounds and flipbooks (children by og.Parent).
    for (c, parent) in parents {
        if ended.contains(&parent)
            && (ctx.get_bytes(c, r.source.0).is_ok() || ctx.get_bytes(c, r.flipbook.0).is_ok())
            && !gone.contains(&c)
        {
            ctx.despawn(c)?;
        }
    }
    for e in ended {
        ctx.despawn(e)?;
    }
    Ok(())
}

// ── extraction ──────────────────────────────────────────────────────────────

fn tinted(c: [f32; 4], t: Vec3) -> [f32; 4] {
    [c[0] * t.x, c[1] * t.y, c[2] * t.z, c[3]]
}

fn euler(deg: Vec3) -> Quat {
    let r = deg * (std::f32::consts::PI / 180.0);
    use og_core::math::quat_from_axis_angle as turn;
    turn(Vec3::X, r.x) * turn(Vec3::Y, r.y) * turn(Vec3::Z, r.z)
}

/// Every playing og.Vfx expanded into the scene's systems, meshes, trails,
/// beams, lights and decals for the frame at `tick`.
pub(crate) fn extract(world: &World, rs: &mut RenderState, h: &mut Hierarchy, tick: u64) {
    let Some(registry) = rs.effects.vfx.clone() else {
        return;
    };
    let rows = super::fx::rows(world, rs.draw.fx.vfx.as_ref());
    if rows.is_empty() {
        return;
    }
    let loaded = lock(&registry).clone();
    let rate = rs.scene.frame.tick_rate.max(1.0);
    let mut air: Option<og_core::air::Air> = None;
    for (e, v) in rows {
        let Ok(row) = record::Vfx::from_json(&v) else {
            continue;
        };
        let Some(l) = index(row.asset).and_then(|i| loaded.get(i)) else {
            continue;
        };
        let start = u64::from(row.start_tick);
        if start == UNSET {
            continue;
        }
        let fx = &l.vfx;
        let Some(cycle) = cycle_start(fx, start, tick, rate) else {
            continue;
        };
        let seed = row.seed;
        let scale = row.scale.max(0.0);
        let tint = Vec3::new(row.tint.r, row.tint.g, row.tint.b);
        let m = h.world_of(e) * Mat4::from_scale(Vec3::splat(scale));
        let t = (tick - cycle) as f32 / rate;
        for (i, (el, link)) in fx.elements.iter().zip(&l.links).enumerate() {
            let te = t - el.at;
            if te < 0.0 {
                continue;
            }
            let local = m * el.local_at(te);
            let life = el.life.max(1e-4);
            let x = (te / life).clamp(0.0, 1.0);
            let rnd = random(seed, i);
            let id = element_id(e, i);
            let alive = te <= el.life;
            match (&el.kind, link) {
                (VfxKind::Emitter { seed: s, .. }, Link::Emitter(handle)) => {
                    let begin = cycle + ticks(el.at, rate);
                    let emitter = tinted_emitter(rs, *handle, tint);
                    // The frame's air, read once for the first emitter it carries.
                    let air = air.get_or_insert_with(|| rs.draw.atmo.air(world, h));
                    rs.scene.effects.particles.push(ParticleSystem {
                        stop_tick: (el.life > 0.0).then(|| begin + ticks(el.life, rate).max(1)),
                        start_tick: begin,
                        air: air.at(local.w_axis.truncate()),
                        ..ParticleSystem::new(
                            id,
                            emitter,
                            local,
                            u64::from(seed.wrapping_add(*s)),
                            tick,
                        )
                    });
                }
                (
                    VfxKind::Mesh {
                        scale: size,
                        spin,
                        color,
                        blend,
                        ..
                    },
                    Link::Mesh(mesh, material),
                ) if alive => {
                    rs.scene.effects.effect_meshes.push(EffectMesh {
                        id,
                        mesh: *mesh,
                        material: *material,
                        transform: local
                            * Mat4::from_quat(euler(*spin * te))
                            * Mat4::from_scale(Vec3::splat(size.eval(x, rnd))),
                        life: x,
                        seed: u64::from(seed) ^ i as u64,
                        color: tinted(*color, tint),
                        blend: *blend,
                        layers: 1,
                    });
                }
                (VfxKind::Flash { size, color }, Link::Mesh(mesh, material)) if alive => {
                    rs.scene.effects.effect_meshes.push(EffectMesh {
                        id,
                        mesh: *mesh,
                        material: *material,
                        transform: local * Mat4::from_scale(Vec3::splat(size.eval(x, rnd))),
                        life: x,
                        seed: u64::from(seed) ^ i as u64,
                        color: tinted(*color, tint),
                        blend: ParticleBlend::Additive,
                        layers: 1,
                    });
                }
                (
                    VfxKind::Trail {
                        width,
                        lifetime,
                        color,
                        orbit,
                        rise,
                        ..
                    },
                    Link::Effect(effect),
                ) if te <= el.life + lifetime => {
                    let tc = te.min(el.life);
                    let a = orbit[1] * tc * std::f32::consts::TAU;
                    let (sa, ca) = og_core::math::sin_cos(a);
                    let p = Vec3::new(orbit[0] * ca, rise * tc, orbit[0] * sa);
                    let style = TrailStyle {
                        lifetime: *lifetime,
                        min_distance: 0.02 * scale,
                        width: Curve::linear(width[0] * scale, width[1] * scale),
                        color: ColorCurve::constant(tinted(*color, tint)),
                        uv: RibbonUv::Stretch,
                        blend: ParticleBlend::Additive,
                        effect: *effect,
                        ..Default::default()
                    };
                    let key = super::fx::style_key(&style);
                    let handle = match rs.fx.trail_styles.get(&key) {
                        Some(h) => *h,
                        None => {
                            let h = rs.scene.add_trail_style(style);
                            rs.fx.trail_styles.insert(key, h);
                            h
                        }
                    };
                    rs.scene.effects.trails.push(Trail {
                        id,
                        style: handle,
                        position: local.transform_point3(p),
                        tick,
                    });
                }
                (
                    VfxKind::Beam {
                        to,
                        width,
                        color,
                        segments,
                        jitter,
                        ..
                    },
                    Link::Effect(effect),
                ) if alive => {
                    rs.scene.effects.beams.push(Beam {
                        start: local.transform_point3(Vec3::ZERO),
                        end: local.transform_point3(*to),
                        width: width * scale,
                        color: tinted(*color, tint),
                        segments: *segments,
                        jitter: jitter * scale,
                        seed: u64::from(seed) ^ i as u64,
                        blend: ParticleBlend::Additive,
                        effect: *effect,
                        ..Default::default()
                    });
                }
                (
                    VfxKind::Light {
                        color,
                        intensity,
                        distance,
                    },
                    _,
                ) if alive => {
                    rs.scene.lighting.lights.push(Light {
                        kind: LightKind::Point,
                        position: local.transform_point3(Vec3::ZERO),
                        color: Vec3::from_array(*color) * tint,
                        intensity: intensity.eval(x, rnd).max(0.0),
                        range: distance * scale,
                        cast_shadows: false,
                    });
                }
                (
                    VfxKind::Decal {
                        uv,
                        size,
                        fade,
                        color,
                    },
                    _,
                ) if alive => {
                    let a = if *fade > 0.0 {
                        ((el.life - te) / fade).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let c = tinted(*color, tint);
                    // Its unit box projects along its -z: turned down the
                    // element's -y, y the depth.
                    rs.scene.effects.decals.push(Decal {
                        transform: local
                            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                            * Mat4::from_scale(Vec3::new(size.x, size.z, size.y)),
                        uv_rect: *uv,
                        color: [c[0], c[1], c[2], c[3] * a],
                        ..Default::default()
                    });
                }
                _ => {}
            }
        }
    }
}

/// `handle` drawn with its colour times `tint`: a copy of the emitter per
/// tint, made once.
fn tinted_emitter(rs: &mut RenderState, handle: EmitterHandle, tint: Vec3) -> EmitterHandle {
    if tint == Vec3::ONE {
        return handle;
    }
    let key = (handle, tint.to_array().map(f32::to_bits));
    if let Some(&h) = rs.effects.tinted.get(&key) {
        return h;
    }
    let Some(mut e) = rs.scene.emitter(handle).cloned() else {
        return handle;
    };
    e.color = tinted(e.color, tint);
    let h = rs.scene.add_emitter(e);
    rs.effects.tinted.insert(key, h);
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vfx(looping: bool) -> Vfx {
        Vfx {
            duration: 1.0,
            looping,
            events: Vec::new(),
            elements: Vec::new(),
        }
    }

    #[test]
    fn a_cycle_starts_at_the_start_and_over_at_each_duration_when_looping() {
        // 60 ticks a second: a 1 s effect started at tick 100.
        assert_eq!(cycle_start(&vfx(false), 100, 99, 60.0), None);
        assert_eq!(cycle_start(&vfx(false), 100, 160, 60.0), Some(100));
        assert_eq!(cycle_start(&vfx(false), 100, 161, 60.0), None);
        assert_eq!(cycle_start(&vfx(true), 100, 161, 60.0), Some(160));
        assert_eq!(cycle_start(&vfx(true), 100, 280, 60.0), Some(280));
    }

    #[test]
    fn instances_differ_by_seed_and_elements_by_index() {
        assert_ne!(random(1, 0), random(2, 0));
        assert_ne!(random(1, 0), random(1, 1));
        assert!((0.0..1.0).contains(&random(7, 3)));
        assert_ne!(element_id(5, 0), element_id(5, 1));
        assert_eq!(element_id(5, 0) >> 63, 0);
    }
}
