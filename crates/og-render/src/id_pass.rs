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
//! discards them, and its stamps' meshes beside it. Left out: scattered
//! copies, grass, particles, strips, text and the overlay (the UI is
//! described from its own tree), morph targets (the bind shape is drawn),
//! and alpha (a cut-out leaf covers its whole triangle).

use crate::frame::camera_frame;
use crate::scene::{Camera, MeshData, MeshHandle, Projection, RenderScene};
use glam::{Mat4, Vec2, Vec3, Vec4};
use og_core::IdMap;
use std::collections::BTreeMap;

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
    let camera: Camera = crate::shaken_camera(&scene.camera, scene.shake);
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
    /// A terrain's: its index in the scene's terrains and each position's
    /// xz in the terrain's space, for its hole test per pixel.
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
    let main = crate::shaken_camera(&scene.camera, scene.shake);
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
        layers: scene.camera_layers,
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
    for (i, inst) in scene.instances.iter().enumerate() {
        let id = sources.instances.get(i).copied().unwrap_or(0);
        // An instance off the camera's layers is not drawn there.
        let shown = crate::views::layer_of(&scene.instance_layers, i) & camera.layers != 0;
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
    for (i, inst) in scene.skinned_instances.iter().enumerate() {
        let id = sources.skinned.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.skinned_layers, i) & camera.layers != 0;
        let Some(data) = (id != 0 && shown).then(|| scene.mesh(inst.mesh)).flatten() else {
            continue;
        };
        // A baked clip's pose at its frame, else its palette's.
        let baked = inst
            .baked
            .and_then(|b| Some(scene.baked_clips.get(b.clip as usize)?.palette(b.frame)))
            .map(crate::JointPalette::from);
        let palette = match inst.baked {
            Some(_) => baked.as_ref(),
            None => scene.joint_palettes.get(inst.palette as usize),
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
    let main = crate::shaken_camera(&scene.camera, scene.shake);
    let mut holes: Vec<Option<Box<CutTest>>> = Vec::new();
    for (k, t) in scene.terrains.iter().enumerate() {
        holes.push(None);
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
        let Some((positions, locals, indices)) =
            terrain_mesh(t, layout, &raw, main.position, main.far, 0.0, &planes)
        else {
            continue;
        };
        holes[k] = match field {
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
        items.push(Item {
            id,
            order: (0, 0),
            positions,
            indices,
            sprite: false,
            ground: Some((k, locals)),
        });
    }
    for (i, st) in scene.terrain_stamps.iter().enumerate() {
        let id = sources.stamps.get(i).copied().unwrap_or(0);
        let drawn = scene.terrains.get(st.terrain).is_some_and(|t| t.visible);
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
    for (i, s) in scene.sprites.iter().enumerate() {
        let id = sources.sprites.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.sprite_layers, i) & camera.layers != 0;
        if id == 0 || !shown || s.size.x == 0.0 || s.size.y == 0.0 {
            continue;
        }
        let (sin, cos) = s.rotation.sin_cos();
        // A billboard faces this camera, as the sprite shader turns it.
        let facing = crate::sprites::facing_of(&scene.sprite_facing, i);
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
            .and_then(|(k, locals)| Some((holes[*k].as_deref()?, locals)));
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
/// camera's position, `far` its far plane; `terrain.wgsl`
/// `terrain_grid_vertex` and `terrain_local_vertex`): world positions,
/// each one's xz in the terrain's space, and indices. `raw` answers the
/// field's level-0 samples. `lift` raises the copy that many metres along
/// world y (0: the terrain itself; a grass field's coverage is the surface
/// lifted). A cell whose corners all lie outside one plane of `planes` is
/// left out; `None` when a sample is unknown.
fn terrain_mesh(
    t: &crate::Terrain,
    layout: crate::TerrainLayout,
    raw: &dyn Fn(u32, u32) -> Option<f32>,
    eye: Vec3,
    far: f32,
    lift: f32,
    planes: &[Vec4; 6],
) -> Option<(Vec<Vec3>, Vec<Vec2>, Vec<u32>)> {
    let clipmap = t.clipmap(layout, eye, far);
    let (cols, rows) = (layout.columns.max(1), layout.rows.max(1));
    let cell = layout.cell();
    let lo = -layout.size * 0.5;
    let top = Vec2::new(cols as f32 - 1.0, rows as f32 - 1.0);
    // The finest level's height at field coordinates `s`, bilinear.
    let height = |s: Vec2| -> Option<f32> {
        let s = s.clamp(Vec2::ZERO, top);
        let i = s.floor();
        let f = s - i;
        let (c0, r0) = (i.x as u32, i.y as u32);
        let (c1, r1) = ((c0 + 1).min(cols - 1), (r0 + 1).min(rows - 1));
        let (h00, h10) = (raw(c0, r0)?, raw(c1, r0)?);
        let (h01, h11) = (raw(c0, r1)?, raw(c1, r1)?);
        let a = h00 + (h10 - h00) * f.x;
        let b = h01 + (h11 - h01) * f.x;
        Some(a + (b - a) * f.y)
    };
    let n = clipmap.ring;
    let unit = clipmap.unit;
    let cam = clipmap.camera;
    let extent = layout.size / unit;
    let (mut positions, mut locals, mut indices) = (Vec::new(), Vec::new(), Vec::new());
    let mut corners: Vec<Vec3> = Vec::with_capacity(((n + 1) * (n + 1)) as usize);
    for level in &clipmap.levels {
        let center = Vec2::new(level.center[0] as f32, level.center[1] as f32);
        let step = level.step as f32;
        let morph = if level.morph {
            clipmap.morph_width
        } else {
            0.0
        };
        corners.clear();
        for gz in 0..=n {
            for gx in 0..=n {
                let g = Vec2::new(gx as f32, gz as f32);
                let mut c = center + (g - n as f32 * 0.5) * step;
                if morph > 0.0 {
                    let d = (c - cam).abs().max_element() / step;
                    let alpha = ((d - clipmap.morph_start) / morph).clamp(0.0, 1.0);
                    let odd = g - 2.0 * (g * 0.5).floor();
                    c -= odd * (step * alpha);
                }
                let c = c.clamp(Vec2::ZERO, extent);
                let xz = lo + c * unit;
                let y = height(c * unit / cell)?;
                corners.push(Vec3::new(xz.x, y, xz.y));
            }
        }
        let world: Vec<Vec3> = corners
            .iter()
            .map(|p| t.transform.transform_point3(*p) + Vec3::Y * lift)
            .collect();
        let skip = level
            .hole
            .map(|(ox, oz)| (n / 4 + ox..3 * n / 4 + ox, n / 4 + oz..3 * n / 4 + oz));
        let at = |x: u32, z: u32| (z * (n + 1) + x) as usize;
        for z in 0..n {
            for x in 0..n {
                if skip
                    .as_ref()
                    .is_some_and(|(sx, sz)| sx.contains(&x) && sz.contains(&z))
                {
                    continue;
                }
                let k = [at(x, z), at(x + 1, z), at(x, z + 1), at(x + 1, z + 1)];
                let outside = planes
                    .iter()
                    .any(|p| k.iter().all(|&i| p.truncate().dot(world[i]) + p.w < 0.0));
                if outside {
                    continue;
                }
                let [p00, p10, p01, p11] = k.map(|i| corners[i]);
                // The vertex shader's split: the diagonal whose ends differ
                // less in height (ties: p00 to p11).
                let tris = if (p00.y - p11.y).abs() > (p10.y - p01.y).abs() {
                    [k[0], k[2], k[1], k[2], k[3], k[1]]
                } else {
                    [k[0], k[2], k[3], k[0], k[3], k[1]]
                };
                for i in tris {
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

/// Whether a pixel whose point (a terrain's xz) is this is cut out: a
/// terrain's hole.
type CutTest<'a> = dyn Fn(Vec2) -> bool + 'a;

/// A triangle's corners' points and the test that cuts a pixel out.
type Cut<'a> = ([Vec2; 3], &'a CutTest<'a>);

struct Raster {
    width: u32,
    height: u32,
    ids: Vec<u32>,
    depth: Vec<f32>,
    /// The meshes' depth, which sprites test against (taken before the first sprite).
    mesh_depth: Vec<f32>,
    /// Per pixel, the last group that covered it drawn alone.
    stamp: Vec<u32>,
    group: u32,
    /// Per id: pixels covered drawn alone, and their bounds.
    drawn: IdMap<u32, (u32, [u32; 4])>,
}

impl Raster {
    /// Clips a clip-space triangle to the near plane (z >= 0) and fills it.
    /// With `cut`, each corner's point (a terrain's xz) is carried to every
    /// pixel, perspective-correct, and a pixel whose point the test cuts
    /// out is not covered (a terrain's holes).
    fn triangle(&mut self, id: u32, sprite: bool, tri: [Vec4; 3], cut: Option<Cut>) {
        let at = |i: usize| cut.map_or(Vec2::ZERO, |(p, _)| p[i]);
        let mut poly: Vec<(Vec4, Vec2)> = Vec::with_capacity(4);
        for i in 0..3 {
            let j = (i + 1) % 3;
            let (a, b) = (tri[i], tri[j]);
            let (da, db) = (a.z, b.z);
            if da >= 0.0 {
                poly.push((a, at(i)));
            }
            if (da >= 0.0) != (db >= 0.0) {
                let t = da / (da - db);
                poly.push((a + (b - a) * t, at(i) + (at(j) - at(i)) * t));
            }
        }
        if poly.len() < 3 || poly.iter().any(|(p, _)| p.w <= 1e-6) {
            return;
        }
        let screen: Vec<(Vec3, Vec3)> = poly
            .iter()
            .map(|(p, q)| {
                let ndc = p.truncate() / p.w;
                let s = Vec3::new(
                    (ndc.x * 0.5 + 0.5) * self.width as f32,
                    (0.5 - ndc.y * 0.5) * self.height as f32,
                    ndc.z,
                );
                (s, (*q / p.w).extend(1.0 / p.w))
            })
            .collect();
        for k in 1..screen.len() - 1 {
            let [a, b, c] = [screen[0], screen[k], screen[k + 1]];
            let test = cut.map(|(_, hole)| ([a.1, b.1, c.1], hole));
            self.fill(id, sprite, [a.0, b.0, c.0], test);
        }
    }

    /// Fills a screen-space triangle (x, y in pixels, z the NDC depth):
    /// pixels whose centre is inside, either winding. `test`: per corner
    /// a point over w and 1 / w, and the test that cuts a pixel out.
    fn fill(
        &mut self,
        id: u32,
        sprite: bool,
        [a, b, c]: [Vec3; 3],
        test: Option<([Vec3; 3], &CutTest)>,
    ) {
        let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if area.abs() < 1e-12 || !area.is_finite() {
            return;
        }
        let lo_x = a.x.min(b.x).min(c.x).floor().max(0.0) as u32;
        let lo_y = a.y.min(b.y).min(c.y).floor().max(0.0) as u32;
        let hi_x = (a.x.max(b.x).max(c.x).ceil().max(0.0) as u32).min(self.width);
        let hi_y = (a.y.max(b.y).max(c.y).ceil().max(0.0) as u32).min(self.height);
        let edge =
            |p: Vec3, q: Vec3, x: f32, y: f32| (q.x - p.x) * (y - p.y) - (q.y - p.y) * (x - p.x);
        for y in lo_y..hi_y {
            for x in lo_x..hi_x {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = edge(b, c, px, py) / area;
                let w1 = edge(c, a, px, py) / area;
                let w2 = edge(a, b, px, py) / area;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let z = w0 * a.z + w1 * b.z + w2 * c.z;
                if !(0.0..=1.0).contains(&z) {
                    continue;
                }
                if let Some(([qa, qb, qc], hole)) = test {
                    let q = qa * w0 + qb * w1 + qc * w2;
                    if hole(q.truncate() / q.z) {
                        continue;
                    }
                }
                let k = (y * self.width + x) as usize;
                if self.stamp[k] != self.group {
                    self.stamp[k] = self.group;
                    let entry = self.drawn.entry(id).or_insert((0, [x, y, x + 1, y + 1]));
                    entry.0 += 1;
                    let bb = &mut entry.1;
                    *bb = [
                        bb[0].min(x),
                        bb[1].min(y),
                        bb[2].max(x + 1),
                        bb[3].max(y + 1),
                    ];
                }
                if sprite {
                    // Over the meshes in front of nothing nearer; sprites
                    // over sprites in their draw order.
                    if z <= self.mesh_depth[k] {
                        self.ids[k] = id;
                        self.depth[k] = z;
                    }
                } else if z < self.depth[k] {
                    self.ids[k] = id;
                    self.depth[k] = z;
                }
            }
        }
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
mod tests {
    use super::*;
    use crate::scene::{Material, Sprite};
    use crate::shapes;

    fn scene_with_boxes() -> (RenderScene, IdSources) {
        scene_in_order(false)
    }

    /// The slab and the cube, submitted cube first when `swapped`.
    fn scene_in_order(swapped: bool) -> (RenderScene, IdSources) {
        let mut scene = RenderScene::new();
        let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
        let m = scene.add_material(Material::default());
        scene.camera.position = Vec3::new(0.0, 0.0, 10.0);
        scene.camera.target = Vec3::ZERO;
        // A wide floor-like slab behind, a box in front of its left half.
        scene.add_instance(
            cube,
            m,
            Mat4::from_scale_rotation_translation(
                Vec3::new(6.0, 2.0, 0.2),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, 0.0, -2.0),
            ),
        );
        scene.add_instance(cube, m, Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0)));
        let mut ids = IdSources {
            instances: vec![7, 9],
            ..IdSources::default()
        };
        if swapped {
            scene.instances.swap(0, 1);
            ids.instances.swap(0, 1);
        }
        (scene, ids)
    }

    #[test]
    fn nearer_object_wins_and_hides_part_of_the_farther() {
        let (scene, ids) = scene_with_boxes();
        let f = render_ids(&scene, &ids, &|_, _, _| None, 200, 100);
        let slab = f.object(7).expect("the slab shows");
        let cube = f.object(9).expect("the cube shows");
        assert_eq!(cube.occlusion(), 0.0, "nothing is in front of the cube");
        assert!(
            slab.occlusion() > 0.05 && slab.occlusion() < 0.5,
            "the cube hides part of the slab: {}",
            slab.occlusion()
        );
        assert!(cube.depth < slab.depth, "the cube is nearer");
        // The cube is left of the centre; the slab spans both sides.
        assert!(cube.bounds[2] <= 100, "{:?}", cube.bounds);
        assert!(
            slab.bounds[0] < cube.bounds[0] && slab.bounds[2] > 120,
            "{:?}",
            slab.bounds
        );
        // The point it names shows it.
        assert_eq!(f.id_at(cube.point[0], cube.point[1]), 9);
        assert_eq!(f.id_at(slab.point[0], slab.point[1]), 7);
        // The hidden part is inside its drawn bounds, not its visible ones.
        assert_eq!(slab.drawn_bounds, slab.bounds);
        let (id, at) = f
            .at(cube.point[0] as f32 + 0.5, cube.point[1] as f32 + 0.5)
            .expect("a hit");
        assert_eq!(id, 9);
        assert!(
            (at.z - 0.5).abs() < 0.05,
            "the cube's front face is z 0.5: {at}"
        );
    }

    /// The cube on layer 1 alone: the main camera (layer 0) does not see
    /// it, a view of layers 0 and 1 does, from its own eye (from the right),
    /// and a view of layer 0 does not.
    #[test]
    fn a_view_sees_what_its_layers_admit_from_its_own_eye() {
        let (mut scene, ids) = scene_with_boxes();
        scene.instance_layers = vec![1, 2];
        let eye = |layers| crate::View {
            camera: Camera {
                position: Vec3::new(8.0, 0.0, 4.0),
                target: Vec3::new(-1.0, 0.0, 0.0),
                ..Camera::default()
            },
            target: crate::ViewTarget::Viewport([0.0, 0.0, 0.5, 1.0]),
            layers,
            ..Default::default()
        };
        scene.views = vec![eye(3), eye(1)];
        assert!(
            render_ids(&scene, &ids, &|_, _, _| None, 200, 100)
                .object(9)
                .is_none()
        );
        let cams = view_id_cameras(&scene, (400, 200));
        assert_eq!(cams.len(), 2);
        assert_eq!(cams[0].2, (200, 200), "the viewport's pixels");
        let seen = render_ids_from(&scene, &ids, &|_, _, _| None, 100, 100, &cams[0].1);
        let cube = seen.object(9).expect("the layer-1 view sees the cube");
        let slab = seen.object(7).expect("and the slab");
        // From its eye the slab's right end is the nearest point (about
        // 7 m along its view; the main camera has the slab 11.9 m away).
        assert!(slab.depth < cube.depth, "{} {}", slab.depth, cube.depth);
        assert!(slab.depth < 9.0, "{} from the view's eye", slab.depth);
        let unseen = render_ids_from(&scene, &ids, &|_, _, _| None, 100, 100, &cams[1].1);
        assert!(unseen.object(9).is_none() && unseen.object(7).is_some());
    }

    #[test]
    fn deterministic_and_order_free() {
        let (scene, ids) = scene_with_boxes();
        let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let (swapped, ids2) = scene_in_order(true);
        let b = render_ids(&swapped, &ids2, &|_, _, _| None, 160, 90);
        assert_eq!(a.ids, b.ids);
        assert_eq!(a.objects, b.objects);
    }

    #[test]
    fn sprites_draw_over_by_layer_and_behind_nearer_meshes() {
        let mut scene = RenderScene::new();
        scene.camera = Camera::orthographic_2d(Vec2::ZERO, 10.0);
        let quad = |x: f32, layer: i32| Sprite {
            position: Vec3::new(x, 0.0, 0.0),
            size: Vec2::new(4.0, 4.0),
            anchor: Vec2::new(0.5, 0.5),
            layer,
            ..Sprite::default()
        };
        scene.add_sprite(quad(-1.0, 2));
        scene.add_sprite(quad(1.0, 1));
        let ids = IdSources {
            sprites: vec![3, 4],
            ..IdSources::default()
        };
        let f = render_ids(&scene, &ids, &|_, _, _| None, 100, 100);
        // At the middle both overlap: the higher layer (3) is on top.
        assert_eq!(f.id_at(50, 50), 3);
        assert!(f.object(4).expect("shows").occlusion() > 0.2);
        assert_eq!(f.id_at(2, 2), 0, "the corner shows nothing");
    }

    /// A terrain 64 m across on a half-metre grid, its heights `h(x, z)`.
    fn field(h: impl Fn(f32, f32) -> f32) -> crate::TerrainData {
        let n = 129u32;
        let mut d = crate::TerrainData::flat(Vec2::splat(64.0), n, n);
        d.heights = (0..n * n)
            .map(|k| h((k % n) as f32 * 0.5 - 32.0, (k / n) as f32 * 0.5 - 32.0))
            .collect::<Vec<_>>()
            .into();
        d
    }

    /// A scene drawing `data` placed by `transform`, every level of it at
    /// the finest spacing (ring 256), seen from `eye` towards `target`.
    fn terrain_scene(
        data: crate::TerrainData,
        transform: Mat4,
        eye: Vec3,
        target: Vec3,
    ) -> (RenderScene, IdSources) {
        let mut scene = RenderScene::new();
        let handle = scene.add_terrain(data);
        let mut t = crate::Terrain::new(handle);
        t.transform = transform;
        t.lod.ring = 256;
        scene.terrains.push(t);
        scene.camera.position = eye;
        scene.camera.target = target;
        scene.camera.far = 500.0;
        let ids = IdSources {
            terrains: vec![5],
            ..IdSources::default()
        };
        (scene, ids)
    }

    fn hill(x: f32, z: f32) -> f32 {
        6.0 * (-(x * x + z * z) / 64.0).exp() + 0.1 * x
    }

    /// Every terrain pixel's world point lies on the field's surface (the
    /// drawn triangles, through the terrain's placement), not on a box.
    fn on_the_surface(f: &IdFrame, data: &crate::TerrainData, transform: Mat4) -> u32 {
        let to_local = transform.inverse();
        let mut n = 0;
        for y in 0..f.height {
            for x in 0..f.width {
                let Some((5, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) else {
                    continue;
                };
                let l = to_local.transform_point3(p);
                let h = data.height(Vec2::new(l.x, l.z));
                assert!(
                    (l.y - h).abs() < 0.05,
                    "pixel ({x}, {y}) sees {l} on the terrain, whose surface there is at {h}"
                );
                n += 1;
            }
        }
        n
    }

    #[test]
    fn a_terrain_is_drawn_under_its_id_on_its_surface() {
        let data = field(hill);
        let eye = Vec3::new(0.0, 14.0, 30.0);
        let (mut scene, ids) = terrain_scene(data.clone(), Mat4::IDENTITY, eye, Vec3::ZERO);
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let o = f.object(5).expect("the terrain shows");
        assert!(o.visible > 160 * 90 / 3, "{} pixels", o.visible);
        assert!(on_the_surface(&f, &data, Mat4::IDENTITY) == o.visible);
        // Hidden, it is not drawn.
        scene.terrains[0].visible = false;
        assert!(
            render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
                .object(5)
                .is_none()
        );
    }

    #[test]
    fn a_turned_and_moved_terrain_is_drawn_where_it_stands() {
        let data = field(hill);
        let transform = Mat4::from_scale_rotation_translation(
            Vec3::new(1.5, 2.0, 1.5),
            glam::Quat::from_rotation_y(1.1),
            Vec3::new(300.0, -20.0, -140.0),
        );
        let target = transform.transform_point3(Vec3::ZERO);
        let eye = target + Vec3::new(-25.0, 30.0, 35.0);
        let (scene, ids) = terrain_scene(data.clone(), transform, eye, target);
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let n = on_the_surface(&f, &data, transform);
        assert!(n > 160 * 90 / 3, "{n} pixels");
        // The peak, turned and lifted with it, is where the frame shows it.
        let peak = transform.transform_point3(Vec3::new(0.0, hill(0.0, 0.0), 0.0));
        let c = f.view_proj * peak.extend(1.0);
        let (x, y) = (
            (c.x / c.w * 0.5 + 0.5) * 160.0,
            (0.5 - c.y / c.w * 0.5) * 90.0,
        );
        let (id, p) = f.at(x, y + 1.0).expect("the peak shows");
        assert_eq!(id, 5);
        assert!(p.distance(peak) < 2.5, "{p} by the peak {peak}");
    }

    #[test]
    fn a_terrain_hides_what_its_hill_stands_before_and_shows_it_through_holes() {
        // A ridge 10 m high across x at z in (-3, 3); a box behind it.
        let ridge = |_: f32, z: f32| if z.abs() < 3.0 { 10.0 } else { 0.0 };
        let mut data = field(ridge);
        let (mut scene, mut ids) = terrain_scene(
            data.clone(),
            Mat4::IDENTITY,
            Vec3::new(0.0, 3.0, 24.0),
            Vec3::new(0.0, 2.0, -10.0),
        );
        let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(2.0)));
        let m = scene.add_material(Material::default());
        scene.add_instance(cube, m, Mat4::from_translation(Vec3::new(0.0, 2.0, -10.0)));
        ids.instances = vec![9];
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        assert!(f.object(5).is_some());
        let behind = f.object(9).expect("the box is drawn");
        assert_eq!(behind.visible, 0, "hidden behind the ridge");
        assert!(behind.drawn > 20 && behind.occlusion() == 1.0);
        // Cut the ridge out under a window 12 m wide: the box shows through.
        let (cx, cz) = data.cells();
        let mut holes = vec![0u8; (cx * cz) as usize];
        for r in 0..cz {
            for c in 0..cx {
                let (x, z) = (c as f32 * 0.5 - 31.75, r as f32 * 0.5 - 31.75);
                holes[(r * cx + c) as usize] = u8::from(x.abs() < 6.0 && z.abs() < 4.0);
            }
        }
        data.holes = Some(holes.into());
        scene.set_terrain(scene.terrains[0].terrain, data.clone());
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let seen = f.object(9).expect("the box is drawn");
        assert!(seen.visible > 20, "through the hole: {}", seen.visible);
        // No terrain pixel lies over a hole.
        for y in 0..f.height {
            for x in 0..f.width {
                if let Some((5, p)) = f.at(x as f32 + 0.5, y as f32 + 0.5) {
                    assert!(!data.hole(Vec2::new(p.x, p.z)), "({x}, {y}) at {p}");
                }
            }
        }
    }

    #[test]
    fn a_streamed_terrain_draws_as_its_field_in_memory_does() {
        let mut data = field(hill);
        let (cx, cz) = data.cells();
        // A hole in the middle, whose flag the samples carry when streamed.
        let mut holes = vec![0u8; (cx * cz) as usize];
        for r in 60..68 {
            for c in 60..68 {
                holes[(r * cx + c) as usize] = 1;
            }
        }
        data.holes = Some(holes.into());
        let eye = Vec3::new(4.0, 18.0, 26.0);
        let (mut scene, ids) = terrain_scene(data.clone(), Mat4::IDENTITY, eye, Vec3::ZERO);
        let layout = scene.terrain_layout(scene.terrains[0].terrain).unwrap();
        assert!(layout.holes);
        // The tiles' samples: the height with the hole flag of the cell at
        // it in its lowest bit.
        let raw = |_: usize, c: u32, r: u32| {
            let h = data.heights[(r * 129 + c) as usize];
            let hole = data.hole(Vec2::new(c as f32 * 0.5 - 31.75, r as f32 * 0.5 - 31.75));
            Some(f32::from_bits((h.to_bits() & !1) | u32::from(hole)))
        };
        let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        scene.terrains[0].terrain = scene.add_terrain_stream(layout, data.range());
        let b = render_ids(&scene, &ids, &raw, 160, 90);
        let differ = a.ids.iter().zip(&b.ids).filter(|(p, q)| p != q).count();
        assert!(differ <= 2, "{differ} pixels differ");
        assert!(a.object(5).unwrap().visible > 1000);
        // Unknown heights leave it out.
        assert!(
            render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
                .object(5)
                .is_none()
        );
    }

    #[test]
    fn a_coarse_ring_covers_the_terrain_without_cracks() {
        // Seen from straight above with 16 quads a level: many levels,
        // morphing into each other, each leaving its finer level's square.
        let data = field(hill);
        let (mut scene, ids) = terrain_scene(
            data.clone(),
            Mat4::IDENTITY,
            Vec3::new(3.3, 70.0, 2.1),
            Vec3::new(3.3, 0.0, 2.0),
        );
        scene.terrains[0].lod.ring = 16;
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 160);
        let corner = |x: f32, z: f32| {
            let c = f.view_proj * Vec3::new(x, hill(x, z), z).extend(1.0);
            (
                ((c.x / c.w * 0.5 + 0.5) * 160.0) as u32,
                ((0.5 - c.y / c.w * 0.5) * 160.0) as u32,
            )
        };
        let (x0, y0) = corner(-29.0, -29.0);
        let (x1, y1) = corner(29.0, 29.0);
        let mut cracks = 0;
        for y in y0.min(y1)..y0.max(y1) {
            for x in x0.min(x1)..x0.max(x1) {
                cracks += u32::from(f.id_at(x, y) != 5);
            }
        }
        assert_eq!(cracks, 0, "pixels inside the terrain that show nothing");
    }

    #[test]
    fn an_origin_shift_draws_the_same_terrain() {
        let data = field(hill);
        let at = Vec3::new(4096.0 * 3.0 + 70.0, 12.0, -4096.0 * 2.0 + 30.0);
        let (mut scene, ids) = terrain_scene(
            data,
            Mat4::from_translation(at),
            at + Vec3::new(0.0, 14.0, 30.0),
            at,
        );
        let a = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        scene.rebase(Vec3::new(4096.0 * 3.0, 0.0, -4096.0 * 2.0));
        let b = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let differ = a.ids.iter().zip(&b.ids).filter(|(p, q)| p != q).count();
        assert!(differ <= 2, "{differ} pixels differ");
        assert!(a.object(5).unwrap().visible > 1000);
    }

    #[test]
    fn a_stamp_is_drawn_under_its_own_id() {
        let data = field(|_, _| 0.0);
        let (mut scene, mut ids) =
            terrain_scene(data, Mat4::IDENTITY, Vec3::new(0.0, 6.0, 20.0), Vec3::ZERO);
        let rock = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 8.0, 6.0)));
        scene.terrain_stamps.push(crate::TerrainStampDraw {
            terrain: 0,
            mesh: rock,
            transform: Mat4::from_translation(Vec3::new(0.0, 4.0, 0.0)),
            blend: 1.0,
            cast_shadows: true,
        });
        ids.stamps = vec![8];
        let f = render_ids(&scene, &ids, &|_, _, _| None, 160, 90);
        let c = f.view_proj * Vec3::new(0.0, 3.0, 3.0).extend(1.0);
        let (x, y) = (
            (c.x / c.w * 0.5 + 0.5) * 160.0,
            (0.5 - c.y / c.w * 0.5) * 90.0,
        );
        assert_eq!(f.at(x, y).map(|a| a.0), Some(8), "the rock's face");
        assert!(
            f.object(5).unwrap().occlusion() > 0.0,
            "it hides some ground"
        );
        // Its terrain hidden, it is not drawn either.
        scene.terrains[0].visible = false;
        assert!(
            render_ids(&scene, &ids, &|_, _, _| None, 160, 90)
                .object(8)
                .is_none()
        );
    }
}
