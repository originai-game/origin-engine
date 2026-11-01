//! Animation state block encoding and restoration.

use super::*;

// ---- the state block -------------------------------------------------------

pub(super) struct AnimState(pub(super) Arc<Mutex<Shared>>);

fn write_phase(w: &mut Writer, p: Phase) {
    w.u32(p.cycles);
    w.f32(p.frac);
}

fn read_phase(r: &mut Reader) -> Result<Phase> {
    Ok(Phase {
        cycles: r.u32()?,
        frac: r.f32()?,
    })
}

fn write_vec3(w: &mut Writer, v: Vec3) {
    for x in v.to_array() {
        w.f32(x);
    }
}

fn read_vec3(r: &mut Reader) -> Result<Vec3> {
    Ok(Vec3::new(r.f32()?, r.f32()?, r.f32()?))
}

/// Sections after the entities, each a tag and its contents, written only
/// when they hold something (so a world without them saves, and hashes, as
/// before they existed).
const SECTION_LOCKS: u32 = 1;
/// Graph layers whose fade a transition's `interrupt` guards.
const SECTION_GUARDS: u32 = 2;
/// Spring bones' tails.
const SECTION_SPRINGS: u32 = 3;
/// Skeleton entities with a hidden node: the hidden nodes of each.
const SECTION_HIDDEN: u32 = 4;

fn guard_code(g: Interrupt) -> u8 {
    match g {
        Interrupt::Any => 0,
        Interrupt::None => 1,
        Interrupt::Source => 2,
        Interrupt::Next => 3,
    }
}

fn encode(
    states: &IdMap<u64, EntityState>,
    locks: &IdMap<u64, LockState>,
    springs: &IdMap<u64, SpringState>,
    out: &mut Vec<u8>,
) {
    let mut w = Writer::new(out);
    w.u32(1); // format
    w.u32(states.len() as u32);
    for (&e, s) in states {
        w.u64(e);
        w.u64(s.skeleton);
        w.u32(s.skin);
        w.u32(s.mesh);
        w.u32(s.pose.len() as u32);
        for t in &s.pose {
            for v in t.translation.to_array() {
                w.f32(v);
            }
            for v in t.rotation.to_array() {
                w.f32(v);
            }
            for v in t.scale.to_array() {
                w.f32(v);
            }
        }
        for v in s.weights {
            w.f32(v);
        }
        let p = &s.player;
        w.u64(p.clip);
        w.f32(p.time);
        w.f32(p.written);
        w.f32(p.speed);
        w.u8(p.looping as u8);
        w.u64(p.prev_clip);
        w.f32(p.prev_time);
        w.f32(p.prev_speed);
        w.u8(p.prev_looping as u8);
        w.f32(p.fade);
        w.f32(p.fade_duration);
        w.u64(s.graph.asset);
        w.u32(s.graph.layers.len() as u32);
        for l in &s.graph.layers {
            w.u32(l.state);
            write_phase(&mut w, l.phase);
            w.f32(l.time);
            w.u32(l.prev);
            write_phase(&mut w, l.prev_phase);
            w.f32(l.fade);
            w.f32(l.fade_duration);
        }
    }
    if !locks.is_empty() {
        w.u32(SECTION_LOCKS);
        w.u32(locks.len() as u32);
        for (&e, l) in locks {
            w.u64(e);
            w.u64(l.skeleton);
            let (mode, at, t) = match l.lock {
                Lock::Free => (0, Vec3::ZERO, 0.0),
                Lock::Held(at) => (1, at, 0.0),
                Lock::Releasing(from, t) => (2, from, t),
            };
            w.u8(mode);
            write_vec3(&mut w, at);
            w.f32(t);
        }
    }
    let guards: Vec<(u64, u32, u8)> = states
        .iter()
        .flat_map(|(&e, s)| {
            s.graph
                .layers
                .iter()
                .enumerate()
                .filter(|(_, l)| l.guard != Interrupt::Any)
                .map(move |(i, l)| (e, i as u32, guard_code(l.guard)))
        })
        .collect();
    if !guards.is_empty() {
        w.u32(SECTION_GUARDS);
        w.u32(guards.len() as u32);
        for (e, layer, g) in guards {
            w.u64(e);
            w.u32(layer);
            w.u8(g);
        }
    }
    if !springs.is_empty() {
        w.u32(SECTION_SPRINGS);
        w.u32(springs.len() as u32);
        for (&e, s) in springs {
            w.u64(e);
            w.u64(s.skeleton);
            w.u32(s.space);
            w.u32(s.tails.len() as u32);
            for &(x, v) in &s.tails {
                write_vec3(&mut w, x);
                write_vec3(&mut w, v);
            }
        }
    }
    let hidden: Vec<(u64, Vec<u32>)> = states
        .iter()
        .filter_map(|(&e, s)| {
            let h: Vec<u32> = (0..s.visible.len() as u32)
                .filter(|&i| !s.visible[i as usize])
                .collect();
            (!h.is_empty()).then_some((e, h))
        })
        .collect();
    if !hidden.is_empty() {
        w.u32(SECTION_HIDDEN);
        w.u32(hidden.len() as u32);
        for (e, nodes) in hidden {
            w.u64(e);
            w.u32(nodes.len() as u32);
            for n in nodes {
                w.u32(n);
            }
        }
    }
}

fn decode(bytes: &[u8]) -> Result<Saved> {
    let mut r = Reader::new(bytes);
    let mut saved = Saved::default();
    if bytes.is_empty() {
        return Ok(saved);
    }
    let format = r.u32()?;
    if format != 1 {
        return Err(Error::new(
            ErrorKind::Parse,
            format!("og.anim state format {format}; this engine reads 1"),
        ));
    }
    let n = r.count(8)?;
    let mut last = None;
    for _ in 0..n {
        let e = r.u64()?;
        if last.is_some_and(|l| l >= e) {
            return Err(Error::new(
                ErrorKind::Parse,
                "og.anim state entities are not in order",
            ));
        }
        last = Some(e);
        let skeleton = r.u64()?;
        let skin = r.u32()?;
        let mesh = r.u32()?;
        let nodes = r.count(40)?;
        let mut pose = Vec::with_capacity(nodes);
        for _ in 0..nodes {
            let mut f = [0f32; 10];
            for v in f.iter_mut() {
                *v = r.f32()?;
            }
            pose.push(Trs::new(
                Vec3::new(f[0], f[1], f[2]),
                Quat::from_xyzw(f[3], f[4], f[5], f[6]),
                Vec3::new(f[7], f[8], f[9]),
            ));
        }
        let mut weights = [0f32; BLENDSHAPES];
        for v in weights.iter_mut() {
            *v = r.f32()?;
        }
        let player = PlayerState {
            clip: r.u64()?,
            time: r.f32()?,
            written: r.f32()?,
            speed: r.f32()?,
            looping: r.u8()? != 0,
            prev_clip: r.u64()?,
            prev_time: r.f32()?,
            prev_speed: r.f32()?,
            prev_looping: r.u8()? != 0,
            fade: r.f32()?,
            fade_duration: r.f32()?,
        };
        let asset = r.u64()?;
        let layers_n = r.count(32)?;
        let mut layers = Vec::with_capacity(layers_n);
        for _ in 0..layers_n {
            layers.push(LayerState {
                state: r.u32()?,
                phase: read_phase(&mut r)?,
                time: r.f32()?,
                prev: r.u32()?,
                prev_phase: read_phase(&mut r)?,
                fade: r.f32()?,
                fade_duration: r.f32()?,
                guard: Interrupt::Any,
            });
        }
        let visible = vec![true; pose.len()];
        saved.states.insert(
            e,
            EntityState {
                skeleton,
                skin,
                mesh,
                pose,
                weights,
                visible,
                player,
                graph: GraphState { asset, layers },
            },
        );
    }
    let mut last_tag = 0;
    while !r.done() {
        let tag = r.u32()?;
        if tag <= last_tag {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("og.anim state section {tag} after {last_tag}"),
            ));
        }
        last_tag = tag;
        match tag {
            SECTION_LOCKS => {
                let n = r.count(33)?;
                for _ in 0..n {
                    let e = r.u64()?;
                    let skeleton = r.u64()?;
                    let mode = r.u8()?;
                    let at = read_vec3(&mut r)?;
                    let t = r.f32()?;
                    let lock = match mode {
                        0 => Lock::Free,
                        1 => Lock::Held(at),
                        2 => Lock::Releasing(at, t),
                        m => {
                            return Err(Error::new(
                                ErrorKind::Parse,
                                format!("og.anim foot lock mode {m}"),
                            ));
                        }
                    };
                    saved.locks.insert(e, LockState { skeleton, lock });
                }
            }
            SECTION_GUARDS => {
                let n = r.count(13)?;
                for _ in 0..n {
                    let (e, layer, g) = (r.u64()?, r.u32()?, r.u8()?);
                    let guard = match g {
                        1 => Interrupt::None,
                        2 => Interrupt::Source,
                        3 => Interrupt::Next,
                        g => {
                            return Err(Error::new(
                                ErrorKind::Parse,
                                format!("og.anim interrupt guard {g}"),
                            ));
                        }
                    };
                    let l = saved
                        .states
                        .get_mut(&e)
                        .and_then(|s| s.graph.layers.get_mut(layer as usize))
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::Parse,
                                format!(
                                    "og.anim guard for entity {e} layer {layer}, which it lacks"
                                ),
                            )
                        })?;
                    l.guard = guard;
                }
            }
            SECTION_SPRINGS => {
                let n = r.count(24)?;
                for _ in 0..n {
                    let (e, skeleton, space) = (r.u64()?, r.u64()?, r.u32()?);
                    let joints = r.count(24)?;
                    let mut tails = Vec::with_capacity(joints);
                    for _ in 0..joints {
                        tails.push((read_vec3(&mut r)?, read_vec3(&mut r)?));
                    }
                    saved.springs.insert(
                        e,
                        SpringState {
                            skeleton,
                            space,
                            tails,
                        },
                    );
                }
            }
            SECTION_HIDDEN => {
                let n = r.count(12)?;
                for _ in 0..n {
                    let e = r.u64()?;
                    let count = r.count(4)?;
                    let s = saved.states.get_mut(&e).ok_or_else(|| {
                        Error::new(
                            ErrorKind::Parse,
                            format!("og.anim hidden nodes for entity {e}, which it lacks"),
                        )
                    })?;
                    for _ in 0..count {
                        let node = r.u32()?;
                        let v = s.visible.get_mut(node as usize).ok_or_else(|| {
                            Error::new(
                                ErrorKind::Parse,
                                format!("og.anim entity {e} hides node {node}, past its pose"),
                            )
                        })?;
                        *v = false;
                    }
                }
            }
            t => {
                return Err(Error::new(
                    ErrorKind::Parse,
                    format!("og.anim state section {t}; this engine reads up to {SECTION_HIDDEN}"),
                ));
            }
        }
    }
    r.finish()?;
    Ok(saved)
}

impl StateBlock for AnimState {
    fn save(&self, out: &mut Vec<u8>) {
        let sh = lock(&self.0);
        encode(&sh.states, &sh.locks, &sh.springs, out);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        Ok(Box::new(decode(bytes)?))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut sh = lock(&self.0);
        let saved = decoded
            .map(|d| *d.downcast::<Saved>().expect("decoded by AnimState::decode"))
            .unwrap_or_default();
        sh.states = saved.states;
        sh.locks = saved.locks;
        sh.springs = saved.springs;
        sh.recompute_outputs();
    }
}
