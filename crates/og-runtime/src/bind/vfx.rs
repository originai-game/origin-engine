//! Composed effects (`type vfx`, og_bundle::vfx) in the tick: `og.load.vfx`,
//! which links an effect's elements to the handles of what they name, and
//! the `og.vfx` system, which runs each playing og.Vfx's gameplay side
//! (og.VfxEvent, shakes, sounds, flipbook sprites, the despawn at the end).
//! What an instance cues at a tick is og_bundle::vfx's answer
//! (`Vfx::cues`); what it draws is extracted by `crate::bind::vfx`.
//!
//! og.Vfx and og.VfxEvent are lazy (`crate::engine::lazy`): a world that never names
//! them keeps its type table and hash, and has no og.vfx system.

use crate::host::codec::{Layout, RecordKind};
use crate::host::wire::{FlipbookInfo, FromJson, record};
use glam::Vec3;
use og_bundle::vfx::{Vfx, VfxCue, VfxKind, VfxShape};
use og_core::{Error, ErrorKind};
use og_ecs::{Entity, QueryDesc, SchemaId, SystemContext, SystemDesc, World};
use og_render::{EffectMaterial, EffectMaterialHandle, EmitterHandle, MeshHandle, ParticleBlend};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(crate) const COMPONENT: &str = "og.Vfx";
pub(crate) const EVENT: &str = "og.VfxEvent";
pub(crate) const RECORDS: [&str; 2] = [COMPONENT, EVENT];
const SYSTEM: &str = "og.vfx";
/// og.Vfx.startTick's and ran's unwritten value.
pub(crate) const UNSET: u64 = u32::MAX as u64;
/// VfxAsset handles: this tag, then the load index plus one.
const TAG: u64 = 0x7d << 32;

fn handle(index: usize) -> f64 {
    (TAG | (index as u64 + 1)) as f64
}

/// A VfxAsset handle's load index.
pub(crate) fn index(h: u64) -> Option<usize> {
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
    /// Per element, a flipbook's first clip's length in seconds (0 for
    /// every other element): what ends one without a `life`.
    pub clips: Vec<f32>,
}

/// The effects a world loaded, by load index; shared by the loads, the
/// og.vfx system and extraction.
pub(crate) type Registry = Arc<Mutex<Vec<Arc<Loaded>>>>;

/// og.vfx on one engine: the effects its world loaded.
#[derive(Clone, Default)]
pub(crate) struct ModuleState(Registry);

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn invalid(m: impl Into<String>) -> Error {
    Error::new(ErrorKind::Invalid, m.into())
}

// ── loading ─────────────────────────────────────────────────────────────────

/// A built-in effect mesh, made once.
fn builtin_mesh(eng: &crate::engine::Engine, shape: &VfxShape) -> Result<MeshHandle, String> {
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
            return crate::present::fx::particle_mesh(eng, og_bundle::fx::ParticleMesh::Asset(m));
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
fn flash_material(eng: &crate::engine::Engine) -> EffectMaterialHandle {
    let mut rs = eng.render.borrow_mut();
    if let Some(h) = rs.effects.flash {
        return h;
    }
    let h = rs.scene.add_effect_material(EffectMaterial::glow());
    rs.effects.flash = Some(h);
    h
}

fn link(eng: &crate::engine::Engine, kind: &VfxKind) -> Result<Link, String> {
    Ok(match kind {
        VfxKind::Emitter { id, .. } => {
            let i = crate::present::fx::load_emitter(eng, id)?;
            Link::Emitter(eng.render.borrow().fx.emitters[i])
        }
        VfxKind::Mesh { shape, effect, .. } => {
            let mesh = builtin_mesh(eng, shape)?;
            Link::Mesh(mesh, crate::present::effect::load_effect(eng, effect)?)
        }
        VfxKind::Trail { effect, .. } | VfxKind::Beam { effect, .. } => Link::Effect(
            effect
                .as_deref()
                .map(|e| crate::present::effect::load_effect(eng, e))
                .transpose()?,
        ),
        VfxKind::Flash { .. } => {
            Link::Mesh(builtin_mesh(eng, &VfxShape::Sphere)?, flash_material(eng))
        }
        VfxKind::Sound { id, .. } => Link::Sound(crate::world::assets::handle(
            crate::world::assets::load_sound(eng, id)?,
        ) as f64),
        VfxKind::Flipbook { id, .. } => {
            let info = FlipbookInfo::from_json(&crate::present::feel::load_flipbook(eng, id)?)?;
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
pub(crate) fn load(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
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
    let clips = links
        .iter()
        .map(|l| match l {
            Link::Flipbook { clip, .. } => *clip,
            _ => 0.0,
        })
        .collect();
    let mut r = lock(&registry);
    r.push(Arc::new(Loaded {
        id,
        vfx,
        links,
        clips,
    }));
    eng.render.borrow_mut().effects.vfx = Some(registry.clone());
    Ok(r.len() - 1)
}

fn loaded(engine: u32, asset: f64) -> Result<Arc<Loaded>, String> {
    let registry = crate::engine::engine(engine).map(|e| e.modules.vfx.0.clone());
    index(asset as u64)
        .and_then(|i| lock(registry.as_ref().ok()?).get(i).cloned())
        .ok_or_else(|| format!("{asset} is not a vfx loaded in this world (og.load.vfx)"))
}

pub fn og_load_vfx(world: u32, path: String) -> Result<serde_json::Value, String> {
    crate::engine::engine(world)
        .and_then(|eng| load(&eng, &path))
        .map(|i| json!(handle(i)))
}

pub fn og_vfx_info(world: u32, asset: f64) -> Result<crate::host::wire::VfxInfo, String> {
    (|| {
        crate::engine::engine(world)?;
        let l = loaded(world, asset)?;
        let v = &l.vfx;
        Ok(crate::host::wire::VfxInfo {
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
        let raw = crate::parse(desc.as_deref())?;
        let d: crate::host::wire::VfxSpawnDesc = crate::host::wire::value(&raw, "desc")?;
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
        let e = crate::host::api::og_world_spawn(world, Some(Value::Object(spawn).to_string()))?;
        let bits = u64::from_json(&e).map_err(|_| "world.spawn answered no entity")?;
        let seed = match d.seed {
            0 => (og_core::hash::splitmix64(bits) as u32).max(1),
            s => s,
        };
        let mut v = json!({
            "asset": asset,
            "seed": seed,
            "startTick": crate::host::api::og_world_get_tick(world),
        });
        if given("scale") {
            v["scale"] = json!(d.scale);
        }
        if given("tint") {
            let t = &d.tint;
            v["tint"] = json!({ "r": t.r, "g": t.g, "b": t.b });
        }
        crate::host::api::og_world_insert(
            world,
            bits as f64,
            COMPONENT.into(),
            Some(v.to_string()),
        )?;
        Ok(e)
    })()
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
/// (`crate::engine::lazy`), after every system scheduled so far.
pub(crate) fn enable(world: &mut World, mount: crate::engine::Mount<'_>) -> og_core::Result<()> {
    let engine_id = mount.id;
    if world.systems().contains(&SYSTEM) || world.id_by_name(COMPONENT).is_err() {
        return Ok(());
    }
    let Ok(event) = layout(world, EVENT, RecordKind::Event) else {
        // og.VfxEvent registers with og.Vfx (crate::engine::lazy's TOGETHER); its
        // partner's enable schedules the system.
        return Ok(());
    };
    let r = Records {
        vfx: layout(world, COMPONENT, RecordKind::Component)?,
        event,
        source: spawned(world, "og.AudioSource")?,
        transform: spawned(world, "og.Transform")?,
        parent: spawned(world, "og.Parent")?,
        sprite: spawned(world, "og.Sprite")?,
        flipbook: spawned(world, "og.Flipbook")?,
        flip_event: layout(world, "og.FlipbookEvent", RecordKind::Event)?,
    };
    let feel = crate::engine::engine(engine_id)
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
    feel: Option<&Mutex<crate::present::feel::FeelShared>>,
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
        // The ticks to run: those after the last run (a startTick written
        // again starts over), at most one cycle back.
        let ran = u64::from(row.ran);
        let Some(from) = fx.run_from(start, (ran != UNSET).then_some(ran), tick, rate) else {
            continue;
        };
        let scale = row.scale;
        for cue in fx.cues(row.seed, start, from, tick, rate, &l.clips) {
            match cue {
                VfxCue::Event(ev) => {
                    let value = json!({ "entity": e.to_bits(), "asset": row.asset, "name": ev.name, "time": ev.time });
                    let b = r.event.1.encode(None, Some(&value)).map_err(invalid)?;
                    ctx.send_bytes(r.event.0, &b)?;
                }
                // A flipbook's sprite goes when its clip (or its life) is over.
                VfxCue::FlipbookEnd { element } => {
                    let Link::Flipbook { sheet, .. } = &l.links[element] else {
                        continue;
                    };
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
                VfxCue::Shake { trauma } => {
                    if let Some(f) = feel {
                        lock(f)
                            .camera_ops
                            .push(crate::present::feel::CameraOp::Shake(trauma));
                    }
                }
                VfxCue::Sound {
                    element,
                    playback_rate,
                } => {
                    let el = &fx.elements[element];
                    let (VfxKind::Sound { volume, .. }, Link::Sound(sound)) =
                        (&el.kind, &l.links[element])
                    else {
                        continue;
                    };
                    let o = el.offset * scale;
                    let src = json!({
                        "sound": sound, "volume": volume, "playbackRate": playback_rate,
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
                VfxCue::Flipbook { element } => {
                    let el = &fx.elements[element];
                    let (
                        VfxKind::Flipbook {
                            size, color, blend, ..
                        },
                        Link::Flipbook {
                            sheet,
                            texture,
                            aspect,
                            ..
                        },
                    ) = (&el.kind, &l.links[element])
                    else {
                        continue;
                    };
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
            }
        }
        v["ran"] = json!(tick);
        let out = r.vfx.1.encode(Some(&bytes), Some(&v)).map_err(invalid)?;
        ctx.set_bytes(e, r.vfx.0, &out)?;
        if fx.ended(start, tick, rate) && row.despawn {
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
