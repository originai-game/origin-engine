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

use super::extract::{RenderState, extract};
use og_ecs::{Entity, World};
use og_probe::{Pick, SceneView, Seen, SeenView, Shown};
use serde_json::json;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// The entity of each drawable extraction made, parallel to the scene's
/// lists (entity bits; 0 for none).
#[derive(Clone, Debug, Default)]
pub(crate) struct DrawIds {
    /// Per `scene.instances` (the instance keys extraction makes for motion).
    pub instances: Vec<u64>,
    /// `(sprite index, entity)` for og.Sprite rows.
    pub sprites: Vec<(u32, u64)>,
}

pub(crate) struct RuntimeView {
    pub render: Rc<RefCell<RenderState>>,
    pub engine: u32,
}

/// One id pass over the current tick: the frame, the entities by id, the
/// CSS pixels per pass pixel.
struct Pass {
    frame: og_render::IdFrame,
    entities: Vec<u64>,
    scale: (f32, f32),
    camera: serde_json::Value,
    /// Each secondary camera's pass: its entity, whether a mirror, the
    /// image's pixels and its id frame (ids as `entities`').
    views: Vec<(u64, bool, (u32, u32), og_render::IdFrame)>,
}

impl RuntimeView {
    fn pass(&self, world: &World) -> Option<Pass> {
        let mut rs = self.render.try_borrow_mut().ok()?;
        rs.terrain.settle = true;
        extract(world, &mut rs);
        let (css_w, css_h) = super::fx::viewport(&rs);
        let (fw, fh) = match rs.backend.as_ref().map(|b| b.size()) {
            Some((w, h)) if w > 0 && h > 0 => (w, h),
            _ => (css_w.round() as u32, css_h.round() as u32),
        };
        let (pw, ph) = og_render::id_pass_size(fw, fh);
        // Entities by id (1-based), in the order they are first met.
        let mut entities: Vec<u64> = Vec::new();
        let mut ids: BTreeMap<u64, u32> = BTreeMap::new();
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
                .take(scene.instances.len())
                .map(|&e| id_of(e))
                .collect(),
            ..Default::default()
        };
        // A skinned instance's entity: the one its palette was made for.
        let by_palette: BTreeMap<u32, u64> =
            rs.atmo.palettes.iter().map(|(&e, &p)| (p, e)).collect();
        sources.skinned = scene
            .skinned_instances
            .iter()
            .map(|s| by_palette.get(&s.palette).map_or(0, |&e| id_of(e)))
            .collect();
        sources.sprites = vec![0; scene.sprites.len()];
        for &(i, e) in &rs.ids.sprites {
            if let Some(slot) = sources.sprites.get_mut(i as usize) {
                *slot = id_of(e);
            }
        }
        let frame = og_render::render_ids(scene, &sources, pw, ph);
        // Each view from its own camera on its own layers, at a quarter of
        // its image's pixels as the main pass.
        let views = og_render::view_id_cameras(scene, (fw, fh))
            .into_iter()
            .filter_map(|(i, camera, (w, h))| {
                let e = *rs.views.entities.get(i)?;
                let mirror = scene.views[i].mirror.is_some();
                let (vw, vh) = og_render::id_pass_size(w, h);
                let f = og_render::render_ids_from(scene, &sources, vw, vh, &camera);
                Some((e, mirror, (w, h), f))
            })
            .collect();
        let c = &scene.camera;
        let camera = json!({
            "position": [c.position.x, c.position.y, c.position.z],
            "target": [c.target.x, c.target.y, c.target.z],
            "up": [c.up.x, c.up.y, c.up.z],
            "projection": og_render::projection_name(c),
            "fov": match c.projection { og_render::Projection::Perspective { fov_y_degrees } => json!(fov_y_degrees), _ => json!(null) },
            "height": match c.projection { og_render::Projection::Orthographic { height } => json!(height), _ => json!(null) },
            "near": c.near,
            "far": c.far,
        });
        Some(Pass {
            frame,
            entities,
            scale: (css_w / pw as f32, css_h / ph as f32),
            camera,
            views,
        })
    }

    /// The entities an id frame shows, its values scaled by `scale` (pixels
    /// of the image per pass pixel).
    fn shown(
        &self,
        world: &World,
        frame: &og_render::IdFrame,
        entities: &[u64],
        (sx, sy): (f32, f32),
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
                })
            })
            .collect()
    }

    /// What draws an entity: `model` (and its asset's path), `mesh`,
    /// `skinned`, `sprite`.
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
            if let Ok(eng) = super::engine::engine(self.engine)
                && let Some(layout) = eng
                    .render
                    .try_borrow()
                    .ok()
                    .and_then(|rs| rs.draw.model.clone())
                && let Ok(id) = world.id_by_name(&layout.schema)
                && let Ok(bytes) = world.get_bytes(entity, id)
            {
                let asset = layout.decode(&bytes)["asset"].as_f64().unwrap_or(0.0);
                if let Some(index) = super::assets::index_of(asset)
                    && let Some(entry) = eng.assets.borrow().entries.get(index)
                {
                    labels.push(entry.path.clone());
                }
            }
        }
        if has("og.Mesh") {
            labels.push("mesh".into());
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
    fn seen(&mut self, world: &World, cover: &[[f32; 4]]) -> Option<Seen> {
        let mut pass = self.pass(world)?;
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
        let entities = self.shown(world, &pass.frame, &pass.entities, pass.scale);
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
                    entities: self.shown(world, frame, &pass.entities, scale),
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

    fn pick(&mut self, world: &World, x: f32, y: f32) -> Option<Pick> {
        let pass = self.pass(world)?;
        let (id, at) = pass.frame.at(x / pass.scale.0, y / pass.scale.1)?;
        let e = *pass.entities.get(id as usize - 1)?;
        Some(Pick {
            entity: Entity::from_bits(e),
            world: vec![at.x, at.y, at.z],
            source: None,
        })
    }
}

/// Hides what the UI covers: every pixel of `frame` whose centre lies in
/// one of `cover` (pass pixels, `[x, y, width, height]`) shows nothing,
/// and each object's visible pixels, bounds and point are counted again
/// from what still shows (its depth stays the nearest point the pass
/// found). Its coverage drawn alone does not change, so its occlusion
/// grows by the share the UI hides.
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
