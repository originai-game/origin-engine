//! The `og.skeleton2d` system: plays 2D rigs ([`crate::Rig2dAsset`]) on
//! `og.Skeleton2D` components by the scaled tick, fires their clip events
//! as `og.Skeleton2DEvent`, and solves each `og.IkTarget2D` against its
//! skeleton's pose to write `reached` and `error`, and keeps the bounding
//! boxes each pose shows ([`Skeleton2dSystem::boxes`], part of its state
//! block) for og.anim.skeleton2dBoxes. The pose itself is a function of the
//! component (rig, clip, time, flipX), its og.Transform and its IK targets
//! ([`skeleton_pose`]), so extraction draws it from the same inputs and
//! nothing visual is kept as state.

use crate::rig2d::{Affine2, Pose2d, Rig2dAsset, atan2};
use crate::schema::Transform;
use og_core::glam::Vec2;
use og_core::{Error, ErrorKind, Reflect, Result, Str64};
use og_ecs::{Entity, QueryDesc, StateBlock, SystemContext, SystemDesc, World};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

/// The system's and the state block's name.
pub const SKELETON2D_SYSTEM: &str = "og.skeleton2d";

/// Plays a clip of a 2D rig at the entity: drawn as textured triangles in
/// the sprite order at `layer`, lit by og.Lighting2D like a sprite there.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Skeleton2D")]
pub struct Skeleton2D {
    /// The rig (og.load.rig2d's `rig`); 0 is none.
    pub rig: u64,
    /// The clip: an index into Rig2DInfo.clips.
    #[reflect(min = 0)]
    pub clip: u32,
    /// Seconds into the clip; written every tick, set it to seek.
    #[reflect(unit = "s", min = 0)]
    pub time: f32,
    /// The playback rate: 1 as authored, negative plays backwards.
    #[reflect(min = -10, max = 10, default = 1, tunable)]
    pub speed: f32,
    /// Wrap at the clip's end; otherwise hold its last pose and set `finished`.
    #[reflect(name = "loop", default = 1)]
    pub looping: bool,
    /// Mirror the rig left to right about the entity's position.
    #[reflect(name = "flipX")]
    pub flip_x: bool,
    /// The sprite layer it draws in.
    pub layer: i32,
    /// Written: a clip that does not loop reached its end.
    pub finished: bool,
}

impl Default for Skeleton2D {
    fn default() -> Self {
        Skeleton2D {
            rig: 0,
            clip: 0,
            time: 0.0,
            speed: 1.0,
            looping: true,
            flip_x: false,
            layer: 0,
            finished: false,
        }
    }
}

/// A clip event a 2D skeleton passed in the last tick.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.Skeleton2DEvent")]
pub struct Skeleton2DEvent {
    /// The entity with the og.Skeleton2D.
    pub entity: u64,
    /// The clip index.
    pub clip: u32,
    /// The event's name.
    pub name: Str64,
    /// The event's time in the clip.
    #[reflect(unit = "s")]
    pub time: f32,
}

/// Moves one IK constraint's target of a 2D skeleton (a foot to the
/// ground under it, a hand to a ledge).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.IkTarget2D")]
pub struct IkTarget2D {
    /// The entity with the og.Skeleton2D.
    pub skeleton: u64,
    /// The constraint: an index into Rig2DInfo.constraints.
    #[reflect(min = 0)]
    pub constraint: u32,
    /// Where the chain should reach, in the frame the skeleton's og.Transform is written in.
    #[reflect(unit = "m")]
    pub target: Vec2,
    /// How much of the solve applies (0 the animated pose, 1 solved), times
    /// the constraint's own weight (its `weight` line or its clip's IK keys).
    #[reflect(min = 0, max = 1, default = 1, tunable)]
    pub weight: f32,
    /// The distance at which the target counts as reached.
    #[reflect(min = 0, default = 0.01, unit = "m")]
    pub tolerance: f32,
    /// Written: the chain's tip is within `tolerance` of the target.
    pub reached: bool,
    /// Written: how far the tip is from the target after solving.
    #[reflect(unit = "m")]
    pub error: f32,
}

impl Default for IkTarget2D {
    fn default() -> Self {
        IkTarget2D {
            skeleton: 0,
            constraint: 0,
            target: Vec2::ZERO,
            weight: 1.0,
            tolerance: 0.01,
            reached: false,
            error: 0.0,
        }
    }
}

/// The transform from a rig's pixels to the frame its entity's og.Transform
/// is in: scaled by 1 / unit, mirrored by flipX, then the Transform's
/// position, turn about z and scale.
pub fn skeleton_frame(rig: &Rig2dAsset, t: &Transform, flip_x: bool) -> Affine2 {
    let q = t.quaternion;
    let degrees =
        (2.0 * atan2(f64::from(q.z), f64::from(q.w)) * 180.0 / std::f64::consts::PI) as f32;
    let place = Affine2::trs(t.position.x, t.position.y, degrees, t.scale.x, t.scale.y);
    let s = 1.0 / rig.unit;
    place.mul(&Affine2::trs(0.0, 0.0, 0.0, if flip_x { -s } else { s }, s))
}

/// The pose of a skeleton whose rig-to-parent frame is `frame`, with IK
/// `targets` (constraint, target in that frame, weight).
pub fn skeleton_pose(
    rig: &Rig2dAsset,
    s: &Skeleton2D,
    frame: &Affine2,
    targets: &[(u32, Vec2, f32)],
) -> Pose2d {
    let inv = frame.inverse();
    let t: Vec<(usize, [f32; 2], f32)> = targets
        .iter()
        .map(|(c, p, w)| (*c as usize, inv.apply(p.x, p.y), *w))
        .collect();
    rig.pose(rig.clips.get(s.clip as usize), s.time, &t)
}

/// A bounding box a skeleton's pose showed: its slot (`slot/child-slot`
/// inside a child armature), its attachment's name and its polygon in the
/// frame the skeleton's og.Transform is written in.
#[derive(Clone, Debug, PartialEq)]
pub struct Skeleton2dBox {
    pub slot: String,
    pub name: String,
    pub points: Vec<Vec2>,
}

/// Per entity: what it played last tick, to tell a new clip or a seek.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Playback {
    rig: u64,
    clip: u32,
    time: f32,
}

#[derive(Default)]
struct Shared {
    rigs: BTreeMap<u64, Arc<Rig2dAsset>>,
    playback: BTreeMap<u64, Playback>,
    /// Per skeleton whose rig has boxes: the boxes its last pose showed.
    boxes: BTreeMap<u64, Vec<Skeleton2dBox>>,
}

fn lock(m: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Registers 2D rigs and reads them back for drawing.
#[derive(Clone)]
pub struct Skeleton2dSystem {
    shared: Arc<Mutex<Shared>>,
}

impl Skeleton2dSystem {
    /// Registers (or replaces) rig `handle` (not 0).
    pub fn add_rig(&self, handle: u64, rig: Rig2dAsset) -> Result<()> {
        if handle == 0 {
            return Err(Error::new(ErrorKind::Invalid, "rig handle 0 means none"));
        }
        lock(&self.shared).rigs.insert(handle, Arc::new(rig));
        Ok(())
    }

    pub fn rig(&self, handle: u64) -> Option<Arc<Rig2dAsset>> {
        lock(&self.shared).rigs.get(&handle).cloned()
    }

    /// The bounding boxes skeleton `entity` (its bits) showed when
    /// og.skeleton2d last posed it; `None` before that, or when it has no
    /// og.Skeleton2D with a loaded rig.
    pub fn boxes(&self, entity: u64) -> Option<Vec<Skeleton2dBox>> {
        lock(&self.shared).boxes.get(&entity).cloned()
    }
}

/// Registers og.Skeleton2D, og.Skeleton2DEvent and og.IkTarget2D and
/// schedules `og.skeleton2d` here, with its state block.
pub fn install_skeleton2d(world: &mut World) -> Result<Skeleton2dSystem> {
    world.register::<Transform>()?;
    world.register::<Skeleton2D>()?;
    world.register::<Skeleton2DEvent>()?;
    world.register::<IkTarget2D>()?;
    let shared = Arc::new(Mutex::new(Shared::default()));
    let desc = SystemDesc::new(SKELETON2D_SYSTEM)
        .read::<Transform>()
        .write::<Skeleton2D>()
        .write::<IkTarget2D>()
        .write::<Skeleton2DEvent>()
        .query(QueryDesc::new().write::<Skeleton2D>())
        .query(QueryDesc::new().write::<IkTarget2D>());
    let s = shared.clone();
    world.add_system(desc, move |ctx| run(ctx, &s))?;
    world.add_state_block(SKELETON2D_SYSTEM, Box::new(SkeletonState(shared.clone())))?;
    Ok(Skeleton2dSystem { shared })
}

/// The most events one skeleton sends per tick.
const MAX_EVENTS: usize = 64;

fn run(ctx: &mut SystemContext<'_>, shared: &Mutex<Shared>) -> Result<()> {
    let dt = ctx.scaled_dt();
    let mut skeletons: Vec<Entity> = Vec::new();
    for a in ctx.query(0)? {
        skeletons.extend_from_slice(a.entities());
    }
    let mut targets: Vec<Entity> = Vec::new();
    for a in ctx.query(1)? {
        targets.extend_from_slice(a.entities());
    }
    let mut sh = lock(shared);
    let mut events = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for &e in &skeletons {
        let key = e.to_bits();
        let mut s: Skeleton2D = ctx.get(e)?;
        let Some(rig) = sh.rigs.get(&s.rig).cloned() else {
            continue;
        };
        let Some(clip) = rig.clips.get(s.clip as usize) else {
            continue;
        };
        seen.insert(key);
        let prev = sh.playback.get(&key).copied();
        let fresh = prev.is_none_or(|p| p.clip != s.clip || p.rig != s.rig);
        let mut from = s.time;
        if fresh {
            // A new clip starts at 0 (firing the events at 0), unless the
            // game set the time in the same tick: a seek fires none.
            if prev.is_some_and(|p| p.time == s.time) {
                s.time = 0.0;
            }
            s.finished = false;
            from = if s.time == 0.0 { -1.0 } else { s.time };
        } else if prev.is_some_and(|p| p.time != s.time) {
            s.finished = false;
        }
        let d = clip.duration;
        let step = dt * s.speed.clamp(-10.0, 10.0);
        let mut fired = Vec::new();
        if !s.finished {
            let to = s.time + step;
            if s.looping {
                // Events crossed on the way, cycle by cycle.
                if step >= 0.0 {
                    let mut lo = from;
                    let mut hi = to;
                    for _ in 0..MAX_EVENTS {
                        fired.extend(
                            clip.events
                                .iter()
                                .filter(|(t, _)| *t > lo && *t <= hi.min(d)),
                        );
                        if hi < d {
                            break;
                        }
                        hi -= d;
                        lo = -1.0;
                    }
                    s.time = to - d * (to / d).floor();
                } else {
                    let (mut lo, mut hi) = (to, from.max(0.0));
                    for _ in 0..MAX_EVENTS {
                        fired.extend(
                            clip.events
                                .iter()
                                .filter(|(t, _)| *t >= lo.max(0.0) && *t < hi),
                        );
                        if lo >= 0.0 {
                            break;
                        }
                        lo += d;
                        hi = d;
                    }
                    s.time = to - d * (to / d).floor();
                }
            } else {
                let to = to.clamp(0.0, d);
                let (lo, hi) = if step >= 0.0 { (from, to) } else { (to, from) };
                fired.extend(clip.events.iter().filter(|(t, _)| *t > lo && *t <= hi));
                s.time = to;
                s.finished = (step > 0.0 && to >= d) || (step < 0.0 && to <= 0.0);
            }
        }
        for (t, name) in fired.into_iter().take(MAX_EVENTS) {
            if let Ok(name) = Str64::new(name) {
                events.push(Skeleton2DEvent {
                    entity: key,
                    clip: s.clip,
                    name,
                    time: *t,
                });
            }
        }
        ctx.set(e, s)?;
        sh.playback.insert(
            key,
            Playback {
                rig: s.rig,
                clip: s.clip,
                time: s.time,
            },
        );
    }
    sh.playback.retain(|k, _| seen.contains(k));
    // Each skeleton with IK targets or boxes posed at the new time: the
    // targets' `reached` and `error`, and the boxes the pose shows.
    let mut by_skeleton: BTreeMap<u64, Vec<(Entity, IkTarget2D)>> = BTreeMap::new();
    for e in targets {
        let t: IkTarget2D = ctx.get(e)?;
        by_skeleton.entry(t.skeleton).or_default().push((e, t));
    }
    let mut boxes = BTreeMap::new();
    for e in skeletons {
        let key = e.to_bits();
        let list = by_skeleton.remove(&key).unwrap_or_default();
        let s: Skeleton2D = ctx.get(e)?;
        let Some(rig) = sh.rigs.get(&s.rig).cloned() else {
            continue;
        };
        let has_boxes = rig.has_boxes();
        if list.is_empty() && !has_boxes {
            continue;
        }
        let place = ctx.get::<Transform>(e).unwrap_or_default();
        let frame = skeleton_frame(&rig, &place, s.flip_x);
        let goals: Vec<(u32, Vec2, f32)> = list
            .iter()
            .map(|(_, t)| (t.constraint, t.target, t.weight))
            .collect();
        let pose = skeleton_pose(&rig, &s, &frame, &goals);
        // Pixels to the Transform's frame: the frame's scale along x.
        let scale = (frame.a * frame.a + frame.b * frame.b).sqrt();
        for (e, mut t) in list {
            let Some(err) = pose.ik_error.get(t.constraint as usize) else {
                continue;
            };
            t.error = err * scale;
            t.reached = t.error <= t.tolerance;
            ctx.set(e, t)?;
        }
        if has_boxes {
            let shown = rig
                .boxes(&pose)
                .into_iter()
                .map(|b| Skeleton2dBox {
                    slot: b.slot,
                    name: b.name,
                    points: b
                        .points
                        .iter()
                        .map(|p| Vec2::from(frame.apply_to(*p)))
                        .collect(),
                })
                .collect();
            boxes.insert(key, shown);
        }
    }
    sh.boxes = boxes;
    drop(sh);
    for ev in events {
        ctx.send(ev)?;
    }
    Ok(())
}

struct SkeletonState(Arc<Mutex<Shared>>);

/// Reads the state block's little-endian fields in order.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn bad() -> Error {
        Error::new(ErrorKind::Parse, "og.skeleton2d state block: truncated")
    }

    fn take(&mut self, n: usize) -> Result<&[u8]> {
        let b = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or_else(Reader::bad)?;
        self.at += n;
        Ok(b)
    }

    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }

    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }

    fn text(&mut self) -> Result<String> {
        let len = self.u32()? as usize;
        String::from_utf8(self.take(len)?.to_vec()).map_err(|_| Reader::bad())
    }
}

/// The state block's contents: the playback and the boxes.
type Saved = (BTreeMap<u64, Playback>, BTreeMap<u64, Vec<Skeleton2dBox>>);

impl StateBlock for SkeletonState {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        out.extend_from_slice(&(sh.playback.len() as u32).to_le_bytes());
        for (k, p) in &sh.playback {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&p.rig.to_le_bytes());
            out.extend_from_slice(&p.clip.to_le_bytes());
            out.extend_from_slice(&p.time.to_le_bytes());
        }
        out.extend_from_slice(&(sh.boxes.len() as u32).to_le_bytes());
        let text = |out: &mut Vec<u8>, s: &str| {
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        };
        for (k, list) in &sh.boxes {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&(list.len() as u32).to_le_bytes());
            for b in list {
                text(out, &b.slot);
                text(out, &b.name);
                out.extend_from_slice(&(b.points.len() as u32).to_le_bytes());
                for p in &b.points {
                    out.extend_from_slice(&p.x.to_le_bytes());
                    out.extend_from_slice(&p.y.to_le_bytes());
                }
            }
        }
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = Reader { bytes, at: 0 };
        let n = r.u32()?;
        let mut playback = BTreeMap::new();
        for _ in 0..n {
            let key = r.u64()?;
            playback.insert(
                key,
                Playback {
                    rig: r.u64()?,
                    clip: r.u32()?,
                    time: f32::from_bits(r.u32()?),
                },
            );
        }
        let mut boxes = BTreeMap::new();
        for _ in 0..r.u32()? {
            let key = r.u64()?;
            let mut list = Vec::new();
            for _ in 0..r.u32()? {
                let slot = r.text()?;
                let name = r.text()?;
                let points = (0..r.u32()?)
                    .map(|_| {
                        Ok(Vec2::new(
                            f32::from_bits(r.u32()?),
                            f32::from_bits(r.u32()?),
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                list.push(Skeleton2dBox { slot, name, points });
            }
            boxes.insert(key, list);
        }
        if r.at != bytes.len() {
            return Err(Reader::bad());
        }
        let saved: Saved = (playback, boxes);
        Ok(Box::new(saved))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let (playback, boxes) = decoded
            .map(|d| *d.downcast::<Saved>().expect("decoded by SkeletonState"))
            .unwrap_or_default();
        let mut sh = lock(&self.0);
        sh.playback = playback;
        sh.boxes = boxes;
    }
}
