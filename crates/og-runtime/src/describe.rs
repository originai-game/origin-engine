//! The probe's scene view (`describe`, `pick`): the frame of the current
//! tick extracted as the renderer draws it, rendered through og-render's
//! entity-id pass (`render_ids`, on the CPU, a quarter of the frame's
//! pixels), and the ids mapped back to entities. Extraction records which
//! entity each drawable came from ([`DrawIds`]); nothing here needs a GPU,
//! so a host without a renderer describes the same frame.

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
    fn seen(&mut self, world: &World) -> Option<Seen> {
        let pass = self.pass(world)?;
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
