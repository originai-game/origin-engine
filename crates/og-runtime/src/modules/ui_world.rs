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
use og_ui::{EmitFrom, MetaDesc, MetaKind, ParticlesDesc, ViewportDesc, WorldOcclusion, WorldView};

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

fn v2(v: &Value, d: Vec2) -> Vec2 {
    if !v.is_object() {
        return d;
    }
    Vec2::new(f(v, "x", d.x), f(v, "y", d.y))
}

use og_core::glam::Vec2;

fn linear(theme: &Theme, v: &Value, d: [f32; 4]) -> [f32; 4] {
    paint(theme, v).map_or(d, |c| c.linear())
}

fn occlusion(d: &Value) -> WorldOcclusion {
    match d["occlusion"].as_str() {
        Some("fade") => WorldOcclusion::Fade {
            alpha: f(d, "occludedAlpha", 0.25).clamp(0.0, 1.0),
        },
        Some("hide") => WorldOcclusion::Hide,
        _ => WorldOcclusion::None,
    }
}

fn template(
    theme: &Theme,
    eng: Option<&Engine>,
    d: &Value,
) -> Result<(UiMassTemplate, UiMassSpace), String> {
    let kind = match d["kind"].as_str() {
        Some("text") => UiMassKind::Text {
            strings: d["strings"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        },
        Some("bar") => UiMassKind::Bar {
            back: linear(theme, &d["barBack"], [0.02, 0.02, 0.03, 0.85]),
            ghost: linear(theme, &d["barGhost"], [1.0, 0.9, 0.9, 1.0]),
            radius: f(d, "barRadius", 0.0).max(0.0),
        },
        Some("icon") => UiMassKind::Icon {
            frames: d["frames"]
                .as_array()
                .map(|a| {
                    let n: Vec<f32> = a
                        .iter()
                        .filter_map(|x| x.as_f64())
                        .map(|x| x as f32)
                        .collect();
                    n.as_chunks::<4>().0.to_vec()
                })
                .unwrap_or_default(),
        },
        Some("particle") => UiMassKind::Particle {
            shape: particle_shape(&d["particleShape"]),
        },
        _ => UiMassKind::Number {
            decimals: d["decimals"].as_u64().unwrap_or(0).min(6) as u32,
            plus: d["plus"].as_bool().unwrap_or(false),
            compact: d["compact"].as_bool().unwrap_or(false),
        },
    };
    let size = match kind {
        UiMassKind::Number { .. } | UiMassKind::Text { .. } => Vec2::splat(24.0),
        UiMassKind::Bar { .. } => Vec2::new(80.0, 8.0),
        UiMassKind::Icon { .. } => Vec2::splat(32.0),
        UiMassKind::Particle { .. } => Vec2::splat(8.0),
    };
    let font = d["font"]
        .as_f64()
        .and_then(|h| eng.and_then(|e| super::ui_font(e, h).ok()))
        .and_then(|f| f.handle);
    let keys = d["keys"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|k| Ok(UiMassKey {
                    t: f(k, "t", 0.0).clamp(0.0, 1.0),
                    offset: Vec2::new(f(k, "offsetX", 0.0), f(k, "offsetY", 0.0)),
                    scale: f(k, "scale", 1.0).max(0.0),
                    alpha: f(k, "alpha", 1.0).clamp(0.0, 1.0),
                    ease: match &k["ease"] {
                        Value::Null => og_tween::Ease::Linear,
                        Value::String(text) => og_tween::Ease::from_text(text).ok_or_else(|| format!("invalid mass ease '{text}': expected a GSAP name, cubic-bezier(x1,y1,x2,y2) with four finite controls and x in [0,1], or steps(n[,position]) with positive integer n and start/end/jump-start/jump-end/jump-none/jump-both (jump-none requires n > 1)"))?,
                        value => return Err(format!("mass ease {value} must be a timing string")),
                    },
                }))
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()?.unwrap_or_default();
    let t = UiMassTemplate {
        kind,
        font,
        texture: texture_of(&d["texture"]),
        size: v2(&d["size"], size),
        pivot: v2(&d["pivot"], Vec2::new(0.5, 0.5)),
        lifetime: d["lifetime"].as_u64().unwrap_or(0) as u32,
        keys,
        drift: f(d, "drift", 0.0),
        gravity: f(d, "gravity", 0.0),
        drag: d["drag"].as_bool().unwrap_or(false),
        distance: Vec3::new(
            f(d, "reference", 0.0).max(0.0),
            f(d, "minScale", 0.25).max(0.0),
            f(d, "maxScale", 4.0).max(0.0),
        ),
        occlusion: match occlusion(d) {
            WorldOcclusion::Fade { alpha } => UiOcclusion::Fade { alpha },
            WorldOcclusion::Hide => UiOcclusion::Hide,
            WorldOcclusion::None => UiOcclusion::None,
        },
        outline: paint(theme, &d["outline"])
            .map(|c| (f(d, "outlineWidth", 2.0).max(0.0), c.linear())),
    };
    let space = match d["space"].as_str() {
        Some("screen") => UiMassSpace::Screen,
        _ => UiMassSpace::World,
    };
    Ok((t, space))
}

fn particle_shape(v: &Value) -> UiParticleShape {
    match v.as_str() {
        Some("circle") => UiParticleShape::Circle,
        Some("diamond") => UiParticleShape::Diamond,
        _ => UiParticleShape::Square,
    }
}

fn instances(theme: &Theme, text: &str) -> Vec<UiMassInstance> {
    let list: Vec<Value> = serde_json::from_str(text).unwrap_or_default();
    list.iter()
        .map(|i| {
            let p = vec3(&i["position"], Vec3::ZERO);
            let vel = v2(&i["velocity"], Vec2::ZERO);
            UiMassInstance {
                position: p.to_array(),
                value: i["value"].as_f64().unwrap_or(0.0) as f32,
                color: linear(theme, &i["color"], [1.0; 4]),
                seed: i["seed"].as_u64().unwrap_or(0) as u32,
                scale: f(i, "scale", 1.0).max(0.0),
                aux: [0.0, 0.0, f(i, "ghost", 0.0), 0.0],
                velocity: [vel.x, vel.y, f(i, "spin", 0.0), f(i, "rotation", 0.0)],
                ..Default::default()
            }
        })
        .collect()
}

// ── og.ui's members ─────────────────────────────────────────────────────────

pub fn og_ui_define_mass(name: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let eng = ui_engine().and_then(|id| engine(id).ok());
    with_ui("og.ui.defineMass", |ui| {
        let theme = *ui.theme();
        match template(&theme, eng.as_deref(), &d) {
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

pub fn og_ui_particles(desc: Option<String>) {
    let d = desc_of(desc);
    with_ui("og.ui.particles", |ui| {
        let theme = *ui.theme();
        let base = ParticlesDesc::default();
        let p = ParticlesDesc {
            rate: f(&d, "rate", 0.0).max(0.0),
            burst: d["burst"].as_u64().unwrap_or(0).min(4096) as u32,
            fire: d["fire"].as_bool().unwrap_or(false),
            lifetime_ms: f(&d, "lifetimeMs", 0.0).max(0.0),
            shape: particle_shape(&d["shape"]),
            size: v2(&d["size"], base.size),
            colors: d["colors"]
                .as_array()
                .map(|a| a.iter().filter_map(|c| paint(&theme, c)).collect())
                .unwrap_or_default(),
            travel: (
                f(&d, "travelMin", base.travel.0).max(0.0),
                f(&d, "travelMax", base.travel.1).max(0.0),
            ),
            direction: f(&d, "direction", base.direction),
            spread: f(&d, "spread", base.spread).max(0.0),
            gravity: f(&d, "gravity", 0.0),
            drag: d["drag"].as_bool().unwrap_or(true),
            spin: f(&d, "spin", 0.0).max(0.0),
            align: d["align"].as_bool().unwrap_or(false),
            from: match d["from"].as_str() {
                Some("area") => EmitFrom::Area,
                Some("edges") => EmitFrom::Edges,
                _ => EmitFrom::Center,
            },
            end_scale: f(&d, "endScale", 1.0).max(0.0),
            fade: d["fade"].as_bool().unwrap_or(true),
            clip: d["clip"].as_bool().unwrap_or(true),
        };
        ui.particles(p)
    });
}

pub fn og_ui_meta(id: String, desc: Option<String>) {
    let d = desc_of(desc);
    // Read before the UI is locked (it reads the loaded materials).
    let material = crate::ui_draw::layer_material(&d);
    with_ui("og.ui.meta", |ui| {
        let theme = *ui.theme();
        let color = paint(&theme, &d["color"]).unwrap_or(og_ui::Srgba::rgb(160, 0, 0));
        let kind = match (d["kind"].as_str(), material) {
            (Some("tint"), _) => MetaKind::Tint { color },
            (Some("material"), Some((material, params))) => MetaKind::Material { material, params },
            _ => MetaKind::Vignette {
                color,
                inner: f(&d, "inner", 0.55).clamp(0.0, 1.0),
                outer: f(&d, "outer", 1.0).clamp(0.0, 1.0),
            },
        };
        ui.meta(
            id.as_str(),
            MetaDesc {
                kind,
                intensity: f(&d, "intensity", 1.0).clamp(0.0, 1.0),
                pulse_hz: f(&d, "pulseHz", 0.0).max(0.0),
                pulse_depth: f(&d, "pulseDepth", 0.5).clamp(0.0, 1.0),
            },
        )
    });
}

pub fn og_ui_scene_viewport(id: String, desc: Option<String>) -> String {
    let d = desc_of(desc);
    let texture = texture_of(&d["texture"]);
    let r = with_ui("og.ui.sceneViewport", |ui| {
        let theme = *ui.theme();
        let fov = f(&d, "fov", 50.0);
        let width = size_or(&d["width"], Size::Fixed(320.0));
        let height = size_or(&d["height"], Size::Fixed(320.0));
        // Its camera for rays: the aspect is the element's as laid out.
        let aspect = {
            let last = ui.element(id.as_str());
            last.map_or(1.0, |e| e.width / e.height.max(1.0))
        };
        let view = (fov > 0.0).then(|| {
            WorldView::look_at(
                vec3(&d["cameraPosition"], Vec3::new(0.0, 1.0, 3.0)),
                vec3(&d["cameraTarget"], Vec3::new(0.0, 1.0, 0.0)),
                Vec3::Y,
                fov,
                aspect,
                0.05,
                500.0,
            )
        });
        ui.scene_viewport(
            id.as_str(),
            ViewportDesc {
                texture,
                width,
                height,
                aspect: f(&d, "aspect", 0.0).max(0.0),
                view,
                background: paint(&theme, &d["background"]),
                radius: f(&d, "radius", 0.0),
                label: d["label"].as_str().unwrap_or_default().to_owned(),
            },
        )
    })
    .unwrap_or_default();
    let vec2_json = |v: Vec2| json!({ "x": v.x, "y": v.y });
    let v3j = |v: Vec3| json!({ "x": v.x, "y": v.y, "z": v.z });
    json!({
        "width": r.rect.width, "height": r.rect.height,
        "pixelWidth": r.pixels.0, "pixelHeight": r.pixels.1,
        "hovered": r.hovered, "pressed": r.pressed, "clicked": r.clicked,
        "drag": vec2_json(r.drag), "wheel": vec2_json(r.wheel),
        "at": r.at.map(vec2_json),
        "ray": r.ray.map(|(o, dir)| json!({ "origin": v3j(o), "direction": v3j(dir) })),
    })
    .to_string()
}
