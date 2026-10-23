//! Render targets, secondary cameras, layers and custom post passes (depth
//! S8): `og.render.renderTarget`, `og.CameraView`, `og.Mirror`, the
//! `layers` masks of og.Mesh, og.Model and og.Camera, `og.load.post`,
//! `PostEffects.custom`, `og.render.setPostParam` and
//! `og.render.postErrors`, onto og-render's `targets.rs`, `views.rs` and
//! `post_custom.rs`.
//!
//! Layers are three.js masks: bit `n` is layer `n`, everything is on layer 0
//! (mask 1) by default, and a camera draws an object when `a & b != 0`, the
//! test `og.render.raycast`'s `layers` makes too (`super::geometry`).

use super::assets::{handle, index_of};
use super::codec::Layout;
use super::engine::engine;
use super::extract::{Hierarchy, RenderState};
use super::fx::{color, f, on, resource, rows};
use super::{ok, ok_void, parse, refuse};
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

pub fn og_render_render_target(world: u32, desc: Option<String>) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let d = parse(desc.as_deref())?;
        let side = |k: &str| d[k].as_f64().map_or(512, |x| x.max(1.0) as u32);
        let (width, height) = (side("width"), side("height"));
        let format = match &d["format"] {
            Value::String(s) if s == "hdr" => og_render::TargetFormat::Rgba16Float,
            Value::Number(n) if n.as_u64() == Some(1) => og_render::TargetFormat::Rgba16Float,
            _ => og_render::TargetFormat::Rgba8,
        };
        let depth = on(&d, "depth", false);
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
        Ok(json!({
            "target": handle(rs.views.targets.len() - 1),
            "texture": texture,
            "depthTexture": depth_texture,
            "width": width,
            "height": height,
        }))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
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
    // Every tier draws a target's depth (og-render views.rs: from the
    // view's linear depth prepass, which WebGL2 reads too).
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
pub(crate) fn load_post(eng: &super::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
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
pub fn og_load_post(world: u32, path: String) -> String {
    let r = (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(handle(load_post(&eng, &path)?)))
    })();
    match r {
        Ok(v) => ok(v),
        Err(e) => refuse(e),
    }
}

pub fn og_render_set_post_param(world: u32, post: f64, name: String, value: String) -> String {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let value: Vec<f32> = match serde_json::from_str::<Value>(&value) {
            Ok(Value::Array(a)) => a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect(),
            Ok(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
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
        Ok(()) => ok_void(),
        Err(e) => refuse(e),
    }
}

pub fn og_render_post_errors(world: u32) -> String {
    let errors = engine(world).ok().map_or_else(Vec::new, |eng| {
        let rs = eng.render.borrow();
        let Some(b) = rs.backend.as_ref() else {
            return Vec::new();
        };
        b.post_errors(&rs.scene)
            .into_iter()
            .filter_map(|(p, message)| {
                let i = rs.views.posts.iter().position(|h| h.0 == p)?;
                Some(json!({ "post": handle(i), "message": message }))
            })
            .collect()
    });
    Value::Array(errors).to_string()
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

/// A handle field's record value as an index.
fn index(v: &Value) -> Option<usize> {
    v.as_f64().and_then(index_of)
}

/// The camera of an entity's world matrix: at its origin, looking down
/// its local -Z with its +Y up, as og.Camera.
fn camera_of(m: Mat4, v: &Value) -> og_render::Camera {
    let position = m.transform_point3(Vec3::ZERO);
    let forward = m.transform_vector3(Vec3::NEG_Z).normalize_or(Vec3::NEG_Z);
    let up = m.transform_vector3(Vec3::Y).normalize_or(Vec3::Y);
    let zoom = f(v, "zoom", 1.0).max(0.01);
    og_render::Camera {
        position,
        target: position + forward,
        up,
        projection: if on(v, "orthographic", false) {
            og_render::Projection::Orthographic {
                height: f(v, "height", 10.0) / zoom,
            }
        } else {
            let half = (f(v, "fov", 50.0).to_radians() / 2.0).tan() / zoom;
            og_render::Projection::Perspective {
                fov_y_degrees: (2.0 * half.atan()).to_degrees(),
            }
        },
        near: f(v, "near", 0.1),
        far: f(v, "far", 2000.0),
        exposure: og_render::Exposure::linear(f(v, "exposure", 1.0).max(0.0)),
    }
}

fn mask(v: &Value) -> u32 {
    v["layers"].as_u64().map_or(1, |x| x as u32)
}

fn every(v: &Value) -> u32 {
    v["every"].as_u64().map_or(1, |x| x as u32)
}

/// Every component's rows in entity order (views tie on it).
fn sorted_rows(world: &World, layout: Option<&Layout>) -> Vec<(u64, Value)> {
    let mut r = rows(world, layout);
    r.sort_by_key(|(e, _)| Entity::from_bits(*e).index());
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
    scene.instance_layers.clear();
    scene.skinned_layers.clear();
    if !by_entity.is_empty() {
        let of = |e: &u64| by_entity.get(e).copied().unwrap_or(1);
        // Instances past the keyed ones (none today) stay on layer 0.
        scene
            .instance_layers
            .extend(keys.iter().take(scene.instances.len()).map(|(e, _)| of(e)));
        let entity_of: HashMap<u32, u64> = rs.atmo.palettes.iter().map(|(e, p)| (*p, *e)).collect();
        scene.skinned_layers.extend(
            scene
                .skinned_instances
                .iter()
                .map(|s| entity_of.get(&s.palette).map_or(1, of)),
        );
    }

    // Views.
    scene.views.clear();
    rs.views.entities.clear();
    let targets = &rs.views.targets;
    let target = |v: &Value| index(&v["target"]).and_then(|i| targets.get(i).copied());
    for (e, v) in sorted_rows(world, rs.draw.views.camera_view.as_ref()) {
        if !on(&v, "active", true) {
            continue;
        }
        let view_target = match target(&v) {
            Some(t) => og_render::ViewTarget::Texture(t),
            None if index(&v["target"]).is_some() => continue,
            None => og_render::ViewTarget::Viewport([
                f(&v, "viewportX", 0.0),
                f(&v, "viewportY", 0.0),
                f(&v, "viewportWidth", 1.0),
                f(&v, "viewportHeight", 1.0),
            ]),
        };
        scene.views.push(og_render::View {
            camera: camera_of(h.world_of(e), &v),
            target: view_target,
            layers: mask(&v),
            order: v["order"].as_i64().unwrap_or(0) as i32,
            clear: on(&v, "clear", false).then(|| color(&v["clearColor"], Vec3::ZERO)),
            every: every(&v),
            mirror: None,
        });
        rs.views.entities.push(e);
    }
    for (e, v) in sorted_rows(world, rs.draw.views.mirror.as_ref()) {
        if !on(&v, "active", true) {
            continue;
        }
        let Some(t) = target(&v) else {
            continue;
        };
        scene.views.push(og_render::View {
            camera: scene.camera,
            target: og_render::ViewTarget::Texture(t),
            layers: mask(&v),
            order: 0,
            clear: None,
            every: every(&v),
            mirror: Some(og_render::MirrorQuad {
                transform: h.world_of(e),
                width: f(&v, "width", 1.0),
                height: f(&v, "height", 1.0),
            }),
        });
        rs.views.entities.push(e);
    }

    // PostEffects.custom, in order, a 0 (or a pass not loaded) skipped.
    scene.custom_post.clear();
    if let Some(p) = resource(world, rs.draw.fx.post.as_ref()) {
        let passes = p["custom"].as_array().map_or(&[][..], Vec::as_slice);
        for v in passes.iter().take(POST_SLOTS) {
            if let Some(h) = index(v).and_then(|i| rs.views.posts.get(i).copied()) {
                scene.custom_post.push(h);
            }
        }
    }
}
