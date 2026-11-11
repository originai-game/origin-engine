//! og.ui's world-space members (`docs/ui.md` §1, §3.3) and what the
//! engine hands og-ui for them each tick: the camera anchored elements
//! project with and world panels' rays start from, the poses of the
//! entities they sit on, the frame's own mass elements in the overlay, and
//! their state (spawned mass elements, particles, anchors' eased values) as
//! a state block of its own, so snapshots and replays carry it.
//!
//! The simulation never reads any of it: the UI's end reads og.Camera and
//! the entities' transforms after the game's systems, and writes nothing.

use super::*;
use og_audio::schema::{Parent, Transform};
use og_render::{
    UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate, UiOcclusion,
    UiParticleShape,
};

/// What the UI's end reads to place world-space UI.
pub(crate) struct Feed {
    camera: Option<(og_ecs::SchemaId, crate::codec::Layout)>,
}

impl Feed {
    pub(crate) fn of(world: &World) -> Feed {
        let camera = world.id_by_name("og.Camera").ok().and_then(|id| {
            let schema = world.schema(id).ok()?;
            Some((
                id,
                crate::codec::Layout::from_schema(schema, crate::codec::RecordKind::Resource),
            ))
        });
        Feed { camera }
    }

    /// The UI's end system, reading what it places with.
    pub(crate) fn desc(&self, name: &str) -> SystemDesc {
        let mut d = SystemDesc::new(name).read::<Transform>().read::<Parent>();
        if self.camera.is_some() {
            d.reads.push("og.Camera".into());
        }
        d
    }

    /// The tick's camera and the poses of the entities this frame's anchors
    /// and panels sit on, handed to the UI before it ends.
    pub(crate) fn before_end(&self, ctx: &og_ecs::SystemContext<'_>, ui: &mut Ui) {
        let vp = ui.viewport();
        let view = self.camera.as_ref().and_then(|(id, layout)| {
            let bytes = ctx.resource_bytes(*id).ok()?;
            let cam = layout.decode(&bytes);
            Some(crate::api::camera_view(
                &cam,
                vp.width.max(1.0),
                vp.height.max(1.0),
            ))
        });
        ui.set_world_view(view);
        let wanted = ui.world_entities();
        if wanted.is_empty() {
            return;
        }
        let pose = |e: Entity| {
            ctx.get::<Transform>(e)
                .ok()
                .map(|t| (t.position, t.quaternion, t.scale))
        };
        let poses: Vec<(u64, og_core::math::Pose)> = wanted
            .into_iter()
            .filter_map(|bits| {
                let e = Entity::from_bits(bits);
                if !ctx.is_alive(e) {
                    return None;
                }
                let local = pose(e)?;
                let p = og_core::math::world_pose(
                    e,
                    local,
                    |c| {
                        ctx.get::<Parent>(c)
                            .ok()
                            .map(|p| Entity::from_bits(p.entity))
                    },
                    pose,
                );
                Some((bits, p))
            })
            .collect();
        ui.set_world_poses(&poses);
    }
}

// ── reading descs ───────────────────────────────────────────────────────────

use crate::wire::{self, MassKind, MassTemplateDesc, ToJson};
use og_core::glam::Vec2;

fn linear(theme: &Theme, p: &Option<wire::Paint>, d: [f32; 4]) -> [f32; 4] {
    p.as_ref()
        .and_then(|p| paint(theme, &p.to_json()))
        .map_or(d, |c| c.linear())
}

fn template(
    theme: &Theme,
    eng: Option<&Engine>,
    d: MassTemplateDesc,
) -> Result<(UiMassTemplate, UiMassSpace), String> {
    let kind = match d.kind {
        MassKind::Text => UiMassKind::Text {
            strings: d.strings.clone(),
        },
        MassKind::Bar => UiMassKind::Bar {
            back: linear(theme, &d.bar_back, [0.02, 0.02, 0.03, 0.85]),
            ghost: linear(theme, &d.bar_ghost, [1.0, 0.9, 0.9, 1.0]),
            radius: d.bar_radius.max(0.0),
        },
        MassKind::Icon => UiMassKind::Icon {
            frames: d.frames.as_chunks::<4>().0.to_vec(),
        },
        MassKind::Particle => UiMassKind::Particle {
            shape: match d.particle_shape {
                wire::ParticleShape::Circle => UiParticleShape::Circle,
                wire::ParticleShape::Diamond => UiParticleShape::Diamond,
                wire::ParticleShape::Square => UiParticleShape::Square,
            },
        },
        MassKind::Number => UiMassKind::Number {
            decimals: d.decimals.min(6),
            plus: d.plus,
            compact: d.compact,
        },
    };
    // A size component of 0 is the kind's default.
    let size = match kind {
        UiMassKind::Number { .. } | UiMassKind::Text { .. } => Vec2::splat(24.0),
        UiMassKind::Bar { .. } => Vec2::new(80.0, 8.0),
        UiMassKind::Icon { .. } => Vec2::splat(32.0),
        UiMassKind::Particle { .. } => Vec2::splat(8.0),
    };
    let size = Vec2::select(d.size.cmpeq(Vec2::ZERO), size, d.size);
    let font = d
        .font
        .and_then(|h| eng.and_then(|e| super::ui_font(e, h as f64).ok()))
        .and_then(|f| f.handle);
    let keys = d
        .keys
        .iter()
        .map(|k| Ok(UiMassKey {
            t: k.t.clamp(0.0, 1.0),
            offset: Vec2::new(k.offset_x, k.offset_y),
            scale: k.scale.max(0.0),
            alpha: k.alpha.clamp(0.0, 1.0),
            ease: og_tween::Ease::from_text(&k.ease).ok_or_else(|| format!("invalid mass ease '{}': expected a GSAP name, cubic-bezier(x1,y1,x2,y2) with four finite controls and x in [0,1], or steps(n[,position]) with positive integer n and start/end/jump-start/jump-end/jump-none/jump-both (jump-none requires n > 1)", k.ease))?,
        }))
        .collect::<Result<Vec<_>, String>>()?;
    let t = UiMassTemplate {
        kind,
        font,
        texture: texture_of(d.texture),
        size,
        pivot: d.pivot,
        lifetime: d.lifetime,
        keys,
        drift: d.drift,
        gravity: d.gravity,
        drag: d.drag,
        distance: Vec3::new(
            d.reference.max(0.0),
            d.min_scale.max(0.0),
            d.max_scale.max(0.0),
        ),
        occlusion: match d.occlusion {
            wire::WorldOcclusion::Fade => UiOcclusion::Fade {
                alpha: d.occluded_alpha.clamp(0.0, 1.0),
            },
            wire::WorldOcclusion::Hide => UiOcclusion::Hide,
            wire::WorldOcclusion::None => UiOcclusion::None,
        },
        outline: d
            .outline
            .as_ref()
            .and_then(|p| paint(theme, &p.to_json()))
            .map(|c| (d.outline_width.max(0.0), c.linear())),
    };
    let space = match d.space {
        wire::MassSpace::Screen => UiMassSpace::Screen,
        wire::MassSpace::World => UiMassSpace::World,
    };
    Ok((t, space))
}

fn instances(theme: &Theme, text: &str) -> Vec<UiMassInstance> {
    let list: Vec<wire::MassInstance> = wire::arg(Some(text), "instances").unwrap_or_default();
    list.iter()
        .map(|i| UiMassInstance {
            position: i.position.to_array(),
            value: i.value as f32,
            color: linear(theme, &i.color, [1.0; 4]),
            seed: i.seed,
            scale: i.scale.max(0.0),
            aux: [0.0, 0.0, i.ghost, 0.0],
            velocity: [i.velocity.x, i.velocity.y, i.spin, i.rotation],
            ..Default::default()
        })
        .collect()
}

// ── og.ui's members ─────────────────────────────────────────────────────────

pub fn og_ui_define_mass(name: String, desc: Option<String>) -> String {
    let d: MassTemplateDesc = match wire::arg(desc.as_deref(), "desc") {
        Ok(d) => d,
        Err(e) => return crate::refuse(e),
    };
    let eng = ui_engine().and_then(|id| engine(id).ok());
    with_ui("og.ui.defineMass", |ui| {
        let theme = *ui.theme();
        match template(&theme, eng.as_deref(), d) {
            Ok((t, space)) => crate::ok(json!(ui.define_mass(&name, t, space).0)),
            Err(reason) => crate::refuse(reason),
        }
    })
    .unwrap_or_else(|| crate::refuse("og.ui.defineMass requires an active UI frame"))
}

pub fn og_ui_spawn_mass(template: u32, list: String) {
    with_ui("og.ui.spawnMass", |ui| {
        let theme = *ui.theme();
        for i in instances(&theme, &list) {
            ui.spawn_mass(og_ui::MassHandle(template), i);
        }
    });
}

pub fn og_ui_mass_count(template: u32) -> u32 {
    with_ui("og.ui.massCount", |ui| {
        ui.mass_count(og_ui::MassHandle(template)) as u32
    })
    .unwrap_or(0)
}
