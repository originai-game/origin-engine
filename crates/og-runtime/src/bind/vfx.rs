//! Composed effects (`file.vfx`, og_bundle::vfx) in the tick: `og.load.vfx`,
//! which links an effect's elements to the handles of what they name, and
//! the `og.vfx` system, which runs each playing og.Vfx's gameplay side
//! (og.VfxEvent, shakes, sounds, flipbook sprites, the despawn at the end).
//! What an instance cues at a tick is og_bundle::vfx's answer
//! (`Vfx::cues`); what it draws is extracted by `crate::bind::vfx`.
//!
//! og.Vfx and og.VfxEvent are lazy (`crate::engine::lazy`): a world that never names
//! them keeps its type table and hash, and has no og.vfx system.

use crate::host::codec::{Layout, Rec, RecordKind, Row, Slot};
use crate::host::wire::{FlipbookInfo, FromJson, record};
use glam::Vec3;
use og_bundle::vfx::{Vfx, VfxCue, VfxKind, VfxShape};
use og_core::{Error, ErrorKind, IdMap};
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

pub(crate) fn handle(index: usize) -> f64 {
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
    /// `(emitter load index in FxState::emitters, renderer handle)`.
    Emitter(usize, EmitterHandle),
    Mesh(MeshHandle, EffectMaterialHandle),
    Effect(Option<EffectMaterialHandle>),
    /// A sound's handle (og.AudioSource.sound).
    Sound(f64),
    /// A decal's image and normal map.
    Decal {
        texture: Option<og_render::TextureHandle>,
        normal: Option<og_render::TextureHandle>,
    },
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
    /// Source bundle index for safe bundle release checks.
    pub bundle: Option<usize>,
    pub vfx: Vfx,
    /// Per element, in file order.
    pub links: Vec<Link>,
    /// Per element, a flipbook's first clip's length in seconds (0 for
    /// every other element): what ends one without a `life`.
    pub clips: Vec<f32>,
    /// Whether any element is a particle emitter (fast early-out in `before_tick`).
    pub has_emitters: bool,
    /// Whether any element builds simulation history (`Emitter` or `Trail`).
    pub has_sim_elements: bool,
    /// Whether any element is a single-frame visual (`Mesh`, `Glow`, `Beam`, `Light`, `Decal`).
    pub has_visual_elements: bool,
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

/// The glow element's material (og_render's glow), made once.
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
            Link::Emitter(i, eng.render.borrow().fx.emitters[i])
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
        VfxKind::Glow { .. } => {
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
        VfxKind::Decal {
            texture, normal, ..
        } => {
            let load = |id: &Option<String>| -> Result<_, String> {
                let Some(id) = id else { return Ok(None) };
                let i = crate::world::assets::load_texture(eng, id)?;
                Ok(eng.render.borrow().textures.get(i).copied())
            };
            Link::Decal {
                texture: load(texture)?,
                normal: load(normal)?,
            }
        }
        VfxKind::Light { .. } | VfxKind::Shake { .. } => Link::None,
    })
}

/// `og.load.vfx`: its load index, loading what it names once.
pub(crate) fn load(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    let registry = eng.modules.vfx.0.clone();
    if let Some(i) = lock(&registry)
        .iter()
        .position(|l| !l.id.is_empty() && l.id == id)
    {
        return Ok(i);
    }
    let (vfx, owner) = {
        let assets = eng.assets.borrow();
        let owner = assets.source(&id, None).ok();
        let (bytes, bundle) = assets.blob(&id)?;
        let vfx = bundle.vfx(&id, bytes.bytes()).map_err(|e| e.to_string())?;
        (vfx, owner)
    };
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
    let has_emitters = vfx
        .elements
        .iter()
        .any(|e| matches!(e.kind, VfxKind::Emitter { .. }));
    let has_sim_elements = vfx
        .elements
        .iter()
        .any(|e| matches!(e.kind, VfxKind::Emitter { .. } | VfxKind::Trail { .. }));
    let has_visual_elements = vfx.elements.iter().any(|e| {
        matches!(
            e.kind,
            VfxKind::Mesh { .. }
                | VfxKind::Glow { .. }
                | VfxKind::Beam { .. }
                | VfxKind::Light { .. }
                | VfxKind::Decal { .. }
        )
    });
    let mut r = lock(&registry);
    r.push(Arc::new(Loaded {
        id,
        bundle: owner,
        vfx,
        links,
        clips,
        has_emitters,
        has_sim_elements,
        has_visual_elements,
    }));
    eng.render.borrow_mut().effects.vfx = Some(registry.clone());
    Ok(r.len() - 1)
}

/// Composed effect `path` loaded, as a timeline's `vfx` key plays it: its
/// handle, length in seconds and whether it loops.
pub(crate) fn load_timed(
    eng: &crate::engine::Engine,
    path: &str,
) -> Result<(f64, f32, bool), String> {
    let i = load(eng, path)?;
    let l = lock(&eng.modules.vfx.0)[i].clone();
    Ok((handle(i), l.vfx.duration, l.vfx.looping))
}

fn loaded(engine: u32, asset: f64) -> Result<Arc<Loaded>, String> {
    let registry = crate::engine::engine(engine).map(|e| e.modules.vfx.0.clone());
    index(asset as u64)
        .and_then(|i| lock(registry.as_ref().ok()?).get(i).cloned())
        .ok_or_else(|| format!("{asset} is not a vfx loaded in this world (og.load.vfx)"))
}

pub fn og_load_vfx(world: u32, path: String) -> Result<serde_json::Value, String> {
    crate::engine::engine(world).and_then(|eng| {
        let i = load(&eng, &path)?;
        // Outside a system, og.Vfx registers (og.vfx with it), so what
        // plays it for the game (og.destruction's breakVfx) may write it.
        if let Ok(w) = eng.world() {
            crate::engine::lazy::ensure(&eng, w, COMPONENT)?;
        }
        Ok(json!(handle(i)))
    })
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
            params: v.params.iter().map(|p| p.name.clone()).collect(),
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
        if d.params.len() > og_bundle::vfx::MAX_PARAMS {
            return Err(format!(
                "desc.params has {} values; an effect has at most {} params",
                d.params.len(),
                og_bundle::vfx::MAX_PARAMS
            ));
        }
        let mut spawn = serde_json::Map::new();
        if given("position") {
            let p = d.position;
            spawn.insert("position".into(), json!({ "x": p.x, "y": p.y, "z": p.z }));
        }
        if given("quaternion") {
            let q = d.quaternion;
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
            "startTick": crate::host::api::og_world_get_tick(world) + delay_ticks(world, d.delay),
        });
        if given("scale") {
            v["scale"] = json!(d.scale);
        }
        if given("tint") {
            let t = &d.tint;
            v["tint"] = json!({ "r": t.r, "g": t.g, "b": t.b });
        }
        if given("emission") {
            v["emission"] = json!(d.emission);
        }
        if given("hue") {
            v["hue"] = json!(d.hue);
        }
        if given("params") {
            v["params"] = json!(d.params);
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

struct VfxSlots {
    asset: Slot,
    seed: Slot,
    start_tick: Slot,
    scale: Slot,
    tint: Slot,
    hue: Slot,
    params: Slot,
    despawn: Slot,
    ran: Slot,
}

struct Records {
    vfx: (SchemaId, Layout, VfxSlots),
    event: (SchemaId, Layout),
    source: (SchemaId, Layout, Slot, Slot),
    transform: (SchemaId, Layout),
    parent: (SchemaId, Layout, Slot),
    sprite: (SchemaId, Layout),
    flipbook: (SchemaId, Layout, Slot),
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
    let (vfx_id, vfx_l) = layout(world, COMPONENT, RecordKind::Component)?;
    let vfx_s = VfxSlots {
        asset: Slot::of(&vfx_l, "asset"),
        seed: Slot::of(&vfx_l, "seed"),
        start_tick: Slot::of(&vfx_l, "startTick"),
        scale: Slot::of(&vfx_l, "scale"),
        tint: Slot::of(&vfx_l, "tint"),
        hue: Slot::of(&vfx_l, "hue"),
        params: Slot::of(&vfx_l, "params"),
        despawn: Slot::of(&vfx_l, "despawn"),
        ran: Slot::of(&vfx_l, "ran"),
    };
    let (src_id, src_l) = spawned(world, "og.AudioSource")?;
    let src_sound = Slot::of(&src_l, "sound");
    let src_playing = Slot::of(&src_l, "playing");
    let (par_id, par_l) = spawned(world, "og.Parent")?;
    let par_entity = Slot::of(&par_l, "entity");
    let (fb_id, fb_l) = spawned(world, "og.Flipbook")?;
    let fb_sheet = Slot::of(&fb_l, "sheet");
    let r = Records {
        vfx: (vfx_id, vfx_l, vfx_s),
        event,
        source: (src_id, src_l, src_sound, src_playing),
        transform: spawned(world, "og.Transform")?,
        parent: (par_id, par_l, par_entity),
        sprite: spawned(world, "og.Sprite")?,
        flipbook: (fb_id, fb_l, fb_sheet),
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
    )?;
    // og.destruction plays an og.Destructible's breakVfx and hitVfx.
    if world.systems().contains(&crate::bind::destruction::SYSTEM) {
        world.grant(crate::bind::destruction::SYSTEM, &[COMPONENT])?;
    }
    Ok(())
}

/// og.Vfx's IDL layout, whose defaults fill a partial value (og.destruction's
/// breakVfx and hitVfx).
pub(crate) fn layout_decl() -> Option<&'static Layout> {
    static L: std::sync::OnceLock<Option<Layout>> = std::sync::OnceLock::new();
    L.get_or_init(|| {
        og_api::host::RECORDS
            .iter()
            .find(|d| d.schema == COMPONENT)
            .and_then(|d| Layout::from_decl(d).ok())
    })
    .as_ref()
}

fn system(
    ctx: &mut SystemContext<'_>,
    registry: &Mutex<Vec<Arc<Loaded>>>,
    feel: Option<&Mutex<crate::present::feel::FeelShared>>,
    r: &Records,
) -> og_core::Result<()> {
    let mut rows = Vec::new();
    for a in ctx.query(0)? {
        rows.extend_from_slice(a.entities());
    }
    if rows.is_empty() {
        return Ok(());
    }
    let time = ctx.time();
    let rate = time.rate as f32;
    let tick = time.tick;
    og_core::sort::sort_by_key(&mut rows, |e| e.index());

    // Index only children whose og.Parent is a live og.Vfx entity (O(P + V)
    // instead of scanning every parented entity per VFX / flipbook event).
    let mut children_of: IdMap<u64, Vec<Entity>> = IdMap::new();
    let mut parent_of: IdMap<u64, Entity> = IdMap::new();
    for a in ctx.query(1)? {
        for &c in a.entities() {
            let bytes = ctx.get_bytes(c, r.parent.0)?;
            let parent_bits = Row::new(&r.parent.1, &bytes).uint(&r.parent.2).unwrap_or(0);
            if parent_bits == 0 {
                continue;
            }
            let parent = Entity::from_bits(parent_bits);
            if rows
                .binary_search_by_key(&parent.index(), |x| x.index())
                .is_ok_and(|k| rows[k] == parent)
            {
                children_of.entry(parent_bits).or_default().push(c);
                parent_of.insert(c.to_bits(), parent);
            }
        }
    }

    let (vfx_id, vfx_l, vfx_s) = &r.vfx;
    // The events of an effect's flipbooks (sent last tick by og.flipbook),
    // passed on as the effect's: a vfx2d's impact is the effect's impact.
    for ev in ctx.events_bytes(r.flip_event.0)? {
        let f: record::FlipbookEvent = r.flip_event.1.read(&ev).map_err(invalid)?;
        let Some(&e) = parent_of.get(&f.entity) else {
            continue;
        };
        let bytes = ctx.get_bytes(e, *vfx_id)?;
        let row = Row::new(vfx_l, &bytes);
        let start = row.uint(&vfx_s.start_tick).unwrap_or(UNSET);
        if start == UNSET || tick < start {
            continue;
        }
        let asset = row.uint(&vfx_s.asset).unwrap_or(0);
        // The frame fired last tick, when og.flipbook ran.
        let at = time.seconds(tick.saturating_sub(1).max(start) - start);
        let value = json!({ "entity": e.to_bits(), "asset": asset, "name": f.name, "time": at });
        let b = r.event.1.encode(None, Some(&value)).map_err(invalid)?;
        ctx.send_bytes(r.event.0, &b)?;
    }

    let loaded = lock(registry).clone();
    let mut ended = Vec::new();
    // Child entities despawned this tick (despawns apply after the system).
    let mut gone: Vec<Entity> = Vec::new();
    let mut params_buf = Vec::new();
    for &e in &rows {
        let bytes = ctx.get_bytes(e, *vfx_id)?;
        let row = Row::new(vfx_l, &bytes);
        let asset = row.uint(&vfx_s.asset).unwrap_or(0);
        let start = row.uint(&vfx_s.start_tick).unwrap_or(UNSET);
        let Some(l) = index(asset).and_then(|i| loaded.get(i)) else {
            continue;
        };
        // Reclaim finished child AudioSource entities spawned by this VFX so
        // looping or non-despawning VFX never leak one-shot sound entities.
        if let Some(kids) = children_of.get(&e.to_bits()) {
            for &c in kids {
                if gone.contains(&c) {
                    continue;
                }
                if let Ok(sb) = ctx.get_bytes(c, r.source.0) {
                    let srow = Row::new(&r.source.1, &sb);
                    let playing = srow.flag(&r.source.3).unwrap_or(false);
                    if !playing {
                        let sound = srow.uint(&r.source.2).unwrap_or(0) as f64;
                        let owned = l
                            .links
                            .iter()
                            .any(|lk| matches!(lk, Link::Sound(s) if *s == sound));
                        if owned {
                            ctx.despawn(c)?;
                            gone.push(c);
                        }
                    }
                }
            }
        }
        if start == UNSET || tick < start {
            continue;
        }
        let fx = &l.vfx;
        // The ticks to run: those after the last run (a startTick written
        // again starts over), at most one cycle back.
        let ran = row.uint(&vfx_s.ran).unwrap_or(UNSET);
        let Some(from) = fx.run_from(start, (ran != UNSET).then_some(ran), tick, rate) else {
            continue;
        };
        let seed = row.uint(&vfx_s.seed).map_or(0, |x| x as u32);
        let scale = row.num(&vfx_s.scale).map_or(1.0, |x| x as f32);
        let tint = Vec3::new(
            row.lane(&vfx_s.tint, "r").map_or(1.0, |x| x as f32),
            row.lane(&vfx_s.tint, "g").map_or(1.0, |x| x as f32),
            row.lane(&vfx_s.tint, "b").map_or(1.0, |x| x as f32),
        );
        let hue = row.num(&vfx_s.hue).map_or(0.0, |x| x as f32);
        let hue = if hue.is_finite() { hue } else { 0.0 };
        let despawn = row.flag(&vfx_s.despawn).unwrap_or(true);
        row.list_f32(&vfx_s.params, &mut params_buf);
        let values = fx.values(&params_buf);
        for cue in fx.cues(seed, &values, start, from..=tick, rate, &l.clips) {
            match cue {
                VfxCue::Event(ev) => {
                    let value = json!({ "entity": e.to_bits(), "asset": asset, "name": ev.name, "time": ev.time });
                    let b = r.event.1.encode(None, Some(&value)).map_err(invalid)?;
                    ctx.send_bytes(r.event.0, &b)?;
                }
                // A flipbook's sprite goes when its clip (or its life) is over.
                VfxCue::FlipbookEnd { element } => {
                    let Link::Flipbook { sheet, .. } = &l.links[element] else {
                        continue;
                    };
                    if let Some(kids) = children_of.get(&e.to_bits()) {
                        for &c in kids {
                            if !gone.contains(&c)
                                && ctx.get_bytes(c, r.flipbook.0).is_ok_and(|b| {
                                    Row::new(&r.flipbook.1, &b)
                                        .uint(&r.flipbook.2)
                                        .is_some_and(|sh| sh as f64 == *sheet)
                                })
                            {
                                ctx.despawn(c)?;
                                gone.push(c);
                            }
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
                    let volume = volume * fx.amount(element, &values);
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
                    // Its colour tinted and turned as the drawn elements' are.
                    let c = og_bundle::vfx::hue_rotated(
                        Vec3::new(color[0], color[1], color[2]) * tint,
                        hue,
                    );
                    let size = size * scale * fx.amount(element, &values);
                    let sprite = json!({
                        "texture": texture,
                        "width": size * aspect,
                        "height": size,
                        "facing": "camera",
                        "color": { "r": c.x, "g": c.y, "b": c.z },
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
        // Patch `ran` in-place in packed bytes (avoids full JSON decode + re-encode).
        let mut out = bytes;
        if let Some(at) = vfx_s.ran.offset()
            && at + 4 <= out.len()
        {
            out[at..at + 4].copy_from_slice(&(tick as u32).to_le_bytes());
        } else {
            let mut v = vfx_l.decode(&out);
            v["ran"] = json!(tick);
            out = vfx_l.encode(Some(&out), Some(&v)).map_err(invalid)?;
        }
        ctx.set_bytes(e, *vfx_id, &out)?;
        if fx.ended(start, tick, rate) {
            if despawn {
                ended.push(e);
            } else if let Some(kids) = children_of.get(&e.to_bits()) {
                // Clean up remaining flipbook children for non-despawning VFX.
                for &c in kids {
                    if !gone.contains(&c) && ctx.get_bytes(c, r.flipbook.0).is_ok() {
                        ctx.despawn(c)?;
                        gone.push(c);
                    }
                }
            }
        }
    }
    if ended.is_empty() {
        return Ok(());
    }
    // The ended effects and their sounds and flipbooks (children by og.Parent).
    for e in ended {
        if let Some(kids) = children_of.get(&e.to_bits()) {
            for &c in kids {
                if !gone.contains(&c)
                    && (ctx.get_bytes(c, r.source.0).is_ok()
                        || ctx.get_bytes(c, r.flipbook.0).is_ok())
                {
                    ctx.despawn(c)?;
                    gone.push(c);
                }
            }
        }
        ctx.despawn(e)?;
    }
    Ok(())
}

/// VfxSpawnDesc.delay in whole ticks of the world's fixed rate, rounded up.
fn delay_ticks(world: u32, delay: f32) -> u32 {
    if delay <= 0.0 {
        return 0;
    }
    let tick = match crate::engine::running(world) {
        // SAFETY: the frame is published while its system runs.
        Some(f) => unsafe { f.ctx() }.time().seconds(1),
        None => crate::engine::engine(world)
            .ok()
            .and_then(|e| e.world().map(|w| w.time().seconds(1)).ok())
            .unwrap_or(1.0 / 60.0),
    };
    (f64::from(delay) / tick - 1e-9).ceil().max(0.0) as u32
}
