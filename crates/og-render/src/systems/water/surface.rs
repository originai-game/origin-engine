use super::*;

/// The surface's plane in world space: (unit normal facing up out of the
/// water, a point on it).
pub(crate) fn plane(water: &Water) -> (Vec3, Vec3) {
    let up = water
        .transform
        .transform_vector3(Vec3::Y)
        .normalize_or(Vec3::Y);
    (up, water.transform.transform_point3(Vec3::ZERO))
}

/// The surface as the screen AO and the planar reflection see it: its plane (normal, a point) and its world xz extent.
/// A surface over a mesh ([`Water::mesh`], `mesh` its data) lies at the mean height of the vertices it draws (its
/// indices; the entity's own origin can be anywhere, e.g. a mesh in world coordinates under an entity at 0) and spans
/// their extent; a grid spans its size around the entity.
pub(crate) fn surface(
    water: &Water,
    mesh: Option<&crate::scene::MeshData>,
) -> (Vec3, Vec3, Vec2, Vec2) {
    let (up, origin) = plane(water);
    if let Some(m) = mesh.filter(|m| !m.indices.is_empty()) {
        let (mut sum, mut lo, mut hi) = (Vec3::ZERO, Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        let mut n = 0.0;
        for &i in &m.indices {
            let Some(v) = m.vertices.get(i as usize) else {
                continue;
            };
            let p = Vec3::from(v.position);
            sum += p;
            n += 1.0;
            let w = water.transform.transform_point3(p);
            lo = lo.min(Vec2::new(w.x, w.z));
            hi = hi.max(Vec2::new(w.x, w.z));
        }
        if n > 0.0 {
            return (up, water.transform.transform_point3(sum / n), lo, hi);
        }
    }
    let h = water.size * 0.5;
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for c in [
        Vec3::new(-h.x, 0.0, -h.y),
        Vec3::new(h.x, 0.0, -h.y),
        Vec3::new(-h.x, 0.0, h.y),
        Vec3::new(h.x, 0.0, h.y),
    ] {
        let w = water.transform.transform_point3(c);
        lo = lo.min(Vec2::new(w.x, w.z));
        hi = hi.max(Vec2::new(w.x, w.z));
    }
    (up, origin, lo, hi)
}

/// The surface's height field when it is one the renderer uses: 2 to
/// [`MAX_RESOLUTION`] heights per side, all of them, over a positive size.
pub(super) fn height_field(water: &Water) -> Option<&WaterHeights> {
    water.heights.as_ref().filter(|f| {
        f.resolution >= 2
            && f.resolution <= MAX_RESOLUTION
            && f.heights.len() == (f.resolution * f.resolution) as usize
            && f.size > 0.0
    })
}

/// The surface's height above its rest plane at local xz (the height
/// field, bilinear; 0 without one).
fn local_height(water: &Water, local: Vec2) -> f32 {
    let Some(f) = height_field(water) else {
        return 0.0;
    };
    let n = f.resolution as usize;
    let cell = f.size / (n - 1) as f32;
    let g =
        ((local + Vec2::splat(f.size * 0.5)) / cell).clamp(Vec2::ZERO, Vec2::splat((n - 1) as f32));
    let i = g.floor().min(Vec2::splat((n - 2) as f32));
    let t = g - i;
    let (x, z) = (i.x as usize, i.y as usize);
    let at = |x: usize, z: usize| f.heights[z * n + x];
    let lo = at(x, z) + (at(x + 1, z) - at(x, z)) * t.x;
    let hi = at(x, z + 1) + (at(x + 1, z + 1) - at(x, z + 1)) * t.x;
    lo + (hi - lo) * t.y
}

/// The surface's height (world y) at world point `at` (xz) as the tick
/// floats bodies on it: its plane and height field, and over a sea its
/// waves and seabed at `seconds` (the scene's `origin` places them).
fn surface_height(w: &Water, at: Vec2, seconds: f64, origin: Vec3) -> f32 {
    let local = w
        .transform
        .inverse()
        .transform_point3(Vec3::new(at.x, 0.0, at.y));
    let plane = w
        .transform
        .transform_point3(Vec3::new(local.x, 0.0, local.z))
        .y;
    let sea = ocean(w).map_or(0.0, |sea| {
        let a = glam::DVec2::new(f64::from(at.x + origin.x), f64::from(at.y + origin.z));
        sea.surface_near(a, seconds, coast(w, origin).as_ref())
            .height
    });
    plane + local_height(w, Vec2::new(local.x, local.z)) + sea
}

/// W6: the surface the camera's near plane crosses, if any, among those
/// with an underwater view: the eye within the near plane's reach of the
/// surface over it (its half diagonal, at most about twice `near` for a
/// field of view up to 90° at 16:9), inside the surface's extent. The frame
/// then draws with the air's fog, and the waterline pass fogs what lies
/// under the surface pixel by pixel, the line between them its meniscus.
pub(crate) fn waterline(
    waters: &[Water],
    camera: &crate::Camera,
    seconds: f64,
    origin: Vec3,
) -> Option<usize> {
    let eye = camera.position;
    waters.iter().position(|w| {
        let reach = 2.1 * camera.near.max(1e-3);
        if w.underwater_density.is_nan() || w.underwater_density <= 0.0 || w.meniscus < 0.0 {
            return false;
        }
        let local = w.transform.inverse().transform_point3(eye);
        local.x.abs() <= w.size.x * 0.5
            && local.z.abs() <= w.size.y * 0.5
            && (eye.y - surface_height(w, Vec2::new(eye.x, eye.z), seconds, origin)).abs() < reach
    })
}

/// The surface the camera at `eye` is below (inside its extent, under its
/// waves), if any, among those with an underwater view.
/// `seconds` and the scene's `origin` place the sea's waves ([`Water::wind`]).
pub(crate) fn underwater(waters: &[Water], eye: Vec3, seconds: f64, origin: Vec3) -> Option<usize> {
    waters.iter().position(|w| {
        if w.underwater_density.is_nan() || w.underwater_density <= 0.0 {
            return false;
        }
        let local = w.transform.inverse().transform_point3(eye);
        let sea = ocean(w).map_or(0.0, |sea| {
            let at = (eye + origin).as_dvec3();
            let at = glam::DVec2::new(at.x, at.z);
            sea.surface_near(at, seconds, coast(w, origin).as_ref())
                .height
        });
        local.x.abs() <= w.size.x * 0.5
            && local.z.abs() <= w.size.y * 0.5
            && local.y < local_height(w, Vec2::new(local.x, local.z)) + sea
    })
}

/// The fog of the underwater view: the water's colour, thickening by its
/// density from the eye, the sky left clear (Snell's window shows it).
/// Its light falls off with depth and comes down in shafts
/// (media.wgsl `underwater_light`), which [`underwater_light`] sets.
pub(crate) fn underwater_fog(w: &Water) -> crate::Fog {
    crate::Fog {
        color: w.underwater_color,
        density: w.underwater_density,
        start: 0.0,
        end: 0.0,
        max_opacity: 1.0,
        height_density: 0.0,
        sky: 0.0,
        sun_color: w.underwater_color,
        sun_power: 8.0,
        ..Default::default()
    }
}

/// How the water's light falls off under surface `w` (`mesh` its data):
/// the surface's height in the frame, the daylight's attenuation per metre
/// of depth and the sun's own share of the water's light.
///
/// The attenuation is 0.45 of the fog's density: the diffuse attenuation
/// of coastal water is its absorption and backscatter, between a third
/// and a half of its beam attenuation (Mobley, Light and Water, 1994,
/// ch. 3 and 5). The sun's share is the light the water volume draws as
/// shafts (volumetric.rs `Medium::Water`): the same light the waves focus
/// into the caustics, so it takes the caustics' strength, up to
/// [`SUN_SHARE`].
pub(crate) fn underwater_light(w: &Water, mesh: Option<&crate::scene::MeshData>) -> [f32; 3] {
    let (_, at, _, _) = surface(w, mesh);
    [
        at.y,
        0.45 * w.underwater_density.max(0.0),
        SUN_SHARE * w.caustics.clamp(0.0, 1.0),
    ]
}

/// The most of the water's own light that is the sun's, still in its
/// beam: the rest is the sky's and what the water has scattered about,
/// which is even. (Under a clear sky the sun is some four fifths of the
/// light entering the water; scattering spreads it with depth.)
const SUN_SHARE: f32 = 0.6;

/// The camera mirrored in the plane through `at` with unit normal `n`.
pub(crate) fn mirrored_camera(camera: &crate::Camera, n: Vec3, at: Vec3) -> crate::Camera {
    let point = |p: Vec3| p - n * (2.0 * (p - at).dot(n));
    let dir = |d: Vec3| d - n * (2.0 * d.dot(n));
    crate::Camera {
        position: point(camera.position),
        target: point(camera.target),
        up: dir(camera.up),
        ..*camera
    }
}

/// `proj` with its near plane replaced by the world plane (`n`, `at`)
/// seen through `view` (Lengyel's oblique near-plane clipping, for depth
/// 0..1): what lies below the plane is clipped and the far plane is
/// tilted to keep the depth range. `lift` raises the plane (m) to hide
/// the seam where geometry meets the water.
pub(crate) fn oblique(proj: Mat4, view: Mat4, n: Vec3, at: Vec3, lift: f32) -> Mat4 {
    let world = n.extend(-(n.dot(at) + lift));
    // Planes transform by the inverse transpose.
    let c = view.inverse().transpose() * world;
    let q = proj.inverse() * Vec4::new(c.x.signum(), c.y.signum(), 1.0, 1.0);
    let c = c * (1.0 / c.dot(q));
    let mut rows = [proj.row(0), proj.row(1), c, proj.row(3)];
    if !rows[2].is_finite() {
        rows[2] = proj.row(2);
    }
    Mat4::from_cols(rows[0], rows[1], rows[2], rows[3]).transpose()
}

/// Appends the strips of the 2D water bodies to `frame` as runs in the
/// sprite order: the body (a gradient from `top` at the surface to
/// `bottom`) and the lip along the surface (soft across its width).
pub(crate) fn water_2d_strips(waters: &[Water2d], frame: &mut crate::ribbons::RibbonFrame) {
    use crate::ribbons::{RibbonRun, RibbonVertex};
    for w in waters {
        let n = w.surface.len();
        if n < 2 || w.width.is_nan() || w.depth.is_nan() || w.width <= 0.0 || w.depth <= 0.0 {
            continue;
        }
        let first = frame.vertices.len() as u32;
        let at = |i: usize| {
            let x = w.origin.x + w.width * i as f32 / (n - 1) as f32;
            let h = w.surface[i];
            (x, w.origin.y + if h.is_finite() { h } else { 0.0 })
        };
        let bottom_y = w.origin.y - w.depth;
        let top = w.top.to_array();
        let bottom = w.bottom.to_array();
        let lip = w.lip.to_array();
        let half = w.lip_width.max(0.0) * 0.5;
        for i in 0..n - 1 {
            let (x0, y0) = at(i);
            let (x1, y1) = at(i + 1);
            let v = |x: f32, y: f32| glam::Vec3::new(x, y, w.z);
            // Body: two triangles from the surface down.
            let (a, b, c, d) = (v(x0, y0), v(x1, y1), v(x0, bottom_y), v(x1, bottom_y));
            for (p, col) in [
                (a, top),
                (c, bottom),
                (b, top),
                (b, top),
                (c, bottom),
                (d, bottom),
            ] {
                frame
                    .vertices
                    .push(RibbonVertex::new(p, 0.5, 0.5, col, true));
            }
            if half > 0.0 {
                let (a, b) = (v(x0, y0 + half), v(x1, y1 + half));
                let (c, d) = (v(x0, y0 - half), v(x1, y1 - half));
                for (p, vv) in [(a, 0.0), (c, 1.0), (b, 0.0), (b, 0.0), (c, 1.0), (d, 1.0)] {
                    frame
                        .vertices
                        .push(RibbonVertex::new(p, 0.5, vv, lip, false));
                }
            }
        }
        frame.runs.push(RibbonRun {
            texture: None,
            blend: crate::ParticleBlend::Alpha,
            layer: Some(w.layer),
            points: false,
            first,
            count: frame.vertices.len() as u32 - first,
            effect: None,
        });
    }
}
