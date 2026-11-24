//! Models and HLOD proxies fading in (og.render.reveal).
//!
//! A streamed world brings its models into view at moments the player
//! must not notice: a cell's entities spawn at its load distance, and a
//! model or proxy whose textures were being prepared stops being hidden
//! (`crate::world::assets::Decoded::hidden_models`). Drawn whole at once,
//! either pops. So each starts a fade: for a few ticks its instances carry
//! how much of them is drawn ([`og_render::InstanceFx::reveal`]), and the
//! renderer dithers them in, in the lit, prepass and shadow passes.
//!
//! What fades, by default: the model of an entity a cell spawned (it
//! carries an og.CellMember), and a model or proxy that stops being
//! hidden. og.render.reveal on the entity, else on its cell, gives the
//! fade's length; 0 draws it whole at once. Nothing fades where an HLOD
//! proxy was already showing it. A light a cell spawned (og.PointLight,
//! og.SpotLight, og.RectLight) has nothing to dither: its intensity
//! ramps with the same fade, held and started with the cell's models, as
//! no proxy shows a light.
//!
//! Presentation only (docs/design.md §2.4). Nothing here is world state:
//! the settings are kept here, beside the renderer, as og.render.snap's
//! are; no component is read or written for them, nothing is hashed or
//! saved, no event is sent, and what a tick does never depends on it.
//!
//! The frames hold a fade at nothing until the renderer has the
//! pipelines its model draws with. An instance drawn for the first time
//! stops its frame while they compile, so a frame draws nothing of a held
//! one and the renderer makes its pipelines on the side
//! (`og_render::FrameStats::compiling_ahead`); the frames then draw the
//! fade as many ticks late as it was held. Usually that is none. The hold
//! is the display's alone: how long a first compile takes is the
//! device's, so a capture and a script's frame
//! (any but `FrameKind::Display`), which wait for what they draw and
//! reproduce run to run, read the fade by its ticks, and neither the hold
//! nor the delay is hashed.
//!
//! Stepped by the tick, never by the frames: [`step`] compares the world
//! each tick left with the one before it, as the next tick starts
//! (`crate::present::presentation::before_tick`) or when a frame is drawn
//! first, whichever comes first; the other then finds nothing new. A frame
//! only reads how far each fade is at its own time, so a capture is the
//! same whether or not frames were drawn before it, and a probe snapshot
//! keeps this state with the rest of the presentation checkpoint.

use crate::host::codec::{Rec, Slot};
use crate::host::wire::record;
use crate::present::extract::{RenderState, column_rows};
use og_core::IdMap;
use og_ecs::{Entity, QueryDesc, World};

/// A fade's length without an og.render.reveal: long enough to read as
/// the model arriving, short enough to finish before the player is near
/// (0.3 to 0.5 s is what streaming open worlds use for dithered LOD and
/// streaming transitions).
pub(crate) const SECONDS: f32 = 0.4;

/// One fade: the tick it started at and how long it lasts, in ticks, and
/// the handle of the model fading in.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Fade {
    start: f64,
    ticks: f64,
    model: u64,
    /// The frames hold it at nothing until one has been drawn since
    /// `Reveal::drawn` counted this many and no pipeline made ahead is
    /// compiling ([`Reveal::drawn`]); none once they draw it, or without
    /// a renderer.
    held: Option<u64>,
    /// The ticks the frames draw it late by, for having held it.
    late: f64,
}

impl Fade {
    /// A fade from tick `start`, held by the frames since `held`.
    fn new(start: f64, ticks: f64, model: u64, held: Option<u64>) -> Fade {
        Fade {
            start,
            ticks,
            model,
            held,
            late: 0.0,
        }
    }

    /// How much is drawn at `now` by its ticks, 0 to 1, eased at both
    /// ends: what a capture shows.
    fn at(&self, now: f64) -> f32 {
        let t = ((now - self.start) / self.ticks.max(1e-6)).clamp(0.0, 1.0) as f32;
        t * t * (3.0 - 2.0 * t)
    }

    /// Whether it runs at tick `now`.
    fn runs(&self, now: f64) -> bool {
        now >= self.start && now - self.start < self.ticks
    }

    /// How much a frame at `now` draws: nothing while held, then the fade
    /// as late as it was held.
    fn shown(&self, now: f64) -> f32 {
        match self.held {
            Some(_) => 0.0,
            None => self.at(now - self.late),
        }
    }

    /// Whether the frames have yet to finish it at tick `now`, `drawn`
    /// frames in: it runs by its ticks, its late end is still to come, or
    /// a frame has seen it and its pipelines compile. One no frame has
    /// seen waits for none past its own ticks (a world stepped without
    /// frames).
    fn busy(&self, now: f64, drawn: u64) -> bool {
        self.runs(now)
            || match self.held {
                Some(since) => since != drawn,
                None => now >= self.start && now - self.start - self.late < self.ticks,
            }
    }

    /// As a tick starts, `drawn` frames in: the frames stop holding it
    /// once one has seen it and no pipeline made ahead of it is
    /// `compiling`, and draw it from where a fade one tick old is.
    fn release(&mut self, now: f64, drawn: u64, compiling: bool) {
        if self.held.is_some_and(|since| since != drawn) && !compiling {
            self.held = None;
            self.late = (now - self.start - 1.0).max(0.0);
        }
    }
}

/// The fades running and what the last step saw of the world.
#[derive(Clone, Default)]
pub(crate) struct Reveal {
    /// The world's tick at the last step; none before the first.
    tick: Option<u64>,
    /// `World::structural_changes` then: while it holds, nothing spawned.
    structural: u64,
    /// The hidden model and proxy handles then, ascending.
    hidden: Vec<u64>,
    /// The entities cells had spawned then (og.CellMember).
    members: IdMap<u64, ()>,
    /// The entities fading in.
    entities: IdMap<u64, Fade>,
    /// The HLOD proxies fading in, by their model's handle.
    proxies: IdMap<u64, Fade>,
    /// The lights fading in, by entity (their fades' model is 0).
    lights: IdMap<u64, Fade>,
    /// og.render.reveal's lengths in seconds, by entity (0: no fade).
    settings: IdMap<u64, f32>,
    /// The time of the frame being read, in ticks, and whether it is a
    /// frame for the display (else a capture, which reads no hold).
    frame: f64,
    display: bool,
    /// The frames the renderer has drawn, and whether the last left a
    /// pipeline made ahead compiling ([`Reveal::drawn`]).
    drawn: u64,
    compiling: bool,
}

/// Compares the world as its tick left it with the last step's: starts the
/// fade of every model that appeared, and ends those that are over. Called
/// again for the same world it finds nothing.
pub(crate) fn step(rs: &mut RenderState, world: &World) {
    let time = world.time();
    let now = time.tick;
    let mut r = std::mem::take(&mut rs.reveal);
    // Only a step from the tick before, or within one tick, sees what
    // appeared: after a restore or a jump the sets are another world's.
    let continuous = r.tick.is_some_and(|t| now == t || now == t + 1);
    if r.tick != Some(now) {
        // A fade from a tick the world went back before never happened,
        // and one the frames have finished is over.
        let (at, drawn, compiling) = (now as f64, r.drawn, r.compiling);
        for fades in [&mut r.entities, &mut r.proxies, &mut r.lights] {
            fades.retain(|_, f| {
                f.release(at, drawn, compiling);
                at >= f.start && f.busy(at, drawn)
            });
        }
    }
    r.tick = Some(now);
    let mut hidden: Vec<u64> = rs.decoded.hidden_models.keys().copied().collect();
    hidden.sort_unstable();
    let shown: Vec<u64> = match continuous {
        true => r
            .hidden
            .iter()
            .copied()
            .filter(|h| hidden.binary_search(h).is_err())
            .collect(),
        false => Vec::new(),
    };
    let structural = world.structural_changes();
    if shown.is_empty() && structural == r.structural {
        r.hidden = hidden;
        rs.reveal = r;
        return;
    }
    let rate = f64::from(time.rate);
    let member = world.id_by_name(crate::world::stream::cells::MEMBER).ok();
    // The cell that spawned an entity (og.CellMember.cell, its only field).
    let cell_of = |e: u64| -> Option<u64> {
        let bytes = world.get_bytes(Entity::from_bits(e), member?).ok()?;
        Some(u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?))
    };
    // Every (entity, model, cell) that started being drawn, and every
    // (entity, cell) that started shining.
    let mut appeared: Vec<(u64, u64, Option<u64>)> = Vec::new();
    let mut lit: Vec<(u64, Option<u64>)> = Vec::new();
    if structural != r.structural {
        r.settings
            .retain(|&e, _| world.is_alive(Entity::from_bits(e)));
        // The cells' entities, and those the last step did not have.
        let mut members = IdMap::new();
        let mut spawned = Vec::new();
        let q = QueryDesc {
            read: vec![crate::world::stream::cells::MEMBER.to_string()],
            ..QueryDesc::default()
        };
        for a in world.query(&q).into_iter().flatten() {
            for e in a.entities() {
                let e = e.to_bits();
                members.insert(e, ());
                if continuous && !r.members.contains_key(&e) {
                    spawned.push(e);
                }
            }
        }
        r.members = members;
        let lights: Vec<_> = [&rs.draw.point, &rs.draw.spot, &rs.draw.rect]
            .into_iter()
            .flatten()
            .filter_map(|layout| world.id_by_name(&layout.schema).ok())
            .collect();
        for &e in &spawned {
            if lights
                .iter()
                .any(|&id| world.get_bytes(Entity::from_bits(e), id).is_ok())
            {
                lit.push((e, cell_of(e)));
            }
        }
        // Those with a model that draws now; one still hidden fades when
        // it is shown.
        let model = rs
            .draw
            .model
            .as_ref()
            .and_then(|layout| Some((layout, world.id_by_name(&layout.schema).ok()?)));
        if let Some((layout, id)) = model {
            for e in spawned {
                let Ok(bytes) = world.get_bytes(Entity::from_bits(e), id) else {
                    continue;
                };
                let Ok(m) = layout.read::<record::Model>(&bytes) else {
                    continue;
                };
                if m.visible && hidden.binary_search(&m.asset).is_err() {
                    appeared.push((e, m.asset, cell_of(e)));
                }
            }
        }
    }
    if !shown.is_empty()
        && let Some(layout) = rs.draw.model.as_ref()
    {
        let (asset, visible) = (Slot::of(layout, "asset"), Slot::of(layout, "visible"));
        for (e, v) in column_rows(world, layout).iter() {
            let model = v.num(&asset).unwrap_or(0.0) as u64;
            if shown.binary_search(&model).is_ok() && v.flag(&visible).unwrap_or(true) {
                appeared.push((e, model, cell_of(e)));
            }
        }
    }
    // Held until a frame has made its pipelines; with no renderer there
    // is none to wait for. Only display frames hold (`FrameKind`): any
    // other reproduces run to run, so it reads a fade by its ticks, as a
    // capture does, and waits for the pipelines it draws with.
    let held = (rs.backend.is_some() && rs.display.kind.displayed()).then_some(r.drawn);
    let fade =
        |seconds: f32, model: u64| Fade::new(now as f64, f64::from(seconds) * rate, model, held);
    if !appeared.is_empty() {
        // Where a proxy was showing a cell's models, they do not fade.
        let stood = rs
            .hlod
            .stood(|h| r.hidden.binary_search(&h).is_err() && !r.proxies.contains_key(&h));
        for (e, model, cell) in appeared {
            if stood.contains_key(&e) || r.entities.contains_key(&e) {
                continue;
            }
            if let Some(seconds) = r.seconds(Some(e), cell) {
                r.entities.insert(e, fade(seconds, model));
            }
        }
    }
    for (e, cell) in lit {
        if let Some(seconds) = r.seconds(Some(e), cell)
            && !r.lights.contains_key(&e)
        {
            r.lights.insert(e, fade(seconds, 0));
        }
    }
    // The proxies shown with nothing drawn in their place.
    let was_hidden = |h: u64| r.hidden.binary_search(&h).is_ok();
    for (proxy, cell) in rs.hlod.fresh(world, &shown, was_hidden) {
        if let Some(seconds) = r.seconds(None, cell)
            && !r.proxies.contains_key(&proxy)
        {
            r.proxies.insert(proxy, fade(seconds, proxy));
        }
    }
    r.hidden = hidden;
    r.structural = structural;
    rs.reveal = r;
}

/// The world's tick, inside a system or between ticks: when the readiness
/// queries ask ([`Reveal::pending`], [`Reveal::active`]).
pub(crate) fn tick(eng: &crate::engine::Engine) -> f64 {
    if let Some(fr) = crate::engine::running(eng.id) {
        // SAFETY: the frame is published while its system runs.
        return unsafe { fr.ctx() }.time().tick as f64;
    }
    eng.world().map_or(0.0, |w| w.time().tick as f64)
}

impl Reveal {
    /// og.render.reveal: `entity`'s fade lasts `seconds` (0: none).
    pub(crate) fn set(&mut self, entity: u64, seconds: f32) {
        self.settings.insert(entity, seconds);
        if seconds == 0.0 {
            self.entities.remove(&entity);
            self.lights.remove(&entity);
        }
    }

    /// The length in seconds of the fade a model starts as it appears, on
    /// `entity` (none: a proxy) spawned by `cell`: its own setting, else
    /// its cell's, else [`SECONDS`]; none when it draws whole at once.
    fn seconds(&self, entity: Option<u64>, cell: Option<u64>) -> Option<f32> {
        let own = entity.and_then(|e| self.settings.get(&e).copied());
        let seconds = own
            .or_else(|| cell.and_then(|c| self.settings.get(&c).copied()))
            .unwrap_or(SECONDS);
        (seconds != 0.0).then_some(seconds)
    }

    /// The frame being read is drawn at `now`, in ticks, for the
    /// `display` or as a capture.
    pub(crate) fn frame(&mut self, now: f64, display: bool) {
        self.frame = now;
        self.display = display;
    }

    /// The time of the frame being read, in ticks ([`Reveal::frame`]).
    pub(crate) fn now(&self) -> f64 {
        self.frame
    }

    /// How much of `fade` the frame being read draws.
    fn part(&self, fade: &Fade) -> f32 {
        match self.display {
            true => fade.shown(self.frame),
            false => fade.at(self.frame),
        }
    }

    /// The renderer drew a frame; `compiling`: a pipeline it made ahead of
    /// an instance still hidden is not made yet
    /// (`og_render::FrameStats::compiling_ahead`). The next step starts
    /// the held fades when none is.
    /// The fades the frames hold at nothing (a pipeline they draw with is
    /// still compiling, or no frame has seen them yet).
    pub(crate) fn held(&self) -> u32 {
        let held = |f: &Fade| u32::from(f.held.is_some());
        self.entities.values().map(held).sum::<u32>() + self.proxies.values().map(held).sum::<u32>()
    }

    pub(crate) fn drawn(&mut self, compiling: bool) {
        self.drawn = self.drawn.wrapping_add(1);
        self.compiling = compiling;
    }

    /// Whether any entity is fading in: the rows then ask [`Self::entity`].
    pub(crate) fn fading(&self) -> bool {
        !self.entities.is_empty()
    }

    /// How much of `entity`'s model is drawn on this frame, 0 to 1.
    pub(crate) fn entity(&self, entity: u64) -> f32 {
        self.entities.get(&entity).map_or(1.0, |f| self.part(f))
    }

    /// How much of the light `entity`'s intensity shines on this frame,
    /// 0 to 1.
    pub(crate) fn light(&self, entity: u64) -> f32 {
        self.lights.get(&entity).map_or(1.0, |f| self.part(f))
    }

    /// How much of the proxy `handle` is drawn on this frame, 0 to 1.
    pub(crate) fn proxy(&self, handle: u64) -> f32 {
        self.proxies.get(&handle).map_or(1.0, |f| self.part(f))
    }

    /// Of `handles`, the proxies fading in and how much of each is drawn,
    /// for what keeps the HLOD's draws ([`crate::present::hlod::PlaceIn`]):
    /// a fading proxy is built again every frame.
    pub(crate) fn proxy_bits(&self, handles: &[u64]) -> Vec<(u64, u32)> {
        if self.proxies.is_empty() {
            return Vec::new();
        }
        let drawn = |h: &u64| Some((*h, self.part(self.proxies.get(h)?).to_bits()));
        handles.iter().filter_map(drawn).collect()
    }

    /// Whether the model or proxy `handle`, which is not hidden, has yet to
    /// finish fading in at tick `now`: its fade runs, or the last step
    /// still saw it hidden and the next may start one. Its group's
    /// presentation is not ready until then (`StreamStatus.ready`). Never
    /// true once the steps stopped (a world without a renderer).
    pub(crate) fn pending(&self, handle: u64, now: f64) -> bool {
        let stepping = self.tick.is_some_and(|t| now <= t as f64 + 1.0);
        let busy = |f: &Fade| f.busy(now, self.drawn);
        self.proxies.get(&handle).is_some_and(busy)
            || self.entities.values().any(|f| f.model == handle && busy(f))
            || (stepping && self.hidden.binary_search(&handle).is_ok())
    }

    /// Whether anything is fading in at tick `now` (og.loading.idle waits
    /// for it).
    pub(crate) fn active(&self, now: f64) -> bool {
        let busy = |f: &Fade| f.busy(now, self.drawn);
        [&self.entities, &self.proxies, &self.lights]
            .into_iter()
            .any(|fades| fades.values().any(busy))
    }

    /// After a restore without this state's checkpoint: nothing fades, and
    /// the next step sees nothing appear. The settings stay.
    pub(crate) fn forget(&mut self) {
        let settings = std::mem::take(&mut self.settings);
        *self = Reveal {
            settings,
            ..Reveal::default()
        };
    }

    /// The fades into the presentation hash, in id order.
    pub(crate) fn hash(&self, mut put: impl FnMut(&[u8])) {
        for fades in [&self.entities, &self.proxies, &self.lights] {
            let mut list: Vec<(&u64, &Fade)> = fades.iter().collect();
            list.sort_unstable_by_key(|(id, _)| **id);
            for (id, f) in list {
                put(&id.to_le_bytes());
                put(&f.start.to_bits().to_le_bytes());
                put(&f.ticks.to_bits().to_le_bytes());
            }
        }
    }
}
