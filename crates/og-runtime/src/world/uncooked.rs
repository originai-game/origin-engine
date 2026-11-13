//! Uncooked loads at run time (IDL `load`: `imageData`, `gltfData`,
//! `environmentData`, `imageInfo`, `environmentInfo`, `completions`,
//! `replayCompletions`): content a player or a
//! generator made, from bytes, without `og cook`.
//!
//! The source is untrusted, so the budget (`LoadBudget`) is checked from its
//! headers before anything is decoded. Decoding never runs on the engine's
//! thread on the web: the host decodes (the Worker's `createImageBitmap`,
//! [`Hooks::image_decode_start`]); natively a worker thread does, with the
//! `image` crate. No decoder is in the wasm. A glTF is read at the call
//! (og-bundle's [`og_bundle::runtime::UncookedGltf`]: its JSON, buffers,
//! triangle count and image headers), then its images decode as above while
//! its primitives are converted at the boundaries, one at a time within the
//! frame budget on the web, and it joins the renderer through the path a
//! cooked model takes. An HDR environment (a Radiance `.hdr`, natively an
//! OpenEXR `.exr` too) decodes on a worker thread natively and in rows at
//! the boundaries on the web (og-bundle's [`og_bundle::runtime::HdrRows`]),
//! its texels packed RGB9E5 and its irradiance gathered as it goes; the
//! renderer prefilters it on the GPU when it first draws with it.
//!
//! Completion is host input. A finished load is put in place at a tick
//! boundary and its tick journaled ([`Uncooked::journal`]), as a replay
//! journals key presses: natively at the first boundary after the call (the
//! boundary waits for the worker thread, so a native run is the same on
//! every machine), on the web at the first boundary after the browser has
//! decoded it, within the frame budget. A replay hands the journal back
//! ([`Uncooked::schedule`]) and each load is put in place at its recorded
//! tick, waited for there.

use crate::Fetch;
use crate::engine::{Engine, Hooks, hooks};
use crate::world::assets::{Entry, Payload, Status};
use serde_json::{Value, json};
use std::collections::BTreeMap;

mod budget;
mod environment;
mod gltf;
mod images;

pub(crate) use budget::*;
pub(crate) use environment::*;
pub(crate) use gltf::*;
pub use images::*;

/// A `LoadBudget` as a load holds to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Budget {
    pub bytes: f64,
    pub triangles: u32,
    pub texture_size: u32,
    pub frame_ms: f64,
}

enum Work {
    Image {
        image: Image,
        pixel: bool,
        linear: bool,
    },
    Gltf(Box<Gltf>),
    Environment(Box<Env>),
}

struct Job {
    request: u32,
    /// Its entry in the engine's assets.
    entry: usize,
    name: String,
    frame_ms: f64,
    work: Work,
}

/// One load put in place (`LoadCompletion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Completion {
    pub request: u32,
    pub tick: u64,
    pub ok: bool,
}

/// Every uncooked load of an engine.
#[derive(Default)]
pub(crate) struct Uncooked {
    next: u32,
    jobs: Vec<Job>,
    next_key: u64,
    /// What was put in place, oldest first (og.load.completions).
    pub journal: Vec<Completion>,
    /// A recorded journal to replay (og.load.replayCompletions), by request.
    schedule: Option<BTreeMap<u32, Completion>>,
    /// The entries uncooked loads made and not released (og.load.release
    /// frees these alone), with a ready glTF's textures in the scene.
    owned: BTreeMap<usize, Vec<og_render::TextureHandle>>,
    /// Host decodes of released loads, taken and dropped as they land.
    discard: Vec<u64>,
}

/// A pending entry and its job, queued for the tick boundaries.
fn queue(
    eng: &Engine,
    name: &str,
    budget: &Budget,
    work: impl FnOnce(&mut Uncooked) -> Work,
) -> usize {
    let mut assets = eng.assets.borrow_mut();
    let entry = assets.entries.len();
    assets.entries.push(Entry {
        path: name.to_string(),
        status: Status::Pending,
        payload: Payload::Uncooked,
    });
    let u = &mut assets.uncooked;
    let request = u.next;
    u.next += 1;
    u.owned.insert(entry, Vec::new());
    let work = work(u);
    u.jobs.push(Job {
        request,
        entry,
        name: name.to_string(),
        frame_ms: budget.frame_ms,
        work,
    });
    entry
}

/// The frame's time at a tick boundary: on the web, work stops once a
/// load's `frameMs` is spent and something was done; natively (and for a
/// replayed load at its tick) it never stops.
struct Clock<'a> {
    now: &'a dyn Fn() -> f64,
    start: f64,
    worked: bool,
}

impl Clock<'_> {
    fn over(&self, frame_ms: f64) -> bool {
        self.worked && (self.now)() - self.start > frame_ms
    }
}

/// Advances a job; whether it is finished (ready to put in place, or
/// failed). `wait` finishes it now.
fn step(job: &mut Job, clock: &mut Clock, h: Option<&dyn Hooks>, wait: bool) -> bool {
    match &mut job.work {
        Work::Image { image, .. } => decoded(image, h, wait),
        Work::Environment(env) => env_step(env, clock, job.frame_ms, wait),
        Work::Gltf(g) => {
            if g.error.is_some() {
                return true;
            }
            let mut images_done = true;
            for (_, image) in &mut g.images {
                images_done &= decoded(image, h, wait);
            }
            // Each primitive converted is a unit of the frame's work.
            while g.prims.len() < g.file.primitive_count() {
                if !wait && clock.over(job.frame_ms) {
                    return false;
                }
                match g.file.primitive(g.prims.len()) {
                    Ok(p) => g.prims.push(p),
                    Err(e) => {
                        g.error = Some(e);
                        return true;
                    }
                }
                clock.worked = true;
            }
            images_done
        }
    }
}

/// At a tick boundary: the finished loads put in place (natively every load
/// called since the last boundary, waited for; on the web those finished,
/// at most a frame budget's worth after the first piece of work), or with a
/// replayed journal each at its recorded tick. `tick` is the world's tick
/// (the next tick is the first to see them).
pub(crate) fn poll(eng: &Engine, tick: u64) {
    let h = hooks();
    let now = || h.as_ref().map_or(0.0, |h| h.now_ms());
    let mut clock = Clock {
        now: &now,
        start: now(),
        worked: false,
    };
    let wait_all = cfg!(not(target_arch = "wasm32"));
    // A released load's host decode is dropped once it lands.
    if let Some(h) = h.as_deref() {
        eng.assets
            .borrow_mut()
            .uncooked
            .discard
            .retain(|&key| matches!(h.image_decode_take(key), Fetch::Pending));
    }
    let mut jobs = std::mem::take(&mut eng.assets.borrow_mut().uncooked.jobs);
    let schedule = eng.assets.borrow().uncooked.schedule.clone();
    let mut kept = Vec::new();
    for mut job in jobs.drain(..) {
        let due = match &schedule {
            Some(s) => match s.get(&job.request) {
                Some(c) if c.tick <= tick => Some(Some(c.ok)),
                _ => None,
            },
            None => Some(None),
        };
        let Some(recorded) = due else {
            kept.push(job);
            continue;
        };
        // A replayed load is waited for at its tick; natively every load
        // is (determinism). On the web the frame budget holds.
        let wait = wait_all || recorded.is_some();
        if (!wait && clock.over(job.frame_ms)) || !step(&mut job, &mut clock, h.as_deref(), wait) {
            kept.push(job);
            continue;
        }
        let request = job.request;
        let ok = apply(eng, job, recorded == Some(false));
        eng.assets
            .borrow_mut()
            .uncooked
            .journal
            .push(Completion { request, tick, ok });
        clock.worked = true;
    }
    eng.assets.borrow_mut().uncooked.jobs.extend(kept);
}

/// Puts a finished job in place (its entry ready), or its entry failed
/// (`recorded_failure`: it failed in the run a replay follows); whether it
/// is ready.
fn apply(eng: &Engine, job: Job, recorded_failure: bool) -> bool {
    let name = job.name;
    let result = if recorded_failure {
        Err(format!("{name}: failed in the recorded run"))
    } else {
        match job.work {
            Work::Image {
                image,
                pixel,
                linear,
            } => take_image(image, &name).map(|d| {
                let mut rs = eng.render.borrow_mut();
                let handle = rs.scene.add_texture(og_render::TextureData {
                    width: d.width,
                    height: d.height,
                    rgba: d.rgba,
                    color_space: if linear {
                        og_render::ColorSpace::Linear
                    } else {
                        og_render::ColorSpace::Srgb
                    },
                    mips: og_render::Mips::Generate,
                });
                if pixel {
                    rs.scene
                        .set_texture_filter(handle, og_render::TextureFilter::Nearest);
                }
                rs.textures.push(handle);
                let texture = rs.textures.len() - 1;
                drop(rs);
                eng.assets.borrow_mut().entries[job.entry].payload = Payload::Image {
                    texture,
                    width: d.width,
                    height: d.height,
                };
            }),
            Work::Gltf(g) => put_gltf(eng, &name, job.entry, *g),
            Work::Environment(env) => put_environment(eng, &name, job.entry, *env),
        }
    };
    match result {
        Ok(()) => {
            eng.assets.borrow_mut().entries[job.entry].status = Status::Ready;
            true
        }
        Err(e) => {
            let reason = if e.starts_with(&name) {
                e
            } else {
                format!("{name}: {e}")
            };
            eng.assets.borrow_mut().entries[job.entry].status = Status::Failed(reason);
            false
        }
    }
}

/// `og.load.release`: frees an uncooked asset. A pending load is dropped
/// (it never completes, so it is never journaled); a ready image's texture
/// and a ready model's meshes and textures are emptied under their handles,
/// so nothing that still names them breaks or draws, and their GPU and CPU
/// copies go; the model's draw parts and cached raycast BVHs go too. The
/// handle then answers failed. Physics shapes built from its geometries are
/// world state and stay. Releasing again is a no-op.
pub(crate) fn release(eng: &Engine, index: Option<usize>) -> Result<(), String> {
    let i = index.ok_or("not an asset handle")?;
    let mut a = eng.assets.borrow_mut();
    let path = a.entries.get(i).ok_or("not an asset handle")?.path.clone();
    let released = format!("{path} was released (og.load.release)");
    if a.entries[i].status == Status::Failed(released.clone()) {
        return Ok(());
    }
    let Some(textures) = a.uncooked.owned.remove(&i) else {
        return Err(format!(
            "{path} is a cooked asset: og.load.release frees what og.load.imageData, og.load.gltfData and og.load.environmentData loaded (a cooked asset stays with its bundle, a streamed one with its group)"
        ));
    };
    if let Some(at) = a.uncooked.jobs.iter().position(|j| j.entry == i) {
        let job = a.uncooked.jobs.remove(at);
        let images: Vec<Image> = match job.work {
            Work::Image { image, .. } => vec![image],
            Work::Gltf(g) => g.images.into_iter().map(|(_, image)| image).collect(),
            Work::Environment(_) => Vec::new(),
        };
        for image in images {
            if let Decode::Host(key) = image.decode {
                a.uncooked.discard.push(key);
            }
        }
    }
    let payload = std::mem::replace(&mut a.entries[i].payload, Payload::Uncooked);
    a.entries[i].status = Status::Failed(released);
    drop(a);
    let mut rs = eng.render.borrow_mut();
    let rs = &mut *rs;
    match payload {
        Payload::Image { texture, .. } => {
            if let Some(&h) = rs.textures.get(texture) {
                rs.scene.set_texture(h, released_texture());
            }
        }
        Payload::Environment { map, .. } => {
            if let Some(h) = rs.env.maps.get_mut(map).and_then(|m| m.1.take()) {
                // The frame stops naming it now; og.Environment's next read
                // finds no map there and lights by the sky.
                if rs
                    .scene
                    .lighting
                    .environment_map
                    .is_some_and(|m| m.map == h)
                {
                    rs.scene.lighting.environment_map = None;
                }
                if let Some(m) = &mut rs.scene.lighting.environment_map
                    && m.blend.is_some_and(|b| b.to == h)
                {
                    m.blend = None;
                }
                rs.scene.remove_environment_map(h);
            }
        }
        Payload::Gltf { .. } => {
            let key = crate::world::assets::handle(i);
            let parts = rs.models.insert(key, Vec::new()).unwrap_or_default();
            rs.model_lods.remove(&key);
            rs.fx.models.remove(&key);
            rs.fx.model_materials.remove(&key);
            let mut meshes: Vec<og_render::MeshHandle> = parts.iter().map(|p| p.0).collect();
            meshes.sort_by_key(|m| m.index());
            meshes.dedup();
            for m in meshes {
                rs.scene.set_mesh(m, og_render::MeshData::default());
                crate::present::geometry::forget_mesh(rs, m);
            }
            for h in textures {
                rs.scene.set_texture(h, released_texture());
                rs.decoded.ids.remove(&h);
            }
        }
        _ => {}
    }
    Ok(())
}

/// `og.load.completions`.
pub(crate) fn completions(eng: &Engine) -> Value {
    Value::Array(
        eng.assets
            .borrow()
            .uncooked
            .journal
            .iter()
            .map(|c| json!({ "request": c.request, "tick": c.tick as f64, "ok": c.ok }))
            .collect(),
    )
}

/// `og.load.replayCompletions`: refused once a tick has run.
pub(crate) fn replay(eng: &Engine, tick: u64, list: &Value) -> Result<(), String> {
    if tick > 0 {
        return Err(format!(
            "og.load.replayCompletions is for a world that has not ticked; this one is at tick {tick}"
        ));
    }
    let items = list
        .as_array()
        .ok_or("completions is a list of { request, tick, ok }")?;
    let mut schedule = BTreeMap::new();
    for (i, c) in items.iter().enumerate() {
        let num = |k: &str| {
            c[k].as_f64()
                .filter(|v| *v >= 0.0 && v.fract() == 0.0)
                .ok_or_else(|| format!("completions[{i}].{k} is a whole number"))
        };
        let request = num("request")? as u32;
        let tick = num("tick")? as u64;
        let ok = c["ok"]
            .as_bool()
            .ok_or_else(|| format!("completions[{i}].ok is a boolean"))?;
        schedule.insert(request, Completion { request, tick, ok });
    }
    eng.assets.borrow_mut().uncooked.schedule = Some(schedule);
    Ok(())
}

#[cfg(test)]
mod tests;
