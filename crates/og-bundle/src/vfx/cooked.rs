//! Validation and binary transport of the native timeline; no author grammar.
use super::*;

fn value(v: &EffectValue) -> bool {
    match v {
        EffectValue::Constant(v) => v.is_finite(),
        EffectValue::Random(v) => v.iter().all(|v| v.is_finite()),
        EffectValue::Curve(c) => {
            (1..=og_render_data::MAX_CURVE_KEYS).contains(&c.keys.len())
                && c.keys.iter().all(|(t, v)| t.is_finite() && v.is_finite())
                && c.keys.windows(2).all(|w| w[0].0 <= w[1].0)
        }
    }
}

impl Vfx {
    pub fn validate(&self) -> Result<(), String> {
        if !self.duration.is_finite()
            || self.duration < 0.0
            || self.elements.len() > MAX_ELEMENTS
            || self.events.len() > MAX_EVENTS
            || self.params.len() > MAX_PARAMS
        {
            return Err("invalid VFX duration or timeline capacity".into());
        }
        for (i, p) in self.params.iter().enumerate() {
            if p.name.is_empty()
                || !p.default.is_finite()
                || self.params[..i].iter().any(|q| q.name == p.name)
            {
                return Err(format!("invalid or duplicate VFX parameter '{}'", p.name));
            }
        }
        for e in &self.events {
            if !e.time.is_finite()
                || e.time < 0.0
                || e.time > self.duration
                || e.name.is_empty()
                || e.name.len() > MAX_EVENT_NAME
            {
                return Err(format!("invalid VFX event '{}'", e.name));
            }
        }
        if self.events.windows(2).any(|w| w[0].time > w[1].time) {
            return Err("VFX events are not ordered by time".into());
        }
        for e in &self.elements {
            if !e.at.is_finite()
                || e.at < 0.0
                || !e.life.is_finite()
                || e.life < 0.0
                || !e.offset.is_finite()
                || !e.rotate.is_finite()
                || !e.velocity.is_finite()
                || !e.acceleration.is_finite()
                || [e.when, e.amount, e.stretch]
                    .into_iter()
                    .flatten()
                    .any(|i| i >= self.params.len())
            {
                return Err("invalid VFX element time, transform or parameter index".into());
            }
            use VfxKind as K;
            let valid = match &e.kind {
                K::Emitter { id, .. } => !id.is_empty(),
                K::Mesh {
                    shape,
                    effect,
                    scale,
                    spin,
                    color,
                    ..
                } => {
                    !effect.is_empty()
                        && value(scale)
                        && spin.is_finite()
                        && color.iter().all(|v| v.is_finite())
                        && match shape {
                            VfxShape::Model(id) => !id.is_empty(),
                            VfxShape::Ring(Some(r)) => {
                                r.inner.is_finite()
                                    && r.band.is_finite()
                                    && r.height.iter().all(|v| v.is_finite())
                                    && r.start.is_finite()
                                    && r.length.is_finite()
                                    && (3..=256).contains(&r.segments)
                                    && r.colors.iter().flatten().all(|v| v.is_finite())
                            }
                            _ => true,
                        }
                }
                K::Trail {
                    width,
                    lifetime,
                    color,
                    orbit,
                    rise,
                    ..
                } => width
                    .iter()
                    .chain(color)
                    .chain(orbit)
                    .chain([lifetime, rise])
                    .all(|v| v.is_finite()),
                K::Beam {
                    to,
                    width,
                    color,
                    segments,
                    jitter,
                    ..
                } => {
                    to.is_finite()
                        && width.is_finite()
                        && color.iter().all(|v| v.is_finite())
                        && (1..=256).contains(segments)
                        && jitter.is_finite()
                }
                K::Light {
                    color,
                    intensity,
                    distance,
                } => {
                    color.iter().all(|v| v.is_finite()) && value(intensity) && distance.is_finite()
                }
                K::Decal {
                    normal_scale,
                    uv,
                    size,
                    fade,
                    color,
                    emissive,
                    roughness,
                    metallic,
                    angle_fade,
                    turn,
                    ..
                } => {
                    size.is_finite()
                        && uv
                            .iter()
                            .chain(color)
                            .chain(emissive)
                            .chain(turn)
                            .chain([normal_scale, fade, roughness, metallic, angle_fade])
                            .all(|v| v.is_finite())
                }
                K::Glow { size, color } => value(size) && color.iter().all(|v| v.is_finite()),
                K::Shake { trauma } => trauma.is_finite(),
                K::Sound { id, volume, pitch } => {
                    !id.is_empty() && volume.is_finite() && pitch.iter().all(|v| v.is_finite())
                }
                K::Flipbook {
                    id, size, color, ..
                } => !id.is_empty() && size.is_finite() && color.iter().all(|v| v.is_finite()),
            };
            if !valid {
                return Err(format!("invalid VFX {} fields", e.kind.name()));
            }
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        crate::fx::cooked::encode(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let v: Self = crate::fx::cooked::decode(bytes)?;
        v.validate()?;
        Ok(v)
    }
}
