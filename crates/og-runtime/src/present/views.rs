//! Render targets, secondary cameras, layers and custom post passes (depth
//! S8): `og.render.renderTarget`, `og.CameraView`, `og.Mirror`, the
//! `layers` masks of og.Mesh, og.Model and og.Camera, `og.load.post`,
//! `PostEffects.custom`, `og.render.setPostParam` and
//! `og.render.postErrors`, onto og-render's `targets.rs`, `views.rs` and
//! `post_custom.rs`.
//!
//! Layers are three.js masks: bit `n` is layer `n`, everything is on layer 0
//! (mask 1) by default, and a camera draws an object when `a & b != 0`, the
//! test `og.render.raycast`'s `layers` makes too (`crate::present::geometry`).

use crate::engine::engine;
use crate::host::codec::Layout;
use crate::host::wire::{FromJson, PostError, RenderTargetInfo, record};
use crate::present::extract::{Hierarchy, RenderState};
use crate::present::fx::{resource, rows};
use crate::world::assets::{handle, index_of};
use glam::{Mat4, Vec3};
use og_ecs::{Entity, QueryDesc, World};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

/// The capacity of PostEffects.custom (`List.Post.8` in the IDL): every
/// pass og-render runs.
pub(crate) const POST_SLOTS: usize = og_render::MAX_CUSTOM_POSTS;

/// The records extraction reads here.
#[derive(Default)]
pub(crate) struct ViewLayouts {
    pub camera_view: Option<Layout>,
    pub mirror: Option<Layout>,
}

impl ViewLayouts {
    pub fn from(get: impl Fn(&str) -> Option<Layout>) -> ViewLayouts {
        ViewLayouts {
            camera_view: get("og.CameraView"),
            mirror: get("og.Mirror"),
        }
    }
}

/// What a world's renderer holds of this module.
#[derive(Default)]
pub(crate) struct ViewState {
    /// Render targets by RenderTarget handle index.
    pub targets: Vec<og_render::RenderTargetHandle>,
    /// Post passes by Post handle index, and the asset each came from.
    pub posts: Vec<og_render::PostHandle>,
    pub post_paths: BTreeMap<String, usize>,
    /// The entity of each of the frame's views, by index in
    /// `RenderScene::views` (the probe's `describe` names them).
    pub entities: Vec<u64>,
}

// ── og.render.renderTarget ──────────────────────────────────────────────────

pub fn og_render_render_target(
    world: u32,
    desc: Option<String>,
) -> Result<crate::host::wire::RenderTargetInfo, String> {
    (|| -> Result<RenderTargetInfo, String> {
        let eng = engine(world)?;
        let d: crate::host::wire::RenderTargetDesc =
            crate::host::wire::arg(desc.as_deref(), "desc")?;
        let (width, height) = (d.width.max(1), d.height.max(1));
        let format = if d.format == crate::host::wire::RenderTargetFormat::Hdr {
            og_render::TargetFormat::Rgba16Float
        } else {
            og_render::TargetFormat::Rgba8
        };
        let depth = d.depth;
        let mut rs = eng.render.borrow_mut();
        let max = max_target_size(&rs);
        if width > max || height > max {
            return Err(format!(
                "a render target of {width}×{height} is past this device's largest texture side, {max} (RenderCapabilities.maxRenderTargetSize)"
            ));
        }
        let t = rs.scene.add_render_target(og_render::RenderTargetDesc {
            width,
            height,
            format,
            depth,
        });
        rs.textures.push(t.color);
        let texture = handle(rs.textures.len() - 1);
        let depth_texture = match t.depth {
            Some(d) => {
                rs.textures.push(d);
                handle(rs.textures.len() - 1)
            }
            None => 0,
        };
        rs.views.targets.push(t);
        Ok(RenderTargetInfo {
            target: handle(rs.views.targets.len() - 1),
            texture,
            depth_texture,
            width,
            height,
        })
    })()
}

/// The largest render target side: the device's, or wgpu's default
/// limit before a renderer is attached.
fn max_target_size(rs: &RenderState) -> u32 {
    rs.backend
        .as_ref()
        .and_then(|b| b.info())
        .map_or(8192, |i| i.max_texture_size)
}

/// RenderCapabilities' members of this module.
pub(crate) fn capabilities(rs: &RenderState) -> Value {
    // Every renderer draws a target's depth (og-render views.rs: from the
    // view's linear depth prepass).
    let renderer = rs.backend.as_ref().and_then(|b| b.info()).is_some();
    json!({
        "renderTargetDepth": renderer,
        "maxRenderTargetSize": max_target_size(rs),
        "maxCustomPosts": POST_SLOTS,
    })
}

// ── og.load.post, og.render.setPostParam, og.render.postErrors ─────────────

/// A cooked post pass added to the scene once per path: its load index
/// (`og.load.post`, a look's PostEffects.custom).
pub(crate) fn load_post(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().views.post_paths.get(&id) {
        return Ok(i);
    }
    let post = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle.post(&id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_post(post);
    rs.views.posts.push(h);
    let i = rs.views.posts.len() - 1;
    rs.views.post_paths.insert(id, i);
    Ok(i)
}

/// `og.load.post`: a cooked post pass, added to the scene once per path.
pub fn og_load_post(world: u32, path: String) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(handle(load_post(&eng, &path)?)))
    })()
}

pub fn og_render_set_post_param(
    world: u32,
    post: f64,
    name: String,
    value: String,
) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let value: Vec<f32> = match serde_json::from_str::<Value>(&value) {
            // Hand read: a bare number, which the IDL's List.f32 does not hold.
            Ok(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
            Ok(v @ Value::Array(_)) => crate::host::wire::value(&v, "value")?,
            _ => return Err("value is a list of numbers".into()),
        };
        let mut rs = eng.render.borrow_mut();
        let h = index_of(post)
            .and_then(|i| rs.views.posts.get(i).copied())
            .ok_or_else(|| format!("post pass {post} is not loaded in this world"))?;
        if !rs.scene.set_post_param(h, &name, &value) {
            return Err(format!("the post pass declares no parameter '{name}'"));
        }
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn og_render_post_errors(world: u32) -> Vec<crate::host::wire::PostError> {
    engine(world).ok().map_or_else(Vec::new, |eng| {
        let rs = eng.render.borrow();
        let Some(b) = rs.backend.as_ref() else {
            return Vec::new();
        };
        b.post_errors(&rs.scene)
            .into_iter()
            .filter_map(|(p, message)| {
                let i = rs.views.posts.iter().position(|h| h.0 == p)?;
                Some(PostError {
                    post: handle(i),
                    message,
                })
            })
            .collect()
    })
}

// ── extraction ──────────────────────────────────────────────────────────────

/// The `layers` of every entity of a record whose mask is not 1, read
/// straight from the column.
fn masks(world: &World, layout: Option<&Layout>, out: &mut HashMap<u64, u32>) {
    let Some(layout) = layout else {
        return;
    };
    let Ok(id) = world.id_by_name(&layout.schema) else {
        return;
    };
    let Ok(field) = world.field_by_id(id, "layers") else {
        return;
    };
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    let Ok(archetypes) = world.query(&q) else {
        return;
    };
    for a in archetypes {
        let Ok(l) = a.read::<u32>(field) else {
            continue;
        };
        for (i, e) in a.entities().iter().enumerate() {
            if l[i] != 1 {
                out.insert(e.to_bits(), l[i]);
            }
        }
    }
}

/// A handle field's value as an index.
fn index(h: u64) -> Option<usize> {
    index_of(h as f64)
}

/// The camera of an entity's world matrix: at its origin, looking down
/// its local -Z with its +Y up, as og.Camera.
// Rendering only: a view's projection.
#[allow(clippy::disallowed_methods)]
fn camera_of(m: Mat4, v: &record::CameraView) -> og_render::Camera {
    let position = m.transform_point3(Vec3::ZERO);
    let forward = m.transform_vector3(Vec3::NEG_Z).normalize_or(Vec3::NEG_Z);
    let up = m.transform_vector3(Vec3::Y).normalize_or(Vec3::Y);
    let zoom = v.zoom.max(0.01);
    og_render::Camera {
        position,
        target: position + forward,
        up,
        projection: if v.orthographic {
            og_render::Projection::Orthographic {
                height: v.height / zoom,
            }
        } else {
            let half = (v.fov.to_radians() / 2.0).tan() / zoom;
            og_render::Projection::Perspective {
                fov_y_degrees: (2.0 * half.atan()).to_degrees(),
            }
        },
        near: v.near,
        far: v.far,
        exposure: og_render::Exposure::linear(v.exposure.max(0.0)),
    }
}

/// Every component's rows in entity order (views tie on it).
fn sorted_rows(world: &World, layout: Option<&Layout>) -> Vec<(u64, Value)> {
    let mut r = rows(world, layout);
    og_core::sort::sort_by_key(&mut r, |(e, _)| Entity::from_bits(*e).index());
    r
}

/// Into the frame: each instance's and skinned instance's layer mask (by
/// the entity `keys` and the palettes name), the views of og.CameraView and
/// og.Mirror, and PostEffects' custom passes. After the camera is final.
pub(crate) fn extract(world: &World, rs: &mut RenderState, keys: &[(u64, u32)], h: &mut Hierarchy) {
    // Layers.
    let mut by_entity = HashMap::new();
    masks(world, rs.draw.mesh.as_ref(), &mut by_entity);
    masks(world, rs.draw.model.as_ref(), &mut by_entity);
    let scene = &mut rs.scene;
    scene.objects.instance_layers.clear();
    scene.objects.skinned_layers.clear();
    if !by_entity.is_empty() {
        let of = |e: &u64| by_entity.get(e).copied().unwrap_or(1);
        // Instances past the keyed ones (none today) stay on layer 0.
        scene.objects.instance_layers.extend(
            keys.iter()
                .take(scene.objects.instances.len())
                .map(|(e, _)| of(e)),
        );
        let entity_of: HashMap<u32, u64> = rs.atmo.palettes.iter().map(|(e, p)| (*p, *e)).collect();
        scene.objects.skinned_layers.extend(
            scene
                .objects
                .skinned_instances
                .iter()
                .map(|s| entity_of.get(&s.palette).map_or(1, of)),
        );
    }

    // Views.
    scene.frame.views.clear();
    rs.views.entities.clear();
    let targets = &rs.views.targets;
    let target = |h: u64| index(h).and_then(|i| targets.get(i).copied());
    for (e, v) in sorted_rows(world, rs.draw.views.camera_view.as_ref()) {
        let Ok(v) = record::CameraView::from_json(&v) else {
            continue;
        };
        if !v.active {
            continue;
        }
        let view_target = match target(v.target) {
            Some(t) => og_render::ViewTarget::Texture(t),
            None if index(v.target).is_some() => continue,
            None => og_render::ViewTarget::Viewport([
                v.viewport_x,
                v.viewport_y,
                v.viewport_width,
                v.viewport_height,
            ]),
        };
        let c = &v.clear_color;
        scene.frame.views.push(og_render::View {
            camera: camera_of(h.world_of(e), &v),
            target: view_target,
            layers: v.layers,
            order: v.order,
            clear: v.clear.then(|| Vec3::new(c.r, c.g, c.b)),
            every: v.every,
            mirror: None,
        });
        rs.views.entities.push(e);
    }
    for (e, v) in sorted_rows(world, rs.draw.views.mirror.as_ref()) {
        let Ok(v) = record::Mirror::from_json(&v) else {
            continue;
        };
        if !v.active {
            continue;
        }
        let Some(t) = target(v.target) else {
            continue;
        };
        scene.frame.views.push(og_render::View {
            camera: scene.frame.camera,
            target: og_render::ViewTarget::Texture(t),
            layers: v.layers,
            order: 0,
            clear: None,
            every: v.every,
            mirror: Some(og_render::MirrorQuad {
                transform: h.world_of(e),
                width: v.width,
                height: v.height,
            }),
        });
        rs.views.entities.push(e);
    }

    // PostEffects.custom, in order, a 0 (or a pass not loaded) skipped.
    scene.look.custom_post.clear();
    if let Some(p) = resource(world, rs.draw.fx.post.as_ref())
        && let Ok(p) = record::PostEffects::from_json(&p)
    {
        for &v in p.custom.iter().take(POST_SLOTS) {
            if let Some(h) = index(v).and_then(|i| rs.views.posts.get(i).copied()) {
                scene.look.custom_post.push(h);
            }
        }
    }
}
