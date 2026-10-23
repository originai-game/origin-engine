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
//! (Metal, Vulkan, lavapipe, WebGPU, WebGL2) and without one, needs no
//! readback (so it answers synchronously on the web too), and never feeds
//! back into the frame. Left out: terrains, scattered copies, particles,
//! strips, text and the overlay (the UI is described from its own tree),
//! morph targets (the bind shape is drawn), and alpha (a cut-out leaf
//! covers its whole triangle).

use crate::frame::camera_frame;
use crate::scene::{Camera, MeshData, MeshHandle, Projection, RenderScene};
use glam::{Mat4, Vec2, Vec3, Vec4};
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
}

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
}

/// Renders the id pass of `scene` at `width`×`height` pixels.
pub fn render_ids(scene: &RenderScene, sources: &IdSources, width: u32, height: u32) -> IdFrame {
    let (width, height) = (width.max(1), height.max(1));
    let (vp, view) = view_proj(scene, width as f32 / height as f32);
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
        // The main camera's view: an instance off its layers is not drawn there.
        let shown = crate::views::layer_of(&scene.instance_layers, i) & scene.camera_layers != 0;
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
        });
    }
    for (i, inst) in scene.skinned_instances.iter().enumerate() {
        let id = sources.skinned.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.skinned_layers, i) & scene.camera_layers != 0;
        let Some(data) = (id != 0 && shown).then(|| scene.mesh(inst.mesh)).flatten() else {
            continue;
        };
        let palette = scene.joint_palettes.get(inst.palette as usize);
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
        });
    }
    for (i, s) in scene.sprites.iter().enumerate() {
        let id = sources.sprites.get(i).copied().unwrap_or(0);
        let shown = crate::views::layer_of(&scene.sprite_layers, i) & scene.camera_layers != 0;
        if id == 0 || !shown || s.size.x == 0.0 || s.size.y == 0.0 {
            continue;
        }
        let (sin, cos) = s.rotation.sin_cos();
        let corner = |u: f32, v: f32| {
            let local = Vec2::new((u - s.anchor.x) * s.size.x, (v - s.anchor.y) * s.size.y);
            let r = Vec2::new(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
            s.position + r.extend(0.0)
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
        });
    }
    // Meshes first, grouped by id (an id's coverage drawn alone is counted
    // once per pixel); then sprites in their draw order.
    items.sort_by_key(|item| {
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
        drawn: BTreeMap::new(),
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
        for tri in item.indices.as_chunks::<3>().0 {
            let [a, b, c] = tri.map(|k| clip.get(k as usize).copied());
            if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                frame.triangle(item.id, item.sprite, [a, b, c]);
            }
        }
    }

    // The objects: what shows of each id, with its bounds and depth.
    let forward = -view.row(2).truncate();
    let mut objects: BTreeMap<u32, IdObject> = frame
        .drawn
        .iter()
        .map(|(&id, &(count, b))| {
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
        })
        .collect();
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
    drawn: BTreeMap<u32, (u32, [u32; 4])>,
}

impl Raster {
    /// Clips a clip-space triangle to the near plane (z >= 0) and fills it.
    fn triangle(&mut self, id: u32, sprite: bool, tri: [Vec4; 3]) {
        let mut poly: Vec<Vec4> = Vec::with_capacity(4);
        for i in 0..3 {
            let (a, b) = (tri[i], tri[(i + 1) % 3]);
            let (da, db) = (a.z, b.z);
            if da >= 0.0 {
                poly.push(a);
            }
            if (da >= 0.0) != (db >= 0.0) {
                let t = da / (da - db);
                poly.push(a + (b - a) * t);
            }
        }
        if poly.len() < 3 || poly.iter().any(|p| p.w <= 1e-6) {
            return;
        }
        let screen: Vec<Vec3> = poly
            .iter()
            .map(|p| {
                let ndc = p.truncate() / p.w;
                Vec3::new(
                    (ndc.x * 0.5 + 0.5) * self.width as f32,
                    (0.5 - ndc.y * 0.5) * self.height as f32,
                    ndc.z,
                )
            })
            .collect();
        for k in 1..screen.len() - 1 {
            self.fill(id, sprite, [screen[0], screen[k], screen[k + 1]]);
        }
    }

    /// Fills a screen-space triangle (x, y in pixels, z the NDC depth):
    /// pixels whose centre is inside, either winding.
    fn fill(&mut self, id: u32, sprite: bool, [a, b, c]: [Vec3; 3]) {
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
        let f = render_ids(&scene, &ids, 200, 100);
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

    #[test]
    fn deterministic_and_order_free() {
        let (scene, ids) = scene_with_boxes();
        let a = render_ids(&scene, &ids, 160, 90);
        let (swapped, ids2) = scene_in_order(true);
        let b = render_ids(&swapped, &ids2, 160, 90);
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
        let f = render_ids(&scene, &ids, 100, 100);
        // At the middle both overlap: the higher layer (3) is on top.
        assert_eq!(f.id_at(50, 50), 3);
        assert!(f.object(4).expect("shows").occlusion() > 0.2);
        assert_eq!(f.id_at(2, 2), 0, "the corner shows nothing");
    }
}
