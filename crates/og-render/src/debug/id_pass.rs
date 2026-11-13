//! The entity-id pass: which object each pixel of a frame shows, rendered
//! on the CPU from the same [`RenderScene`] the GPU draws, at a fraction of
//! the frame (the probe's `describe` and `pick`, docs/design.md §3.4).
//!
//! Every drawable that carries an id ([`IdSources`]: per instance, skinned
//! instance and world sprite) is rasterised with a depth test into an id
//! buffer and a depth buffer: meshes (skinned ones posed on the CPU by their
//! joint palette, as the vertex shader poses them), then world sprites in
//! layer order, tested against the meshes' depth without writing it, as the
//! sprite batch draws over the scene. Each object also gets its coverage
//! when drawn alone, so its occlusion is the share of that coverage another
//! object hides.
//!
//! Done on the CPU on purpose: the answer is the same on every backend
//! (Metal, Vulkan, DX12, lavapipe, WebGPU) and without one, needs no
//! readback (so it answers synchronously on the web too), and never feeds
//! back into the frame. Terrains are drawn as the GPU draws them: the
//! clipmap the renderer lays out around the main camera (its levels, the
//! ring each leaves for the finer one, the morph towards the coarser, the
//! clamp at the field's edges), each cell split along the diagonal the
//! vertex shader picks, its heights read bilinearly from the finest level
//! of the field, its holes cut per pixel where the fragment shader
//! discards them, and its stamps' meshes beside it.
//!
//! A grass field is drawn under its own id at **field fidelity, not blade
//! fidelity**: the blades are generated on the GPU each frame and never
//! read back, so the pass draws the field's coverage instead, the
//! terrain's surface lifted half the field's tallest blade, cut per pixel
//! where the field grows no blades (its layer's weight times its density
//! map under one half, a hole, or past its `distance` from the camera). A
//! pixel between two blades still answers the field, and a blade's tip
//! above the half height answers what is behind it. Over a streamed
//! terrain the cut reads its resident level-0 tiles (their heights,
//! control and hole bits), so the pass stays synchronous; where none is
//! resident the field grows nothing, as on the GPU.
//!
//! Left out: scattered copies, particles, strips, text and the overlay (the
//! UI is described from its own tree), morph targets (the bind shape is
//! drawn), and alpha (a cut-out leaf covers its whole triangle).

use crate::frame::camera_frame;
use crate::scene::{Camera, MeshData, MeshHandle, Projection, RenderScene};
use glam::{Mat4, Vec2, Vec3, Vec4};
use og_core::IdMap;
use std::collections::BTreeMap;

mod raster;

use raster::*;

/// The id of each drawable, parallel to the scene's lists; 0 is none (the
/// drawable is not drawn into the pass), and a shorter list gives none to
/// the drawables past its end.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdSources {
    /// Per [`RenderScene::instances`].
    pub instances: Vec<u32>,
    /// Per [`RenderScene::skinned_instances`].
    pub skinned: Vec<u32>,
    /// Per [`RenderScene::sprites`] (world sprites only).
    pub sprites: Vec<u32>,
    /// Per [`RenderScene::terrains`].
    pub terrains: Vec<u32>,
    /// Per [`RenderScene::terrain_stamps`].
    pub stamps: Vec<u32>,
    /// Per [`RenderScene::grass`]: each field's coverage (the module's
    /// notes on its fidelity).
    pub grass: Vec<u32>,
}

/// The heights of the terrains whose fields the scene does not hold (a
/// streamed terrain's tiles live on the GPU): `(its index in
/// [`RenderScene::terrains`], column, row)` to the level-0 sample there,
/// as its tile holds it (its lowest bit the hole flag of the cell at it,
/// [`crate::hole_bit`]); `None` leaves the terrain out of the pass.
pub type IdHeights<'a> = dyn Fn(usize, u32, u32) -> Option<f32> + 'a;

/// What the pass saw of one id.
#[derive(Clone, Debug, PartialEq)]
pub struct IdObject {
    pub id: u32,
    /// Pixels of the pass showing it.
    pub visible: u32,
    /// Pixels it covers drawn alone (with nothing in front of it).
    pub drawn: u32,
    /// The visible pixels' bounds, `[x0, y0, x1, y1]` in pass pixels (x1,
    /// y1 exclusive); all 0 when nothing shows.
    pub bounds: [u32; 4],
    /// The bounds of its coverage drawn alone (what shows and what is hidden).
    pub drawn_bounds: [u32; 4],
    /// The distance from the camera to its nearest visible point, in world
    /// units (along the view direction); infinite when nothing shows.
    pub depth: f32,
    /// The visible pixel nearest the centre of `bounds`, `[x, y]` in pass
    /// pixels: a point that shows it (what `pick` answers it at).
    pub point: [u32; 2],
}

impl IdObject {
    /// The share of its coverage drawn alone that something else hides,
    /// 0 to 1 (0 for an object that covers nothing).
    pub fn occlusion(&self) -> f32 {
        if self.drawn == 0 {
            0.0
        } else {
            1.0 - self.visible as f32 / self.drawn as f32
        }
    }
}

/// The pass's result.
#[derive(Clone, Debug, PartialEq)]
pub struct IdFrame {
    pub width: u32,
    pub height: u32,
    /// Per pixel, row-major from the top left: the id it shows (0 none).
    pub ids: Vec<u32>,
    /// Per pixel: the NDC depth of what it shows (0 near, 1 far; 1 for none).
    pub depth: Vec<f32>,
    /// Every id that covers a pixel drawn alone, by id.
    pub objects: Vec<IdObject>,
    /// The camera's view-projection the pass used, for unprojecting.
    pub view_proj: Mat4,
}

impl IdFrame {
    /// The id at a pass pixel (0 for none or outside).
    pub fn id_at(&self, x: u32, y: u32) -> u32 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        self.ids[(y * self.width + x) as usize]
    }

    /// What a point of the pass shows, `x`, `y` in pass pixels (fractions
    /// allowed): its id and the world point the pixel's centre sees there.
    pub fn at(&self, x: f32, y: f32) -> Option<(u32, Vec3)> {
        if !(x >= 0.0 && y >= 0.0) {
            return None;
        }
        let (px, py) = (x as u32, y as u32);
        let id = self.id_at(px, py);
        if id == 0 {
            return None;
        }
        let depth = self.depth[(py * self.width + px) as usize];
        Some((id, self.unproject(px as f32 + 0.5, py as f32 + 0.5, depth)))
    }

    /// The world point at pass coordinates and NDC depth.
    pub fn unproject(&self, x: f32, y: f32, depth: f32) -> Vec3 {
        let ndc = Vec4::new(
            x / self.width as f32 * 2.0 - 1.0,
            1.0 - y / self.height as f32 * 2.0,
            depth,
            1.0,
        );
        let p = self.view_proj.inverse() * ndc;
        p.truncate() / p.w
    }

    /// The object of an id.
    pub fn object(&self, id: u32) -> Option<&IdObject> {
        self.objects
            .binary_search_by_key(&id, |o| o.id)
            .ok()
            .map(|i| &self.objects[i])
    }
}

/// The camera a frame of `aspect` uses (with the scene's shake), as the
/// renderer builds it.
fn view_proj(scene: &RenderScene, aspect: f32) -> (Mat4, Mat4) {
    let camera: Camera = crate::shaken_camera(&scene.frame.camera, scene.frame.shake);
    let f = camera_frame(&camera, aspect, Vec2::ZERO);
    (f.view_proj, f.view)
}

/// One triangle list to draw under an id: world-space positions.
struct Item {
    id: u32,
    /// Draw order within the sprites (layer, index); meshes are all 0.
    order: (i32, usize),
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    /// A sprite: tested against the depth, never writing it.
    sprite: bool,
    /// A terrain's or a grass field's: its cut test (an index into the
    /// pass's tests) and each position's xz in the terrain's space, which
    /// the test reads per pixel.
    ground: Option<(usize, Vec<Vec2>)>,
}

/// The camera an id pass draws from: the main one ([`render_ids`]) or a
/// secondary camera's ([`view_id_cameras`]), with the layers it draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IdCamera {
    pub view_proj: Mat4,
    pub view: Mat4,
    /// Three.js layer mask: an object whose mask shares no bit is left out.
    pub layers: u32,
}

/// Each of the scene's views ([`RenderScene::views`]) as an id-pass camera,
/// whatever its `every`: (its index, the camera, the pixels it draws: its
/// target's size, or its viewport's rect of a frame of `frame` pixels). A
/// view without a live target, or a mirror seen from behind, is left out.
pub fn view_id_cameras(
    scene: &RenderScene,
    frame: (u32, u32),
) -> Vec<(usize, IdCamera, (u32, u32))> {
    let main = crate::shaken_camera(&scene.frame.camera, scene.frame.shake);
    crate::views::plan(scene, &main, frame, |_| true)
        .into_iter()
        .map(|p| {
            (
                p.index,
                IdCamera {
                    view_proj: p.frame.view_proj,
                    view: p.frame.view,
                    layers: p.layers,
                },
                p.size,
            )
        })
        .collect()
}

/// Renders the id pass of `scene` at `width`×`height` pixels from the main
/// camera (on its layers, [`RenderScene::camera_layers`]); `heights`
/// answers the streamed terrains' samples.
pub fn render_ids(
    scene: &RenderScene,
    sources: &IdSources,
    heights: &IdHeights,
    width: u32,
    height: u32,
) -> IdFrame {
    let (vp, view) = view_proj(scene, width.max(1) as f32 / height.max(1) as f32);
    let camera = IdCamera {
        view_proj: vp,
        view,
        layers: scene.frame.camera_layers,
    };
    render_ids_from(scene, sources, heights, width, height, &camera)
}

/// Renders the id pass of `scene` at `width`×`height` pixels from `camera`.
pub fn render_ids_from(
    scene: &RenderScene,
    sources: &IdSources,
    heights: &IdHeights,
    width: u32,
    height: u32,
    camera: &IdCamera,
) -> IdFrame {
    let (width, height) = (width.max(1), height.max(1));
    let (vp, view) = (camera.view_proj, camera.view);
    let mut items: Vec<Item> = Vec::new();
    let mut bounds: BTreeMap<MeshHandle, Option<(Vec3, f32)>> = BTreeMap::new();
    let mut sphere = |mesh: MeshHandle, data: &MeshData| {
        *bounds.entry(mesh).or_insert_with(|| {
            data.bounds()
                .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo).length() * 0.5))
        })
    };
    let planes = frustum(vp);
    let in_view = |center: Vec3, radius: f32| {
        planes
            .iter()
            .all(|p| p.truncate().dot(center) + p.w >= -radius)
    };
    for (i, inst) in scene.objects.instances.iter().enumerate() {
        let id = sources.instances.get(i).copied().unwrap_or(0);
        // An instance off the camera's layers is not drawn there.
        let shown = crate::views::layer_of(&scene.objects.instance_layers, i) & camera.layers != 0;
        let Some(data) = (id != 0 && shown).then(|| scene.mesh(inst.mesh)).flatten() else {
            continue;
        };
        if let Some((c, r)) = sphere(inst.mesh, data) {
            let scale = inst
                .transform
                .to_scale_rotation_translation()
                .0
                .abs()
                .max_element();
            if !in_view(inst.transform.transform_point3(c), r * scale) {
                continue;
            }
        }
        items.push(Item {
            id,
            order: (0, 0),
            positions: data
                .vertices
                .iter()
                .map(|v| inst.transform.transform_point3(Vec3::from(v.position)))
                .collect(),
            indices: data.indices.clone(),
            sprite: false,
            ground: None,
        });
    }
    for (i, inst) in scene.objects.skinned_instances.iter().enumerate() {
        let id = sources.skinned.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.objects.skinned_layers, i) & camera.layers != 0;
        let Some(data) = (id != 0 && shown).then(|| scene.mesh(inst.mesh)).flatten() else {
            continue;
        };
        // A baked clip's pose at its frame, else its palette's.
        let baked = inst
            .baked
            .and_then(|b| {
                Some(
                    scene
                        .objects
                        .baked_clips
                        .get(b.clip as usize)?
                        .palette(b.frame),
                )
            })
            .map(crate::JointPalette::from);
        let palette = match inst.baked {
            Some(_) => baked.as_ref(),
            None => scene.objects.joint_palettes.get(inst.palette as usize),
        };
        let skin = scene.skin(inst.mesh);
        let positions = data
            .vertices
            .iter()
            .enumerate()
            .map(|(k, v)| {
                let p = Vec3::from(v.position);
                let posed = match (palette, skin) {
                    (Some(palette), Some(skin)) => pose(p, k, palette, skin),
                    _ => p,
                };
                inst.transform.transform_point3(posed)
            })
            .collect();
        items.push(Item {
            id,
            order: (0, 0),
            positions,
            indices: data.indices.clone(),
            sprite: false,
            ground: None,
        });
    }
    // Terrains, laid out around the main camera as the renderer lays them
    // out for every pass of the frame, and their stamps.
    let main = crate::shaken_camera(&scene.frame.camera, scene.frame.shake);
    let mut tests: Vec<Box<CutTest>> = Vec::new();
    for (k, t) in scene.world.terrains.iter().enumerate() {
        let id = sources.terrains.get(k).copied().unwrap_or(0);
        if id == 0 || !t.visible {
            continue;
        }
        let Some(layout) = scene.terrain_layout(t.terrain) else {
            continue;
        };
        let field = scene.terrain(t.terrain);
        let raw = move |c: u32, r: u32| match field {
            Some(d) => d.heights.get((r * d.columns + c) as usize).copied(),
            None => heights(k, c, r),
        };
        let Some((positions, locals, indices)) = scene
            .terrain_clipmap(k, &terrain_view(&main, height, main.far))
            .and_then(|c| terrain_mesh(t, layout, &raw, &c, 0.0, &planes))
        else {
            continue;
        };
        let hole: Option<Box<CutTest>> = match field {
            Some(d) => d
                .holes
                .is_some()
                .then(|| Box::new(move |xz: Vec2| d.hole(xz)) as Box<CutTest>),
            None => layout.holes.then(|| {
                let (cols, rows) = (layout.columns, layout.rows);
                let cell = layout.cell();
                let lo = -layout.size * 0.5;
                Box::new(move |xz: Vec2| {
                    let s = ((xz - lo) / cell).floor().max(Vec2::ZERO);
                    let (c, r) = ((s.x as u32).min(cols - 1), (s.y as u32).min(rows - 1));
                    heights(k, c, r).is_some_and(crate::hole_bit)
                }) as Box<CutTest>
            }),
        };
        let ground = hole.map(|hole| {
            tests.push(hole);
            (tests.len() - 1, locals)
        });
        items.push(Item {
            id,
            order: (0, 0),
            positions,
            indices,
            sprite: false,
            ground,
        });
    }
    // Grass fields: their coverage over their terrains (the module's notes).
    for (i, g) in scene.world.grass.iter().enumerate() {
        let id = sources.grass.get(i).copied().unwrap_or(0);
        let Some(t) = scene
            .world
            .terrains
            .get(g.terrain)
            .filter(|t| id != 0 && t.visible)
        else {
            continue;
        };
        let grass = &g.grass;
        if !grass.usable() || grass.layer as usize >= t.layers.len().max(1) {
            continue;
        }
        let (Some(layout), Some(source)) = (
            scene.terrain_layout(t.terrain),
            scene.terrain_sources().0.get(t.terrain.index()),
        ) else {
            continue;
        };
        let k = g.terrain;
        let field = scene.terrain(t.terrain);
        let raw = |c: u32, r: u32| match field {
            Some(d) => d.heights.get((r * d.columns + c) as usize).copied(),
            None => heights(k, c, r),
        };
        let lift = grass.tallest() * 0.5;
        let far = main.far.min(grass.distance + lift);
        let Some((positions, locals, indices)) = scene
            .terrain_clipmap(g.terrain, &terrain_view(&main, height, far))
            .and_then(|c| terrain_mesh(t, layout, &raw, &c, lift, &planes))
        else {
            continue;
        };
        let painted = grass.density_map.and_then(|h| scene.texture(h));
        let (eye, reach, layer) = (main.position, grass.distance, grass.layer as usize);
        tests.push(Box::new(move |xz: Vec2| {
            // The ground the blades grow on: the field in memory, or a
            // streamed terrain's resident level-0 tiles (none resident
            // there: no blades, as on the GPU).
            let tiles;
            let ground: &dyn crate::scatter::Ground = match source {
                crate::terrain::TerrainSource::Data(d, ..) => d,
                crate::terrain::TerrainSource::Stream { tiles: t, .. } => {
                    tiles = crate::scatter::TileGround { layout, tiles: t };
                    &tiles
                }
            };
            let (Some(h), Some(false), Some(w)) =
                (ground.height(xz), ground.hole(xz), t.weights_on(ground, xz))
            else {
                return true;
            };
            let root = t.transform.transform_point3(Vec3::new(xz.x, h, xz.y));
            let density = painted.map_or(1.0, |m| red_at(m, xz / layout.size + 0.5));
            root.distance(eye) > reach || w[layer] * density < 0.5
        }));
        items.push(Item {
            id,
            order: (0, 0),
            positions,
            indices,
            sprite: false,
            ground: Some((tests.len() - 1, locals)),
        });
    }
    for (i, st) in scene.world.terrain_stamps.iter().enumerate() {
        let id = sources.stamps.get(i).copied().unwrap_or(0);
        let drawn = scene
            .world
            .terrains
            .get(st.terrain)
            .is_some_and(|t| t.visible);
        let Some(data) = (id != 0 && drawn).then(|| scene.mesh(st.mesh)).flatten() else {
            continue;
        };
        items.push(Item {
            id,
            order: (0, 0),
            positions: data
                .vertices
                .iter()
                .map(|v| st.transform.transform_point3(Vec3::from(v.position)))
                .collect(),
            indices: data.indices.clone(),
            sprite: false,
            ground: None,
        });
    }
    for (i, s) in scene.sprite.sprites.iter().enumerate() {
        let id = sources.sprites.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.sprite.sprite_layers, i) & camera.layers != 0;
        if id == 0 || !shown || s.size.x == 0.0 || s.size.y == 0.0 {
            continue;
        }
        let (sin, cos) = s.rotation.sin_cos();
        // A billboard faces this camera, as the sprite shader turns it.
        let facing = crate::sprites::facing_of(&scene.sprite.sprite_facing, i);
        let (x, y) =
            crate::sprites::face_axes(facing, view.row(0).truncate(), view.row(1).truncate());
        let corner = |u: f32, v: f32| {
            let local = Vec2::new((u - s.anchor.x) * s.size.x, (v - s.anchor.y) * s.size.y);
            let r = Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
            if facing == crate::SpriteFacing::None {
                s.position + r.extend(0.0)
            } else {
                s.position + x * r.x + y * r.y
            }
        };
        items.push(Item {
            id,
            order: (s.layer, i),
            positions: vec![
                corner(0.0, 0.0),
                corner(1.0, 0.0),
                corner(1.0, 1.0),
                corner(0.0, 1.0),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            sprite: true,
            ground: None,
        });
    }
    // Meshes first, grouped by id (an id's coverage drawn alone is counted
    // once per pixel); then sprites in their draw order.
    og_core::sort::sort_by_key(&mut items, |item| {
        (
            item.sprite,
            if item.sprite { 0 } else { item.id },
            item.order,
        )
    });

    let n = (width * height) as usize;
    let mut frame = Raster {
        width,
        height,
        ids: vec![0; n],
        depth: vec![1.0; n],
        mesh_depth: Vec::new(),
        stamp: vec![0; n],
        group: 0,
        drawn: IdMap::new(),
    };
    let mut last: Option<(bool, u32)> = None;
    for item in &items {
        if last != Some((item.sprite, item.id)) || item.sprite {
            frame.group += 1;
            last = Some((item.sprite, item.id));
        }
        if item.sprite && frame.mesh_depth.is_empty() {
            frame.mesh_depth = frame.depth.clone();
        }
        let clip: Vec<Vec4> = item.positions.iter().map(|p| vp * p.extend(1.0)).collect();
        let ground = item
            .ground
            .as_ref()
            .map(|(k, locals)| (&*tests[*k], locals));
        for tri in item.indices.as_chunks::<3>().0 {
            let [a, b, c] = tri.map(|k| clip.get(k as usize).copied());
            let (Some(a), Some(b), Some(c)) = (a, b, c) else {
                continue;
            };
            let cut = ground.and_then(|(hole, locals)| {
                let [p, q, r] = tri.map(|k| locals.get(k as usize).copied());
                Some(([p?, q?, r?], hole))
            });
            frame.triangle(item.id, item.sprite, [a, b, c], cut);
        }
    }

    // The objects: what shows of each id, with its bounds and depth.
    let forward = -view.row(2).truncate();
    let mut objects: IdMap<u32, IdObject> =
        IdMap::from_iter(frame.drawn.iter().map(|(&id, &(count, b))| {
            (
                id,
                IdObject {
                    id,
                    visible: 0,
                    drawn: count,
                    bounds: [0; 4],
                    drawn_bounds: b,
                    depth: f32::INFINITY,
                    point: [0, 0],
                },
            )
        }));
    let ivp = vp.inverse();
    let eye = view.inverse().w_axis.truncate();
    for y in 0..height {
        for x in 0..width {
            let k = (y * width + x) as usize;
            let id = frame.ids[k];
            let Some(o) = objects.get_mut(&id) else {
                continue;
            };
            o.visible += 1;
            if o.visible == 1 {
                o.bounds = [x, y, x + 1, y + 1];
            } else {
                o.bounds = [
                    o.bounds[0].min(x),
                    o.bounds[1].min(y),
                    o.bounds[2].max(x + 1),
                    o.bounds[3].max(y + 1),
                ];
            }
            let ndc = Vec4::new(
                (x as f32 + 0.5) / width as f32 * 2.0 - 1.0,
                1.0 - (y as f32 + 0.5) / height as f32 * 2.0,
                frame.depth[k],
                1.0,
            );
            let p = ivp * ndc;
            let p = p.truncate() / p.w;
            let d = (p - eye).dot(forward);
            if d < o.depth {
                o.depth = d;
            }
        }
    }
    for o in objects.values_mut() {
        if o.visible == 0 {
            continue;
        }
        // The visible pixel nearest the bounds' centre (first in row order
        // on a tie): inside the object even when its middle is a hole.
        let cx = (o.bounds[0] + o.bounds[2]) as f32 * 0.5;
        let cy = (o.bounds[1] + o.bounds[3]) as f32 * 0.5;
        let mut best = (f32::INFINITY, [0, 0]);
        for y in o.bounds[1]..o.bounds[3] {
            for x in o.bounds[0]..o.bounds[2] {
                if frame.ids[(y * width + x) as usize] != o.id {
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
    IdFrame {
        width,
        height,
        ids: frame.ids,
        depth: frame.depth,
        objects: objects.into_values().collect(),
        view_proj: vp,
    }
}

/// A terrain's triangles as its clipmap draws them around `eye` (the main
/// camera's position, `far` its far plane; `terrain_vertex.wgsl`
/// `terrain_local_vertex`, `terrain::level_vertices`): world positions,
/// each one's xz in the terrain's space, and indices. `raw` answers the
/// field's level-0 samples. `lift` raises the copy that many metres along
/// world y (0: the terrain itself; a grass field's coverage is the surface
/// lifted). A cell whose corners all lie outside one plane of `planes` is
/// left out; `None` when a sample is unknown.
fn terrain_mesh(
    t: &crate::Terrain,
    layout: crate::TerrainLayout,
    raw: &dyn Fn(u32, u32) -> Option<f32>,
    clipmap: &crate::TerrainClipmap,
    lift: f32,
    planes: &[Vec4; 6],
) -> Option<(Vec<Vec3>, Vec<Vec2>, Vec<u32>)> {
    let (cols, rows) = (layout.columns.max(1), layout.rows.max(1));
    let top = Vec2::new(cols as f32 - 1.0, rows as f32 - 1.0);
    // The field's height at field coordinates `s`: between samples on
    // the triangle of the cell's split (terrain_vertex.wgsl field_height,
    // og_core's heightfield_zigzag).
    let height = |s: Vec2| -> Option<f32> {
        let s = s.clamp(Vec2::ZERO, top);
        let i = s.floor();
        let f = s - i;
        let (c0, r0) = (i.x as u32, i.y as u32);
        let (c1, r1) = ((c0 + 1).min(cols - 1), (r0 + 1).min(rows - 1));
        let (h00, h10) = (raw(c0, r0)?, raw(c1, r0)?);
        let (h01, h11) = (raw(c0, r1)?, raw(c1, r1)?);
        Some(crate::terrain::split_height([h00, h10, h01, h11], f))
    };
    let n = clipmap.ring;
    let (mut positions, mut locals, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    for level in &clipmap.levels {
        let corners = crate::terrain::level_vertices(clipmap, level, layout, &height)?;
        let world: Vec<Vec3> = corners
            .iter()
            .map(|p| t.transform.transform_point3(*p) + Vec3::Y * lift)
            .collect();
        let at = |x: u32, z: u32| (z * (n + 1) + x) as usize;
        for z in 0..n {
            for x in 0..n {
                if crate::terrain::in_hole(n, level.hole, x, z) {
                    continue;
                }
                let k = [at(x, z), at(x + 1, z), at(x, z + 1), at(x + 1, z + 1)];
                let outside = planes
                    .iter()
                    .any(|p| k.iter().all(|&i| p.truncate().dot(world[i]) + p.w < 0.0));
                if outside {
                    continue;
                }
                // The vertex shader's fan: four triangles to the centre,
                // whose height carries the cell's diagonal.
                for i in crate::terrain::cell_fan(n, x, z) {
                    let i = i as usize;
                    indices.push(positions.len() as u32);
                    positions.push(world[i]);
                    locals.push(Vec2::new(corners[i].x, corners[i].z));
                }
            }
        }
    }
    Some((positions, locals, indices))
}

/// A vertex posed by its four joints, as the skinning vertex shader does.
fn pose(p: Vec3, k: usize, palette: &crate::JointPalette, skin: &crate::SkinData) -> Vec3 {
    let (Some(joints), Some(weights)) = (skin.joints.get(k), skin.weights.get(k)) else {
        return p;
    };
    let total: f32 = weights.iter().sum();
    if total <= 0.0 {
        return p;
    }
    let mut out = Vec3::ZERO;
    for (j, w) in joints.iter().zip(weights) {
        if let Some(m) = palette.matrices.get(*j as usize) {
            out += m.transform_point3(p) * (*w / total);
        }
    }
    out
}

/// The six planes of a view-projection with depth 0..1, `n · p + d >= 0` inside.
fn frustum(m: Mat4) -> [Vec4; 6] {
    let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
    [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|p| p / p.truncate().length().max(1e-20))
}

/// The red channel of an RGBA8 map at `uv` (0..1 over it, clamped),
/// bilinear, linear (a painted grass map, as the GPU samples it).
fn red_at(m: &crate::TextureData, uv: Vec2) -> f32 {
    let (w, h) = (m.width.max(1), m.height.max(1));
    let texel = |x: i64, y: i64| {
        let (x, y) = (x.clamp(0, w as i64 - 1), y.clamp(0, h as i64 - 1));
        let v = m
            .rgba
            .get(((y * w as i64 + x) * 4) as usize)
            .copied()
            .unwrap_or(255) as f32
            / 255.0;
        match m.color_space {
            crate::ColorSpace::Srgb if v <= 0.04045 => v / 12.92,
            crate::ColorSpace::Srgb => ((v + 0.055) / 1.055).powf(2.4),
            crate::ColorSpace::Linear => v,
        }
    };
    let s = uv.clamp(Vec2::ZERO, Vec2::ONE) * Vec2::new(w as f32, h as f32) - 0.5;
    let i = s.floor();
    let f = s - i;
    let (x, y) = (i.x as i64, i.y as i64);
    let lo = texel(x, y) + (texel(x + 1, y) - texel(x, y)) * f.x;
    let hi = texel(x, y + 1) + (texel(x + 1, y + 1) - texel(x, y + 1)) * f.x;
    lo + (hi - lo) * f.y
}

/// The view a frame drawn by `camera` lays its terrains out for, seen
/// from an id pass `height` pixels tall ([`id_pass_size`]: half the
/// frame's), out to `far`.
fn terrain_view(camera: &Camera, height: u32, far: f32) -> crate::TerrainView {
    crate::TerrainView {
        far,
        ..crate::TerrainView::of(camera, height * 2)
    }
}

/// The aspect-preserving id pass size for a frame: a quarter of its pixels
/// (half its width and half its height), at least 1×1.
pub fn id_pass_size(width: u32, height: u32) -> (u32, u32) {
    (width.div_ceil(2).max(1), height.div_ceil(2).max(1))
}

/// Projection kind of a camera as the probe reports it.
pub fn projection_name(camera: &Camera) -> &'static str {
    match camera.projection {
        Projection::Perspective { .. } => "perspective",
        Projection::Orthographic { .. } => "orthographic",
    }
}

#[cfg(test)]
mod tests;
