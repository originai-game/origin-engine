//! Native animation graph transport. Clip handles are local, one-based slots;
//! loading binds those slots to registered clips without parsing author text.
use super::*;
use og_core::bytes::{ByteReader, ByteWriter};

impl GraphSource {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut w = ByteWriter::new(&mut bytes);
        w.bool(self.sprite);
        w.length(self.bindings.len());
        for b in &self.bindings {
            w.str(&b.name);
            w.str(&b.source);
            w.str(&b.clip);
        }
        w.length(self.clips.len());
        for c in &self.clips {
            w.str(c);
        }
        let g = &self.asset;
        for strings in [&g.params, &g.sync_groups] {
            w.length(strings.len());
            for s in strings {
                w.str(s);
            }
        }
        w.length(g.nodes.len());
        for n in &g.nodes {
            w.str(&n.name);
            match &n.motion {
                Motion::Clip {
                    clip,
                    speed,
                    looping,
                    mirror,
                } => {
                    w.u8(0);
                    w.u64(*clip);
                    w.f32(*speed);
                    w.bool(*looping);
                    w.bool(*mirror);
                }
                Motion::Blend1D { param, children } => {
                    w.u8(1);
                    w.length(*param);
                    w.length(children.len());
                    for (threshold, node) in children {
                        w.f32(*threshold);
                        w.length(*node);
                    }
                }
                Motion::Blend2D { x, y, children } => {
                    w.u8(2);
                    w.length(*x);
                    w.length(*y);
                    w.length(children.len());
                    for (point, node) in children {
                        w.vec2(*point);
                        w.length(*node);
                    }
                }
            }
        }
        w.length(g.layers.len());
        for l in &g.layers {
            w.str(&l.name);
            w.u8(match l.mode {
                LayerMode::Override => 0,
                LayerMode::Additive => 1,
            });
            w.f32(l.weight);
            w.u32(l.weight_param.map_or(u32::MAX, |p| p as u32));
            w.length(l.entry);
            w.length(l.states.len());
            for s in &l.states {
                w.str(&s.name);
                w.length(s.motion);
                w.f32(s.speed);
                w.u32(s.sync.map_or(u32::MAX, |p| p as u32));
            }
            w.length(l.mask.len());
            for m in &l.mask {
                w.str(&m.node);
                w.f32(m.weight);
            }
            w.length(l.transitions.len());
            for t in &l.transitions {
                w.u32(t.from.map_or(u32::MAX, |p| p as u32));
                w.length(t.to);
                w.bool(t.exit.is_some());
                w.f32(t.exit.unwrap_or(0.0));
                w.f32(t.fade);
                w.u8(match t.interrupt {
                    Interrupt::Any => 0,
                    Interrupt::None => 1,
                    Interrupt::Source => 2,
                    Interrupt::Next => 3,
                });
                w.length(t.conditions.len());
                for c in &t.conditions {
                    w.length(c.param);
                    w.u8(match c.op {
                        Op::Gt => 0,
                        Op::Ge => 1,
                        Op::Lt => 2,
                        Op::Le => 3,
                        Op::Eq => 4,
                        Op::Ne => 5,
                    });
                    w.f32(c.value);
                }
            }
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut r = ByteReader::new(bytes, "animation graph");
        fn flag(r: &mut ByteReader<'_>) -> Result<bool> {
            match r.u8()? {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(r.error("invalid boolean")),
            }
        }
        fn optional(r: &mut ByteReader<'_>) -> Result<Option<usize>> {
            Ok(match r.u32()? {
                u32::MAX => None,
                n => Some(n as usize),
            })
        }
        fn strings(r: &mut ByteReader<'_>) -> Result<Vec<String>> {
            (0..r.count(4)?).map(|_| Ok(r.str()?.to_string())).collect()
        }
        let sprite = flag(&mut r)?;
        let mut bindings = Vec::new();
        for _ in 0..r.count(12)? {
            bindings.push(ClipBinding {
                name: r.str()?.into(),
                source: r.str()?.into(),
                clip: r.str()?.into(),
            });
        }
        let clips = strings(&mut r)?;
        let params = strings(&mut r)?;
        let sync_groups = strings(&mut r)?;
        let mut nodes = Vec::new();
        for _ in 0..r.count(5)? {
            let name = r.str()?.into();
            let motion = match r.u8()? {
                0 => Motion::Clip {
                    clip: r.u64()?,
                    speed: r.f32()?,
                    looping: flag(&mut r)?,
                    mirror: flag(&mut r)?,
                },
                1 => {
                    let param = r.u32()? as usize;
                    let children = (0..r.count(8)?)
                        .map(|_| Ok((r.f32()?, r.u32()? as usize)))
                        .collect::<Result<_>>()?;
                    Motion::Blend1D { param, children }
                }
                2 => {
                    let x = r.u32()? as usize;
                    let y = r.u32()? as usize;
                    let children = (0..r.count(12)?)
                        .map(|_| Ok((r.vec2()?, r.u32()? as usize)))
                        .collect::<Result<_>>()?;
                    Motion::Blend2D { x, y, children }
                }
                _ => return Err(r.error("invalid motion tag")),
            };
            nodes.push(MotionNode { name, motion });
        }
        let mut layers = Vec::new();
        for _ in 0..r.count(21)? {
            let name = r.str()?.into();
            let mode = match r.u8()? {
                0 => LayerMode::Override,
                1 => LayerMode::Additive,
                _ => return Err(r.error("invalid layer mode")),
            };
            let weight = r.f32()?;
            let weight_param = optional(&mut r)?;
            let entry = r.u32()? as usize;
            let mut states = Vec::new();
            for _ in 0..r.count(16)? {
                states.push(State {
                    name: r.str()?.into(),
                    motion: r.u32()? as usize,
                    speed: r.f32()?,
                    sync: optional(&mut r)?,
                });
            }
            let mut mask = Vec::new();
            for _ in 0..r.count(8)? {
                mask.push(MaskEntry {
                    node: r.str()?.into(),
                    weight: r.f32()?,
                });
            }
            let mut transitions = Vec::new();
            for _ in 0..r.count(22)? {
                let from = optional(&mut r)?;
                let to = r.u32()? as usize;
                let has_exit = flag(&mut r)?;
                let exit = r.f32()?;
                let fade = r.f32()?;
                let interrupt = match r.u8()? {
                    0 => Interrupt::Any,
                    1 => Interrupt::None,
                    2 => Interrupt::Source,
                    3 => Interrupt::Next,
                    _ => return Err(r.error("invalid interrupt")),
                };
                let mut conditions = Vec::new();
                for _ in 0..r.count(9)? {
                    let param = r.u32()? as usize;
                    let op = match r.u8()? {
                        0 => Op::Gt,
                        1 => Op::Ge,
                        2 => Op::Lt,
                        3 => Op::Le,
                        4 => Op::Eq,
                        5 => Op::Ne,
                        _ => return Err(r.error("invalid comparison")),
                    };
                    conditions.push(Condition {
                        param,
                        op,
                        value: r.f32()?,
                    });
                }
                transitions.push(Transition {
                    from,
                    to,
                    conditions,
                    exit: has_exit.then_some(exit),
                    fade,
                    interrupt,
                });
            }
            layers.push(Layer {
                name,
                mode,
                weight,
                weight_param,
                states,
                entry,
                transitions,
                mask,
            });
        }
        r.finish()?;
        let asset = GraphAsset {
            params,
            nodes,
            layers,
            sync_groups,
        };
        let graph = Self {
            asset,
            clips,
            sprite,
            bindings,
        };
        graph.validate()?;
        Ok(graph)
    }
}
