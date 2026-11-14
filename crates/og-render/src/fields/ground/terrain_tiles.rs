use crate::ground::{Terrain, horizontal_scale};
use glam::{Vec2, Vec3};
pub use og_render_data::terrain_tiles::*;

/// Where a terrain is seen from, for [`terrain_tiles_wanted`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainView {
    /// The camera in the world.
    pub camera: Vec3,
    /// Its far plane (metres).
    pub far: f32,
    /// The angle one pixel spans at the centre of the view (radians): the
    /// vertical field of view over the target's height.
    pub pixel: f32,
    /// Metres from the camera within which the finest tiles are wanted
    /// whatever their size on screen (scattered copies stand on them).
    pub detail: f32,
}

impl TerrainView {
    /// `camera`'s view on a target `height` pixels tall (an orthographic
    /// camera's pixel taken at the distance to its target).
    pub fn of(camera: &crate::Camera, height: u32) -> TerrainView {
        match camera.projection {
            crate::Projection::Perspective { fov_y_degrees } => {
                TerrainView::perspective(camera.position, camera.far, fov_y_degrees, height)
            }
            crate::Projection::Orthographic { height: h } => TerrainView {
                camera: camera.position,
                far: camera.far,
                pixel: h / height.max(1) as f32 / camera.position.distance(camera.target).max(1.0),
                detail: 0.0,
            },
        }
    }

    /// A perspective camera's view on a target `height` pixels tall.
    pub fn perspective(camera: Vec3, far: f32, fov_y_degrees: f32, height: u32) -> TerrainView {
        TerrainView {
            camera,
            far,
            pixel: 2.0 * (fov_y_degrees.to_radians() * 0.5).tan() / height.max(1) as f32,
            detail: 0.0,
        }
    }
}

/// The tiles a frame needs for `terrain` (drawn with `layout`, its heights
/// within `range`) seen from `view`, best first, at most `capacity`. From
/// the top level down, the tile whose samples look largest on screen (its
/// spacing over its distance from the camera, in pixels) is taken and its
/// four children become candidates, until every candidate's samples are
/// under a pixel or the capacity is reached; the level-0 tiles within
/// `view.detail` come right after the top level. Every tile comes after its
/// parent, and the top level is always first.
pub fn terrain_tiles_wanted(
    layout: &TerrainLayout,
    terrain: &Terrain,
    range: (f32, f32),
    view: &TerrainView,
    capacity: usize,
) -> Vec<u32> {
    use std::collections::{BinaryHeap, HashSet};
    let model = terrain.transform;
    let along = horizontal_scale(model);
    let local = model.inverse().transform_point3(view.camera);
    let eye = Vec2::new(local.x, local.z);
    let cell = layout.cell().min_element();
    let far = view.far / along;
    let pixel = view.pixel.max(1e-6);
    let dy = local.y - local.y.clamp(range.0, range.1);
    // How large a tile's samples look: their spacing over the distance, in pixels.
    let error = |index: u32| -> f32 {
        let (k, ..) = layout.locate(index).unwrap_or((0, 0, 0));
        let (lo, hi) = layout.rect(index);
        let d_xz = (eye.clamp(lo, hi) - eye).length();
        let d = (d_xz * d_xz + dy * dy).sqrt();
        if d_xz > far {
            return 0.0;
        }
        cell * layout.step(k) as f32 / (d.max(cell * 0.5) * pixel)
    };
    #[derive(PartialEq)]
    struct Candidate(f32, u32);
    impl Eq for Candidate {}
    impl PartialOrd for Candidate {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Candidate {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.0.total_cmp(&other.0).then(other.1.cmp(&self.1))
        }
    }
    let capacity = capacity.max(1);
    let mut out = Vec::new();
    let mut taken = HashSet::new();
    let mut heap = BinaryHeap::new();
    let top = layout.levels - 1;
    let (tx, tz) = layout.tiles(top);
    for z in 0..tz {
        for x in 0..tx {
            let i = layout.index(top, x, z);
            out.push(i);
            taken.insert(i);
            heap.push(Candidate(error(i), i));
        }
    }
    // The finest tiles near the camera (and their ancestors), whatever their size on screen.
    if view.detail > 0.0 {
        let d = view.detail / along;
        let mut near = Vec::new();
        cover(layout, 0, eye - d, eye + d, |i| near.push(i));
        og_core::sort::sort_by(&mut near, |a, b| {
            let da = (eye.clamp(layout.rect(*a).0, layout.rect(*a).1) - eye).length();
            let db = (eye.clamp(layout.rect(*b).0, layout.rect(*b).1) - eye).length();
            da.total_cmp(&db).then(a.cmp(b))
        });
        for i in near {
            let mut chain = vec![i];
            while let Some(p) = layout.parent(*chain.last().expect("one")) {
                chain.push(p);
            }
            for t in chain.into_iter().rev() {
                if taken.insert(t) {
                    out.push(t);
                    heap.push(Candidate(error(t), t));
                }
            }
        }
    }
    out.truncate(capacity);
    taken = out.iter().copied().collect();
    // Refine where the samples look largest.
    let mut expanded = HashSet::new();
    while out.len() < capacity {
        let Some(Candidate(e, i)) = heap.pop() else {
            break;
        };
        if e <= 1.0 {
            break;
        }
        if !taken.contains(&i) || !expanded.insert(i) {
            continue;
        }
        let Some((k, x, z)) = layout.locate(i) else {
            continue;
        };
        if k == 0 {
            continue;
        }
        let (cx, cz) = layout.tiles(k - 1);
        let mut children: Vec<u32> = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .map(|(ox, oz)| (2 * x + ox, 2 * z + oz))
            .filter(|&(x, z)| x < cx && z < cz)
            .map(|(x, z)| layout.index(k - 1, x, z))
            .collect();
        og_core::sort::sort_by(&mut children, |a, b| {
            error(*b).total_cmp(&error(*a)).then(a.cmp(b))
        });
        for c in children {
            if out.len() >= capacity {
                break;
            }
            if taken.insert(c) {
                out.push(c);
            }
            heap.push(Candidate(error(c), c));
        }
    }
    out
}
