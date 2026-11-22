//! The probe's scene view (`describe`, `pick`): the frame of the current
//! tick extracted as the renderer draws it, rendered through og-render's
//! entity-id pass (`render_ids`, on the CPU, a quarter of the frame's
//! pixels), and the ids mapped back to entities. Extraction records which
//! entity each drawable came from ([`DrawIds`]); nothing here needs a GPU,
//! so a host without a renderer describes the same frame.
//!
//! The UI draws over the finished frame: a pixel of the pass whose centre
//! lies under an element with an opaque background ([`hide_under`]) shows
//! the UI, not the entity, so the entity's visible pixels, box, point and
//! occlusion count the UI as something in front of it.

use crate::present::extract::{RenderState, extract};
use og_core::IdMap;
use og_ecs::{Entity, World};
use og_probe::{Pick, SceneView, Seen};
#[cfg(not(feature = "publish"))]
use og_probe::{SeenView, Shown};
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

/// The entity of each drawable extraction made, parallel to the scene's
/// lists (entity bits; 0 for none).
#[derive(Clone, Debug, Default)]
pub(crate) struct DrawIds {
    /// Per `scene.instances` (the instance keys extraction makes for motion).
    pub instances: Vec<u64>,
    /// `(sprite index, entity)` for og.Sprite rows.
    pub sprites: Vec<(u32, u64)>,
    /// Per `scene.terrains`.
    pub terrains: Vec<u64>,
    /// Per `scene.terrain_stamps`.
    pub stamps: Vec<u64>,
    /// Per `scene.grass`.
    pub grass: Vec<u64>,
}

/// The owners of one drawn frame's drawables, parallel to its cull record
/// (og-render's `CullRecord`): kept past the extraction that makes `ids`
/// again, until the record is read ([`keep_drawn`]).
#[cfg(not(feature = "publish"))]
#[derive(Clone, Debug, Default)]
pub(crate) struct CullIds {
    frame: u64,
    /// Per scene instance.
    instances: Vec<u64>,
    /// Per skinned instance: the entity its palette was made for.
    skinned: Vec<u64>,
    /// `(sprite index, entity)` for og.Sprite rows.
    sprites: Vec<(u32, u64)>,
    /// What lays itself out around the camera (terrains, stamps, grass):
    /// drawn when extracted.
    laid_out: Vec<u64>,
    /// Models drawn at a level: those with no instance of their own drew
    /// as an impostor card or a crowd.
    levels: Vec<u64>,
    /// Models an HLOD proxy drew in place of.
    hlod: Vec<u64>,
}

#[cfg(not(feature = "publish"))]
impl CullIds {
    fn of(rs: &RenderState, frame: u64) -> CullIds {
        let by_palette: IdMap<u32, u64> =
            IdMap::from_iter(rs.atmo.palettes.iter().map(|(&e, &p)| (p, e)));
        let laid_out = (rs.ids.terrains.iter())
            .chain(&rs.ids.stamps)
            .chain(&rs.ids.grass)
            .copied()
            .collect();
        CullIds {
            frame,
            instances: rs.ids.instances.clone(),
            skinned: (rs.scene.objects.skinned_instances.iter())
                .map(|s| by_palette.get(&s.palette).copied().unwrap_or(0))
                .collect(),
            sprites: rs.ids.sprites.clone(),
            laid_out,
            levels: rs.lod_drawn.keys().copied().collect(),
            hlod: rs.hlod.replaced.keys().copied().collect(),
        }
    }
}

/// Keeps the owners of the frame drawn last ([`CullIds`]) before `ids`
/// describe another: a probe call's extraction, or a frame whose record
/// lands frames later (the web's readback).
#[cfg(not(feature = "publish"))]
pub(crate) fn keep_drawn(rs: &mut RenderState) {
    /// As many frames as the renderer keeps waiting for their words.
    const KEPT: usize = 4;
    let Some(frame) = rs.drawn_frame.take() else {
        return;
    };
    if rs.cull_ids.iter().any(|i| i.frame == frame) {
        return;
    }
    if rs.cull_ids.len() == KEPT {
        rs.cull_ids.pop_front();
    }
    let ids = CullIds::of(rs, frame);
    rs.cull_ids.push_back(ids);
}

pub(crate) struct RuntimeView {
    pub render: Rc<RefCell<RenderState>>,
    #[cfg_attr(feature = "publish", allow(dead_code))]
    pub engine: u32,
}

/// One id pass over the current tick: the frame, the entities by id, the
/// CSS pixels per pass pixel.
#[cfg_attr(feature = "publish", allow(dead_code))]
struct Pass {
    frame: og_render::IdFrame,
    entities: Vec<u64>,
    scale: (f32, f32),
    camera: serde_json::Value,
    /// The render origin less the world's zero: what takes a point of the
    /// pass (drawn from the render origin) back into the world.
    back: glam::Vec3,
    /// Each secondary camera's pass: its entity, whether a mirror, the
    /// image's pixels and its id frame (ids as `entities`').
    views: Vec<(u64, bool, (u32, u32), og_render::IdFrame)>,
    /// The main camera's eye and unit forward, from the render origin.
    eye: glam::Vec3,
    forward: glam::Vec3,
}

/// What projecting a skeleton's joints through the main pass needs.
#[cfg(not(feature = "publish"))]
struct JointView<'a> {
    back: glam::Vec3,
    eye: glam::Vec3,
    forward: glam::Vec3,
    /// The UI's opaque rects, in pass pixels.
    cover: &'a [[f32; 4]],
}

impl RuntimeView {
    /// The main camera's id pass and, for `describe` (`pick` reads the
    /// main camera's pass alone), each secondary camera's and the camera.
    fn pass(&self, world: &World, describe: bool) -> Option<Pass> {
        let mut rs = self.render.try_borrow_mut().ok()?;
        rs.terrain.settle = true;
        #[cfg(not(feature = "publish"))]
        keep_drawn(&mut rs);
        extract(world, &mut rs);
        let (css_w, css_h) = crate::present::fx::viewport(&rs);
        let (fw, fh) = match rs.backend.as_ref().map(|b| b.size()) {
            Some((w, h)) if w > 0 && h > 0 => (w, h),
            _ => (css_w.round() as u32, css_h.round() as u32),
        };
        let (pw, ph) = og_render::id_pass_size(fw, fh);
        // Entities by id (1-based), in the order they are first met.
        let mut entities: Vec<u64> = Vec::new();
        let mut ids: IdMap<u64, u32> = IdMap::new();
        let mut id_of = |e: u64| -> u32 {
            if e == 0 || !world.is_alive(Entity::from_bits(e)) {
                return 0;
            }
            *ids.entry(e).or_insert_with(|| {
                entities.push(e);
                entities.len() as u32
            })
        };
        let scene = &rs.scene;
        let mut sources = og_render::IdSources {
            instances: rs
                .ids
                .instances
                .iter()
                .take(scene.objects.instances.len())
                .map(|&e| id_of(e))
                .collect(),
            ..Default::default()
        };
        // A skinned instance's entity: the one its palette was made for.
        let by_palette: IdMap<u32, u64> =
            IdMap::from_iter(rs.atmo.palettes.iter().map(|(&e, &p)| (p, e)));
        sources.skinned = scene
            .objects
            .skinned_instances
            .iter()
            .map(|s| by_palette.get(&s.palette).map_or(0, |&e| id_of(e)))
            .collect();
        sources.sprites = vec![0; scene.sprite.sprites.len()];
        for &(i, e) in &rs.ids.sprites {
            if let Some(slot) = sources.sprites.get_mut(i as usize) {
                *slot = id_of(e);
            }
        }
        sources.terrains = rs.ids.terrains.iter().map(|&e| id_of(e)).collect();
        sources.stamps = rs.ids.stamps.iter().map(|&e| id_of(e)).collect();
        sources.grass = rs.ids.grass.iter().map(|&e| id_of(e)).collect();
        let heights = crate::bind::terrain::id_heights(&rs);
        let frame = og_render::render_ids(scene, &sources, &heights, pw, ph);
        // Each view from its own camera on its own layers, at a quarter of
        // its image's pixels as the main pass.
        let views = (if describe {
            og_render::view_id_cameras(scene, (fw, fh))
        } else {
            Vec::new()
        })
        .into_iter()
        .filter_map(|(i, camera, (w, h))| {
            let e = *rs.views.entities.get(i)?;
            let mirror = scene.frame.views[i].mirror.is_some();
            let (vw, vh) = og_render::id_pass_size(w, h);
            let f = og_render::render_ids_from(scene, &sources, &heights, vw, vh, &camera);
            Some((e, mirror, (w, h), f))
        })
        .collect();
        let c = &scene.frame.camera;
        // In the world: the frame is drawn from its render origin.
        let back = scene.frame.origin - rs.zero;
        let camera = if !describe {
            serde_json::Value::Null
        } else {
            let (position, target) = (c.position + back, c.target + back);
            json!({
            "position": [position.x, position.y, position.z],
            "target": [target.x, target.y, target.z],
            "up": [c.up.x, c.up.y, c.up.z],
            "projection": og_render::projection_name(c),
            "fov": match c.projection { og_render::Projection::Perspective { fov_y_degrees } => json!(fov_y_degrees), _ => json!(null) },
            "height": match c.projection { og_render::Projection::Orthographic { height } => json!(height), _ => json!(null) },
            "near": c.near,
            "far": c.far,
            })
        };
        Some(Pass {
            frame,
            entities,
            scale: (css_w / pw as f32, css_h / ph as f32),
            camera,
            back,
            views,
            eye: c.position,
            forward: (c.target - c.position).normalize_or_zero(),
        })
    }

    #[cfg(not(feature = "publish"))]
    /// The entities an id frame shows, its values scaled by `scale` (pixels
    /// of the image per pass pixel).
    fn shown(
        &self,
        world: &World,
        frame: &og_render::IdFrame,
        entities: &[u64],
        (sx, sy): (f32, f32),
        joints: Option<&JointView>,
    ) -> Vec<Shown> {
        let area = (frame.width * frame.height).max(1) as f32;
        let r = |b: [u32; 4]| {
            [
                b[0] as f32 * sx,
                b[1] as f32 * sy,
                (b[2] - b[0]) as f32 * sx,
                (b[3] - b[1]) as f32 * sy,
            ]
        };
        frame
            .objects
            .iter()
            .filter(|o| o.visible > 0)
            .filter_map(|o| {
                let e = *entities.get(o.id as usize - 1)?;
                Some(Shown {
                    entity: Entity::from_bits(e),
                    rect: r(o.bounds),
                    drawn: r(o.drawn_bounds),
                    point: [
                        (o.point[0] as f32 + 0.5) * sx,
                        (o.point[1] as f32 + 0.5) * sy,
                    ],
                    depth: o.depth,
                    coverage: o.visible as f32 / area,
                    occlusion: o.occlusion(),
                    labels: self.labels(world, e),
                    joints: joints
                        .map(|v| self.joints(world, e, o.id, frame, (sx, sy), v))
                        .unwrap_or_default(),
                })
            })
            .collect()
    }

    #[cfg(not(feature = "publish"))]
    /// An og.Skeleton's skin joints as the last tick posed them, projected
    /// through the pass's camera: named by humanoid role where
    /// og.anim.humanoid recognises the rig, else by node name; visible when
    /// on the surface, in front, under no opaque UI and behind nothing
    /// else the pass drew (its own surface does not hide it).
    fn joints(
        &self,
        world: &World,
        e: u64,
        id: u32,
        frame: &og_render::IdFrame,
        (sx, sy): (f32, f32),
        v: &JointView,
    ) -> Vec<og_probe::ShownJoint> {
        use og_ecs::transform::{Parent, Transform};
        let entity = Entity::from_bits(e);
        let Ok(skeleton) = world.get::<og_anim::Skeleton>(entity) else {
            return Vec::new();
        };
        let Ok(rs) = self.render.try_borrow() else {
            return Vec::new();
        };
        let Some(anim) = rs.fx.anim.as_ref() else {
            return Vec::new();
        };
        let (Some(asset), Some(out)) = (anim.skeleton(skeleton.asset), anim.output(entity)) else {
            return Vec::new();
        };
        let model = og_ecs::transform::world_matrix(
            entity,
            &IdMap::new(),
            |p| {
                world
                    .get::<Parent>(p)
                    .ok()
                    .map(|p| Entity::from_bits(p.entity))
            },
            |p| world.get::<Transform>(p).ok(),
        );
        let humanoid = og_anim::HumanoidMap::detect(&asset);
        let nodes: Vec<u32> = match asset.skins.get(skeleton.skin as usize) {
            Some(skin) => skin.joints.clone(),
            None => (0..asset.nodes.len() as u32).collect(),
        };
        let (w, h) = (frame.width as f32, frame.height as f32);
        nodes
            .into_iter()
            .filter_map(|n| {
                let posed = out.nodes.get(n as usize)?;
                let at = model.transform_point3(posed.w_axis.truncate()) - v.back;
                let role = humanoid.nodes.iter().position(|&m| m == n);
                let name = match role {
                    Some(b) => og_anim::HUMAN_BONES[b].to_string(),
                    None => asset.nodes[n as usize].name.clone(),
                };
                let clip = frame.view_proj * at.extend(1.0);
                let ndc = clip.truncate() / clip.w;
                let (px, py) = ((ndc.x + 1.0) / 2.0 * w, (1.0 - ndc.y) / 2.0 * h);
                let inside = clip.w > 0.0
                    && px >= 0.0
                    && py >= 0.0
                    && px < w
                    && py < h
                    && (0.0..=1.0).contains(&ndc.z);
                let covered = v
                    .cover
                    .iter()
                    .any(|r| px >= r[0] && px < r[0] + r[2] && py >= r[1] && py < r[1] + r[3]);
                let k = (py as u32 * frame.width + px as u32) as usize;
                let behind =
                    inside && frame.ids[k] != 0 && frame.ids[k] != id && frame.depth[k] < ndc.z;
                Some(og_probe::ShownJoint {
                    name,
                    humanoid: role.is_some(),
                    point: [px * sx, py * sy],
                    depth: (at - v.eye).dot(v.forward),
                    visible: inside && !covered && !behind,
                })
            })
            .collect()
    }

    #[cfg(not(feature = "publish"))]
    /// What draws an entity: `model` (and its asset's path), `mesh`,
    /// `terrain`, `terrainStamp`, `grass`, `skinned`, `sprite`.
    fn labels(&self, world: &World, e: u64) -> Vec<String> {
        let entity = Entity::from_bits(e);
        let mut labels = Vec::new();
        let has = |name: &str| {
            world
                .id_by_name(name)
                .is_ok_and(|id| world.get_bytes(entity, id).is_ok())
        };
        if has("og.Model") {
            labels.push("model".to_string());
            if let Ok(eng) = crate::engine::engine(self.engine)
                && let Some(layout) = eng
                    .render
                    .try_borrow()
                    .ok()
                    .and_then(|rs| rs.draw.model.clone())
                && let Ok(id) = world.id_by_name(&layout.schema)
                && let Ok(bytes) = world.get_bytes(entity, id)
                && let Ok(model) = layout.read::<crate::host::wire::record::Model>(&bytes)
                && let Some(index) = crate::world::assets::index_of(model.asset as f64)
                && let Some(entry) = eng.assets.borrow().entries.get(index)
            {
                labels.push(entry.path.clone());
            }
        }
        if has("og.Mesh") {
            labels.push("mesh".into());
        }
        if has("og.Terrain") {
            labels.push("terrain".into());
        }
        if has("og.TerrainStamp") {
            labels.push("terrainStamp".into());
        }
        if has("og.Grass") {
            labels.push("grass".into());
        }
        if self
            .render
            .try_borrow()
            .is_ok_and(|rs| rs.atmo.palettes.contains_key(&e))
        {
            labels.push("skinned".into());
        }
        if has("og.Sprite") {
            labels.push("sprite".into());
        }
        labels
    }
}

impl SceneView for RuntimeView {
    /// A publish build serves no `describe`: nothing describes the frame,
    /// so the views' id passes and what reads the passes for it are not
    /// built in (`pick` keeps the main camera's).
    #[cfg(feature = "publish")]
    fn seen(&mut self, _world: &World, _cover: &[[f32; 4]]) -> Option<Seen> {
        None
    }

    #[cfg(not(feature = "publish"))]
    fn seen(&mut self, world: &World, cover: &[[f32; 4]]) -> Option<Seen> {
        let mut pass = self.pass(world, true)?;
        // The UI's opaque rects in pass pixels.
        let cover: Vec<[f32; 4]> = cover
            .iter()
            .map(|r| {
                [
                    r[0] / pass.scale.0,
                    r[1] / pass.scale.1,
                    r[2] / pass.scale.0,
                    r[3] / pass.scale.1,
                ]
            })
            .collect();
        hide_under(&mut pass.frame, &cover);
        let joint_view = JointView {
            back: pass.back,
            eye: pass.eye,
            forward: pass.forward,
            cover: &cover,
        };
        let entities = self.shown(
            world,
            &pass.frame,
            &pass.entities,
            pass.scale,
            Some(&joint_view),
        );
        let views = pass
            .views
            .iter()
            .map(|(e, mirror, size, frame)| {
                let scale = (
                    size.0 as f32 / frame.width as f32,
                    size.1 as f32 / frame.height as f32,
                );
                SeenView {
                    entity: Entity::from_bits(*e),
                    mirror: *mirror,
                    size: *size,
                    entities: self.shown(world, frame, &pass.entities, scale, None),
                }
            })
            .collect();
        Some(Seen {
            pass: (pass.frame.width, pass.frame.height),
            camera: pass.camera,
            entities,
            views,
        })
    }

    /// The main camera's cull record of the last frame drawn (og-render's
    /// `Renderer::cull_record`, what the frame's own culls decided), per
    /// entity: its instances, skinned instances, sprites and terrains drawn
    /// and culled; a model drawn only as an impostor or a crowd counts
    /// drawn; one in an HLOD proxy's place `hlod`; a shown model whose
    /// asset has not loaded `loading`. `None` without a renderer, or before
    /// a record answers (the web's first call asks for the next frame's).
    #[cfg(not(feature = "publish"))]
    fn drawn(&self, world: &World) -> Option<Vec<og_probe::Drawn>> {
        use og_probe::DrawReason;
        use og_render::Culled;
        let mut rs = self.render.try_borrow_mut().ok()?;
        let record = rs.backend.as_mut()?.cull_record()?;
        let ids = match rs.cull_ids.iter().find(|i| i.frame == record.frame) {
            Some(ids) => ids.clone(),
            None if rs.drawn_frame == Some(record.frame) => CullIds::of(&rs, record.frame),
            None => return None,
        };
        // Per entity, its drawables by what the cull made of them.
        const KINDS: usize = Culled::Empty as usize + 1;
        let mut tally: std::collections::BTreeMap<u64, [u32; KINDS]> = Default::default();
        let mut add = |e: u64, c: Culled| {
            if e != 0 {
                tally.entry(e).or_default()[c as usize] += 1;
            }
        };
        for (&e, &c) in ids.instances.iter().zip(&record.instances) {
            add(e, c);
        }
        for (&e, &c) in ids.skinned.iter().zip(&record.skinned) {
            add(e, c);
        }
        for &(i, e) in &ids.sprites {
            if let Some(&c) = record.sprites.get(i as usize) {
                add(e, c);
            }
        }
        for &e in &ids.laid_out {
            add(e, Culled::Drawn);
        }
        // Models drawn as an impostor card or a crowd: no instance of their own.
        for &e in &ids.levels {
            tally.entry(e).or_insert_with(|| {
                let mut t = [0; KINDS];
                t[Culled::Drawn as usize] = 1;
                t
            });
        }
        let verdict = |e: u64, reason: DrawReason, detail: &str, t: [u32; KINDS]| og_probe::Drawn {
            entity: Entity::from_bits(e),
            reason,
            detail: detail.into(),
            drawn: t[Culled::Drawn as usize],
            culled: t[Culled::Layer as usize]
                + t[Culled::OutsideView as usize]
                + t[Culled::Occluded as usize],
        };
        let mut out: Vec<og_probe::Drawn> = Vec::with_capacity(tally.len());
        for (&e, &t) in &tally {
            // The first that holds, from what shows to what has nothing.
            let (reason, detail) = [
                (Culled::Drawn, DrawReason::Drawn, ""),
                (Culled::Occluded, DrawReason::Occluded, ""),
                (Culled::OutsideView, DrawReason::OutsideView, ""),
                (Culled::Layer, DrawReason::Layer, ""),
                (Culled::FadingIn, DrawReason::FadingIn, ""),
                (Culled::Lines, DrawReason::Untracked, "wireframe"),
            ]
            .into_iter()
            .find(|&(c, ..)| t[c as usize] > 0)
            // Only meshes not loaded or empty.
            .map_or((DrawReason::Loading, ""), |(_, r, d)| (r, d));
            out.push(verdict(e, reason, detail, t));
        }
        for &e in &ids.hlod {
            if !tally.contains_key(&e) {
                out.push(verdict(e, DrawReason::Hlod, "", [0; KINDS]));
            }
        }
        // A shown model whose parts have not loaded, or whose textures are
        // still on their way (it is held back rather than drawn bare).
        if let Some(layout) = rs.draw.model.clone()
            && let Ok(id) = world.id_by_name(&layout.schema)
            && let Ok(eng) = crate::engine::engine(self.engine)
        {
            for e in world.entities() {
                let bits = e.to_bits();
                if tally.contains_key(&bits) || rs.hlod.replaced.contains_key(&bits) {
                    continue;
                }
                let Some(model) = world
                    .get_bytes(e, id)
                    .ok()
                    .and_then(|b| layout.read::<crate::host::wire::record::Model>(&b).ok())
                else {
                    continue;
                };
                let asset = model.asset;
                if !model.visible
                    || asset == 0
                    || (rs.models.contains_key(&asset)
                        && !rs.decoded.hidden_models.contains_key(&asset))
                {
                    continue;
                }
                let path = crate::world::assets::index_of(model.asset as f64)
                    .and_then(|i| eng.assets.borrow().entries.get(i).map(|a| a.path.clone()))
                    .unwrap_or_default();
                out.push(verdict(bits, DrawReason::Loading, &path, [0; KINDS]));
            }
        }
        Some(out)
    }

    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let pass = self.pass(world, false)?;
        let (id, at) = pass.frame.at(x / pass.scale.0, y / pass.scale.1)?;
        let e = *pass.entities.get(id as usize - 1)?;
        let at = at + pass.back;
        Some(Pick {
            entity: Entity::from_bits(e),
            world: vec![at.x, at.y, at.z],
            source: None,
        })
    }
}

#[cfg(not(feature = "publish"))]
/// Hides what the UI covers: every pixel of `frame` whose centre lies in
/// one of `cover` (pass pixels, `[x, y, width, height]`) shows nothing,
/// and each object's visible pixels, bounds and point are counted again
/// from what still shows (its depth stays the nearest point the pass
/// found). Its coverage drawn alone does not change, so its occlusion
/// grows by the share the UI hides.
// Tooling only: a description's id frame.
#[allow(clippy::disallowed_methods)]
pub(crate) fn hide_under(frame: &mut og_render::IdFrame, cover: &[[f32; 4]]) {
    if cover.is_empty() {
        return;
    }
    let (w, h) = (frame.width, frame.height);
    let mut changed = false;
    for y in 0..h {
        let cy = y as f32 + 0.5;
        for x in 0..w {
            let cx = x as f32 + 0.5;
            let k = (y * w + x) as usize;
            if frame.ids[k] != 0
                && cover
                    .iter()
                    .any(|r| cx >= r[0] && cx < r[0] + r[2] && cy >= r[1] && cy < r[1] + r[3])
            {
                frame.ids[k] = 0;
                frame.depth[k] = 1.0;
                changed = true;
            }
        }
    }
    if !changed {
        return;
    }
    for o in &mut frame.objects {
        o.visible = 0;
        o.bounds = [u32::MAX, u32::MAX, 0, 0];
    }
    for y in 0..h {
        for x in 0..w {
            let id = frame.ids[(y * w + x) as usize];
            if id == 0 {
                continue;
            }
            if let Ok(i) = frame.objects.binary_search_by_key(&id, |o| o.id) {
                let o = &mut frame.objects[i];
                o.visible += 1;
                o.bounds = [
                    o.bounds[0].min(x),
                    o.bounds[1].min(y),
                    o.bounds[2].max(x + 1),
                    o.bounds[3].max(y + 1),
                ];
            }
        }
    }
    for o in &mut frame.objects {
        if o.visible == 0 {
            o.bounds = [0; 4];
            o.point = [0, 0];
            o.depth = f32::INFINITY;
            continue;
        }
        // The visible pixel nearest the bounds' centre, first in row order
        // on a tie, as the pass picks it.
        let cx = (o.bounds[0] + o.bounds[2]) as f32 * 0.5;
        let cy = (o.bounds[1] + o.bounds[3]) as f32 * 0.5;
        let mut best = (f32::INFINITY, [0, 0]);
        for y in o.bounds[1]..o.bounds[3] {
            for x in o.bounds[0]..o.bounds[2] {
                if frame.ids[(y * w + x) as usize] != o.id {
                    continue;
                }
                let d = (x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2);
                if d < best.0 {
                    best = (d, [x, y]);
                }
            }
        }
        o.point = best.1;
    }
}
