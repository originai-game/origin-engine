//! Checked FX declarations become native render data, never author text.
//! Resource callbacks bind canonical content references; field assets are
//! supplied by the cook's prepared lattice/image inputs.
use super::*;
use og_render_data as render;

fn curve(keys: &[[f32; 2]]) -> Result<render::Curve, String> {
    ordered(keys.iter().map(|p| p[0]))?;
    Ok(render::Curve::new(
        &keys.iter().map(|p| (p[0], p[1])).collect::<Vec<_>>(),
    ))
}

fn ordered(times: impl Iterator<Item = f32>) -> Result<(), String> {
    let mut previous = f32::NEG_INFINITY;
    let mut count = 0;
    for time in times {
        if !time.is_finite() || time < previous {
            return Err("curve keys must be finite and ordered by time".into());
        }
        previous = time;
        count += 1;
    }
    if !(1..=render::MAX_CURVE_KEYS).contains(&count) {
        return Err(format!("a curve has 1..={} keys", render::MAX_CURVE_KEYS));
    }
    Ok(())
}

fn color_curve(keys: &[(f32, [f32; 4])]) -> Result<render::ColorCurve, String> {
    ordered(keys.iter().map(|p| p.0))?;
    Ok(render::ColorCurve::new(keys))
}

fn value(input: &EffectValue) -> Result<render::EffectValue, String> {
    Ok(match input {
        EffectValue::Constant(v) => render::EffectValue::Constant(*v),
        EffectValue::Variable(ValueInput::Random { min, max }) => {
            render::EffectValue::Random([*min, *max])
        }
        EffectValue::Variable(ValueInput::Curve { keys }) => {
            render::EffectValue::Curve(curve(keys)?)
        }
    })
}

fn blend(input: &Blend) -> render::ParticleBlend {
    match input {
        Blend::Alpha => render::ParticleBlend::Alpha,
        Blend::Additive => render::ParticleBlend::Additive,
        Blend::Multiply => render::ParticleBlend::Multiply,
        Blend::Subtractive => render::ParticleBlend::Subtractive,
    }
}

fn slot(
    input: &EffectTextureSlot,
    default: render::EffectChannel,
    texture: &mut dyn FnMut(&str) -> Result<Option<render::TextureHandle>, String>,
) -> Result<render::EffectTexture, String> {
    let mut uv = render::EffectUv::default();
    if let Some(input) = &input.uv {
        if let Some(v) = input.tile {
            uv.tile = v.into();
        }
        if let Some(v) = input.offset {
            uv.offset = v.into();
        }
        if let Some(v) = input.scroll {
            uv.scroll = v.into();
        }
        if let Some(v) = input.rotate {
            uv.rotation = v;
        }
        if let Some(v) = input.polar {
            uv.polar = v;
        }
    }
    Ok(render::EffectTexture {
        texture: input
            .texture
            .as_ref()
            .map(|r| texture(&r.id))
            .transpose()?
            .flatten(),
        uv,
        channel: match input.channel.as_ref() {
            None => default,
            Some(Channel::Rgba) => render::EffectChannel::Rgba,
            Some(Channel::Red) => render::EffectChannel::Red,
            Some(Channel::Green) => render::EffectChannel::Green,
            Some(Channel::Blue) => render::EffectChannel::Blue,
            Some(Channel::Alpha) => render::EffectChannel::Alpha,
        },
    })
}

impl EffectInput {
    /// Build validated native material fields, resolving resources through the
    /// same typed links as the runtime. This does not serialize author text.
    pub fn build(
        &self,
        links: &mut og_bundle::effect::EffectLinks<'_>,
    ) -> Result<render::EffectMaterial, String> {
        let mut m = render::EffectMaterial::default();
        if let Some(v) = &self.main {
            m.main = slot(v, render::EffectChannel::Rgba, links.texture)?;
        }
        if let Some(v) = &self.mask {
            m.mask = Some(slot(v, render::EffectChannel::Red, links.texture)?);
        }
        if let Some(v) = &self.ramp {
            m.ramp = Some(color_curve(v)?);
        }
        if let Some(amount) = &self.erosion {
            let mut erosion = render::Erosion {
                amount: value(amount)?,
                ..Default::default()
            };
            if let Some(v) = &self.noise {
                erosion.noise = slot(v, render::EffectChannel::Red, links.texture)?;
            }
            if let Some(v) = self.erosion_softness {
                erosion.softness = v;
            }
            if let Some(v) = &self.erosion_edge {
                erosion.edge = v.width;
                erosion.edge_color = v.color;
            }
            m.erosion = Some(erosion);
        } else if self.noise.is_some() {
            return Err("a noise texture needs erosion".into());
        }
        if let Some(map) = &self.flow_map {
            m.flow = Some(render::Flow {
                map: slot(map, render::EffectChannel::Rgba, links.texture)?,
                strength: self.flow.as_ref().and_then(|v| v.strength).unwrap_or(0.05),
                speed: self.flow.as_ref().and_then(|v| v.frequency).unwrap_or(0.5),
            });
        }
        if let Some(v) = &self.fresnel {
            m.fresnel = Some(render::Fresnel {
                power: v.power,
                strength: v.strength.unwrap_or(1.0),
                invert: v.invert.unwrap_or(false),
            });
        }
        if let Some(v) = self.soft_depth {
            m.soft_depth = v;
        }
        if let Some(v) = self.color {
            m.color = v;
        }
        if let Some(v) = &self.intensity {
            m.intensity = value(v)?;
        }
        if let Some(v) = &self.alpha {
            m.alpha = value(v)?;
        }
        if let Some(v) = &self.offset_u {
            m.offset[0] = value(v)?;
        }
        if let Some(v) = &self.offset_v {
            m.offset[1] = value(v)?;
        }
        if let Some(custom) = &self.custom {
            if custom.len() > 8 {
                return Err("an effect has at most 8 custom values".into());
            }
            for (v, target) in custom.iter().zip(m.custom.iter_mut().flatten()) {
                *target = value(v)?;
            }
        }
        m.blend = self.blend.as_ref().map(blend);
        m.double_sided = self.double_sided.unwrap_or(false);
        if let Some(shader) = &self.shader {
            if let Some((handle, params)) = (links.shader)(&shader.id)? {
                m.shader = Some(handle);
                for (target, p) in m.params.iter_mut().zip(&params) {
                    *target = p.default;
                }
                for (name, v) in self.params.iter().flatten() {
                    let index = params
                        .iter()
                        .position(|p| &p.name == name)
                        .ok_or_else(|| format!("the shader declares no parameter '{name}'"))?;
                    let target = m.params.get_mut(index).ok_or_else(|| {
                        format!("parameter '{name}' exceeds the effect parameter slots")
                    })?;
                    *target = [0.0; 4];
                    target[..v.0.len()].copy_from_slice(&v.0);
                }
            }
        } else if self.params.as_ref().is_some_and(|p| !p.is_empty()) {
            return Err("effect params need a shader".into());
        }
        m.validate()?;
        Ok(m)
    }
}

fn mesh(
    input: &MeshInput,
    links: &mut og_bundle::fx::EmitterLinks<'_>,
) -> Result<render::MeshHandle, String> {
    use og_bundle::fx::ParticleMesh as M;
    let reference = match input {
        MeshInput::Builtin(ParticleMesh::Box) => M::Builtin("box"),
        MeshInput::Builtin(ParticleMesh::Sphere) => M::Builtin("sphere"),
        MeshInput::Builtin(ParticleMesh::Tetrahedron) => M::Builtin("tetrahedron"),
        MeshInput::Model(r) => M::Asset(&r.id),
    };
    Ok((links.mesh)(reference)?.unwrap_or(render::MeshHandle::from_bits(0)))
}

fn sheet_mode(input: &SheetMode) -> render::SheetMode {
    match input {
        SheetMode::Off => render::SheetMode::Off,
        SheetMode::Random => render::SheetMode::Random,
        SheetMode::Life { cycles } => render::SheetMode::OverLife {
            cycles: cycles.unwrap_or(1.0),
        },
        SheetMode::Fps { fps } => render::SheetMode::Fps(*fps),
    }
}

impl ParticlesInput {
    /// Build the native emitter. Asset vector fields are supplied as prepared
    /// grids, never guessed sibling files or text inserted into a declaration.
    pub fn build(
        &self,
        links: &mut og_bundle::fx::EmitterLinks<'_>,
        field: &dyn Fn(&ContentRef, f32, f32) -> Result<render::VectorField, String>,
    ) -> Result<render::ParticleEmitter, String> {
        let mut em = render::ParticleEmitter::default();
        macro_rules! set {
            ($($input:ident => $target:ident),* $(,)?) => { $(if let Some(v) = self.$input { em.$target = v; })* };
        }
        set!(max_particles => max_particles, spread => spread_degrees, rate => rate,
            duration => duration, looping => looping, lifetime => lifetime, speed => speed,
            size => size, color => color, rotation => rotation_degrees,
            angular_velocity => angular_velocity_degrees, drag => drag, wind => wind,
            force_fields => force_fields, soft_distance => soft_distance, lit => lit,
            roughness => roughness, distortion => distortion, glow => glow, round => round,
            inherit_velocity => inherit_velocity);
        if let Some(v) = self.gravity {
            em.gravity = v.into();
        }
        if let Some(shape) = &self.shape {
            em.shape = match shape {
                EmitterShape::Point => render::EmitterShape::Point,
                EmitterShape::Sphere { radius, surface } => render::EmitterShape::Sphere {
                    radius: *radius,
                    surface: surface.unwrap_or(false),
                },
                EmitterShape::Cone {
                    angle_degrees,
                    radius,
                } => render::EmitterShape::Cone {
                    angle_degrees: *angle_degrees,
                    radius: *radius,
                },
                EmitterShape::Box { half_extents } => render::EmitterShape::Box {
                    half_extents: (*half_extents).into(),
                },
                EmitterShape::Mesh { mesh: m } => render::EmitterShape::Mesh {
                    mesh: mesh(m, links)?,
                },
                EmitterShape::Spline { points, closed } => {
                    if !(2..=render::MAX_SPLINE_POINTS).contains(&points.len()) {
                        return Err("a spline has 2..=8 points".into());
                    }
                    let mut native = [glam::Vec3::ZERO; render::MAX_SPLINE_POINTS];
                    for (target, p) in native.iter_mut().zip(points) {
                        *target = (*p).into();
                    }
                    render::EmitterShape::Spline {
                        points: native,
                        count: points.len() as u32,
                        closed: closed.unwrap_or(false),
                    }
                }
            };
        }
        if let Some(v) = &self.bursts {
            em.bursts = v
                .iter()
                .map(|b| render::Burst {
                    time: b.time,
                    count: b.count,
                    cycles: b.cycles.unwrap_or(1),
                    interval: b.interval.unwrap_or(0.0),
                })
                .collect();
        }
        if let Some(v) = &self.size_over_life {
            em.size_over_life = curve(&v.0)?;
        }
        if let Some(v) = &self.color_over_life {
            em.color_over_life = color_curve(&v.0)?;
        }
        if let Some(v) = &self.random_color_over_life {
            em.random_color_over_life = Some(color_curve(&v.0)?);
        }
        for (input, target) in [
            (&self.height_over_life, &mut em.height_over_life),
            (&self.speed_over_life, &mut em.speed_over_life),
            (&self.spin_over_life, &mut em.spin_over_life),
            (&self.rate_over_time, &mut em.rate_over_time),
        ] {
            *target = input.as_ref().map(|v| curve(&v.0)).transpose()?;
        }
        if let Some(v) = &self.noise {
            em.noise.strength = v.strength;
            if let Some(f) = v.frequency {
                em.noise.frequency = f;
            }
            if let Some(s) = v.scroll {
                em.noise.scroll = s;
            }
        }
        if let Some(v) = &self.texture {
            em.texture = (links.texture)(&v.id)?;
        }
        if let Some(v) = &self.sheet {
            em.sheet = render::SpriteSheet {
                columns: v.columns,
                rows: v.rows,
                frames: v.frames.unwrap_or(0),
                mode: v.mode.as_ref().map(sheet_mode).unwrap_or_default(),
                blend: v.blend.unwrap_or(false),
            };
        }
        if let Some(v) = &self.array {
            if self.sheet.is_some() {
                return Err("particles use a sheet or an array, not both".into());
            }
            em.array = (links.array)(&v.asset.id)?;
            em.sheet.columns = 1;
            em.sheet.rows = 1;
            em.sheet.frames = v.layers.unwrap_or(0);
            em.sheet.mode = v.mode.as_ref().map(sheet_mode).unwrap_or_default();
        }
        em.blend = self.blend.as_ref().map(blend);
        if let Some(v) = &self.space {
            em.space = match v {
                Space::World => render::SimulationSpace::World,
                Space::Local => render::SimulationSpace::Local,
            };
        }
        if let Some(v) = &self.align {
            em.align = match v {
                Align::Camera => render::ParticleAlign::Camera,
                Align::Velocity { stretch } => render::ParticleAlign::Velocity {
                    stretch: stretch.unwrap_or(0.05),
                },
                Align::Upright { axis } => render::ParticleAlign::Upright {
                    axis: (*axis).into(),
                },
                Align::Facing { normal } => render::ParticleAlign::Facing {
                    normal: (*normal).into(),
                },
            };
            let axis = match em.align {
                render::ParticleAlign::Upright { axis } => Some(axis),
                render::ParticleAlign::Facing { normal } => Some(normal),
                _ => None,
            };
            if axis.is_some_and(|a| a.length_squared() < 1e-12) {
                return Err("particle alignment needs a nonzero axis".into());
            }
        }
        if let Some(v) = &self.collision {
            let d = render::ParticleCollision::default();
            em.collision = Some(render::ParticleCollision {
                bounce: v.bounce.unwrap_or(d.bounce),
                friction: v.friction.unwrap_or(d.friction),
                die: v.die.unwrap_or(false),
                against: match v.against.as_ref() {
                    None => d.against,
                    Some(CollisionTarget::Depth) => render::CollisionTarget::Depth,
                    Some(CollisionTarget::Colliders) => render::CollisionTarget::Colliders,
                },
            });
        }
        if let Some(v) = &self.render {
            em.render = match v {
                ParticleRender::Billboard => render::ParticleRender::Billboard,
                ParticleRender::Sprite { layer } => render::ParticleRender::Sprite {
                    layer: layer.unwrap_or(0),
                },
                ParticleRender::Mesh { mesh: m } => render::ParticleRender::Mesh {
                    mesh: mesh(m, links)?,
                },
            };
        }
        if self.array.is_some() && !matches!(em.render, render::ParticleRender::Sprite { .. }) {
            return Err("a texture array needs sprite rendering".into());
        }
        if let Some(v) = &self.events {
            for event in v {
                let event = match event {
                    ParticleEvent::Birth => render::ParticleEventKind::Birth,
                    ParticleEvent::Death => render::ParticleEventKind::Death,
                    ParticleEvent::Collision => render::ParticleEventKind::Collision,
                };
                if !em.events.contains(&event) {
                    em.events.push(event);
                }
            }
        }
        if let Some(v) = &self.sub_emitters {
            if v.len() > render::MAX_SUB_EMITTERS {
                return Err("at most 4 sub-emitters".into());
            }
            for s in v {
                let trigger = match s.trigger {
                    Trigger::Birth => render::SubTrigger::Event(render::ParticleEventKind::Birth),
                    Trigger::Death => render::SubTrigger::Event(render::ParticleEventKind::Death),
                    Trigger::Collision => {
                        render::SubTrigger::Event(render::ParticleEventKind::Collision)
                    }
                    Trigger::Life => render::SubTrigger::Life,
                };
                let count = if trigger == render::SubTrigger::Life {
                    if s.count.is_some() {
                        return Err(
                            "a life sub-emitter runs its own emission, without count".into()
                        );
                    }
                    0
                } else {
                    s.count
                        .filter(|n| (1..=4096).contains(n))
                        .ok_or("an event sub-emitter needs count 1..=4096")?
                };
                if let Some(emitter) = (links.emitter)(&s.emitter.id)? {
                    em.sub_emitters.push(render::SubEmitter {
                        trigger,
                        emitter,
                        count,
                        inherit_velocity: s.inherit_velocity.unwrap_or(0.0),
                    });
                }
            }
        }
        if let Some(v) = &self.effect {
            em.effect = (links.effect)(&v.id)?;
        }
        if let Some(v) = &self.trail {
            em.trail = Some(render::ParticleTrail {
                lifetime: v.lifetime,
                width: v.width.unwrap_or(1.0),
                head: v.head.unwrap_or(true),
            });
        }
        if let Some(v) = &self.attractors {
            em.attractors = v
                .iter()
                .map(|a| render::Attractor {
                    position: a.position.into(),
                    strength: a.strength,
                    radius: a.radius.unwrap_or(0.0),
                })
                .collect();
        }
        if let Some(v) = &self.field {
            em.field = Some(match v {
                ParticleField::Curl {
                    strength,
                    frequency,
                    scroll,
                } => render::VectorField::Curl {
                    strength: *strength,
                    frequency: frequency.unwrap_or(0.5),
                    scroll: scroll.unwrap_or(0.2),
                },
                ParticleField::Asset {
                    asset,
                    strength,
                    size,
                } => field(asset, *strength, *size)?,
                ParticleField::Grid {
                    cells,
                    size,
                    strength,
                    vectors,
                } => render::VectorField::Grid {
                    cells: *cells,
                    size: *size,
                    strength: *strength,
                    vectors: vectors.iter().map(|v| (*v).into()).collect(),
                },
            });
        }
        em.validate()?;
        Ok(em)
    }
}

impl VfxInput {
    /// Resolve timeline parameters and native element fields. Resource IDs stay
    /// canonical until the content graph binds them to actual cooked outputs.
    pub fn build(&self) -> Result<og_bundle::vfx::Vfx, String> {
        use og_bundle::vfx as native;
        let mut params = Vec::<native::VfxParam>::new();
        for p in self.params.iter().flatten() {
            if p.name.is_empty() || params.iter().any(|n| n.name == p.name) {
                return Err(format!("empty or duplicate vfx parameter '{}'", p.name));
            }
            params.push(native::VfxParam {
                name: p.name.clone(),
                default: p.default,
            });
        }
        if params.len() > native::MAX_PARAMS {
            return Err("at most 4 vfx parameters".into());
        }
        let parameter = |name: &Option<String>| {
            name.as_ref()
                .map(|name| {
                    params
                        .iter()
                        .position(|p| &p.name == name)
                        .ok_or_else(|| format!("unknown vfx parameter '{name}'"))
                })
                .transpose()
        };
        let mut elements = Vec::new();
        for input in &self.elements {
            use VfxNode as N;
            use native::VfxKind as K;
            let mut stretch = None;
            let kind = match &input.node {
                N::Emitter { asset, seed } => K::Emitter {
                    id: asset.id.clone(),
                    seed: seed.unwrap_or(0),
                },
                N::Mesh {
                    shape,
                    effect,
                    scale,
                    spin,
                    color,
                    blend: b,
                    face_camera,
                    stretch: s,
                } => {
                    stretch = parameter(s)?;
                    let shape = match shape {
                        VfxShape::Sphere => native::VfxShape::Sphere,
                        VfxShape::Quad => native::VfxShape::Quad,
                        VfxShape::Box => native::VfxShape::Box,
                        VfxShape::Cylinder => native::VfxShape::Cylinder,
                        VfxShape::Model { asset } => native::VfxShape::Model(asset.id.clone()),
                        VfxShape::Ring { ring } => native::VfxShape::Ring(ring.as_ref().map(|r| {
                            let mut n = render::EffectRing::default();
                            macro_rules! set { ($($f:ident),*) => { $(if let Some(v) = r.$f { n.$f = v; })* }; }
                            set!(inner, band, height, start, length, segments, colors);
                            n
                        })),
                    };
                    K::Mesh {
                        shape,
                        effect: effect.id.clone(),
                        scale: scale
                            .as_ref()
                            .map(value)
                            .transpose()?
                            .unwrap_or(render::EffectValue::Constant(1.0)),
                        spin: spin.unwrap_or([0.0; 3]).into(),
                        color: color.unwrap_or([1.0; 4]),
                        blend: b
                            .as_ref()
                            .map(blend)
                            .unwrap_or(render::ParticleBlend::Additive),
                        face_camera: face_camera.unwrap_or(false),
                    }
                }
                N::Trail {
                    effect,
                    width,
                    lifetime,
                    color,
                    orbit,
                    rise,
                } => K::Trail {
                    effect: effect.as_ref().map(|r| r.id.clone()),
                    width: width.unwrap_or([0.2, 0.0]),
                    lifetime: lifetime.unwrap_or(0.4),
                    color: color.unwrap_or([1.0; 4]),
                    orbit: orbit.unwrap_or([0.0; 2]),
                    rise: rise.unwrap_or(0.0),
                },
                N::Beam {
                    effect,
                    to,
                    width,
                    color,
                    segments,
                    jitter,
                    stretch: s,
                } => {
                    stretch = parameter(s)?;
                    K::Beam {
                        effect: effect.as_ref().map(|r| r.id.clone()),
                        to: to.unwrap_or([0.0, 1.0, 0.0]).into(),
                        width: width.unwrap_or(0.2),
                        color: color.unwrap_or([1.0; 4]),
                        segments: segments.unwrap_or(1),
                        jitter: jitter.unwrap_or(0.0),
                    }
                }
                N::Light {
                    color,
                    intensity,
                    distance,
                } => K::Light {
                    color: color.unwrap_or([1.0; 3]),
                    intensity: intensity
                        .as_ref()
                        .map(value)
                        .transpose()?
                        .unwrap_or(render::EffectValue::Constant(1.0)),
                    distance: distance.unwrap_or(5.0),
                },
                N::Decal {
                    uv,
                    size,
                    fade,
                    color,
                    texture,
                    normal,
                    normal_scale,
                    emissive,
                    roughness,
                    metallic,
                    wet,
                    angle_fade,
                    turn,
                } => K::Decal {
                    uv: uv.unwrap_or([0.0, 0.0, 1.0, 1.0]),
                    size: size.unwrap_or([1.0; 3]).into(),
                    fade: fade.unwrap_or(0.0),
                    color: color.unwrap_or([1.0; 4]),
                    texture: texture.as_ref().map(|r| r.id.clone()),
                    normal: normal.as_ref().map(|r| r.id.clone()),
                    normal_scale: normal_scale.unwrap_or(1.0),
                    emissive: emissive.unwrap_or([0.0; 3]),
                    roughness: roughness.unwrap_or(-1.0),
                    metallic: metallic.unwrap_or(-1.0),
                    wet: wet.unwrap_or(false),
                    angle_fade: angle_fade.unwrap_or(75.0),
                    turn: turn.unwrap_or([0.0; 2]),
                },
                N::Glow {
                    size,
                    color,
                    stretch: s,
                } => {
                    stretch = parameter(s)?;
                    K::Glow {
                        size: size
                            .as_ref()
                            .map(value)
                            .transpose()?
                            .unwrap_or(render::EffectValue::Constant(1.0)),
                        color: color.unwrap_or([1.0; 4]),
                    }
                }
                N::Shake { trauma } => K::Shake {
                    trauma: trauma.unwrap_or(0.2),
                },
                N::Sound {
                    asset,
                    volume,
                    pitch,
                } => K::Sound {
                    id: asset.id.clone(),
                    volume: volume.unwrap_or(1.0),
                    pitch: pitch.unwrap_or([1.0; 2]),
                },
                N::Flipbook {
                    asset,
                    size,
                    color,
                    blend,
                } => K::Flipbook {
                    id: asset.id.clone(),
                    size: size.unwrap_or(1.0),
                    color: color.unwrap_or([1.0; 4]),
                    blend: match blend {
                        Some(SpriteBlend::Additive) => render::ParticleBlend::Additive,
                        _ => render::ParticleBlend::Alpha,
                    },
                },
            };
            let at = input.at.unwrap_or(0.0);
            let life = match (input.life, input.until) {
                (Some(_), Some(_)) => {
                    return Err("vfx element needs life or until, not both".into());
                }
                _ if matches!(kind, K::Shake { .. } | K::Sound { .. }) => 0.0,
                (Some(l), None) => l,
                (None, Some(until)) if until >= at => until - at,
                (None, Some(_)) => return Err("vfx until is before at".into()),
                (None, None) if matches!(kind, K::Emitter { .. } | K::Flipbook { .. }) => 0.0,
                _ => return Err(format!("a {} needs life or until", kind.name())),
            };
            elements.push(native::VfxElement {
                at,
                life,
                offset: input.offset.unwrap_or([0.0; 3]).into(),
                rotate: input.rotate.unwrap_or([0.0; 3]).into(),
                velocity: input.velocity.unwrap_or([0.0; 3]).into(),
                acceleration: input.acceleration.unwrap_or([0.0; 3]).into(),
                when: parameter(&input.when)?,
                amount: parameter(&input.amount)?,
                stretch,
                kind,
            });
        }
        if elements.len() > native::MAX_ELEMENTS {
            return Err("at most 64 vfx elements".into());
        }
        let mut events = Vec::<native::VfxEventKey>::new();
        for e in self.events.iter().flatten() {
            if e.name.is_empty() || e.name.len() > native::MAX_EVENT_NAME || e.time > self.duration
            {
                return Err(format!("invalid vfx event '{}' at {}", e.name, e.time));
            }
            events.push(native::VfxEventKey {
                time: e.time,
                name: e.name.clone(),
            });
        }
        if events.len() > native::MAX_EVENTS {
            return Err("at most 16 vfx events".into());
        }
        og_core::sort::sort_by_key(&mut events, |e| e.time.to_bits());
        let vfx = native::Vfx {
            duration: self.duration,
            looping: self.looping.unwrap_or(false),
            events,
            elements,
            params,
        };
        vfx.validate()?;
        Ok(vfx)
    }
}
