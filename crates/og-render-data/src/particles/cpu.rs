use super::*;
use og_env::force::Forces;

// ---------------------------------------------------------------------------
// Particle state and the CPU simulation.
// ---------------------------------------------------------------------------

/// One particle slot (`struct Particle` in particles_sim.wgsl). Dead when
/// `age >= life`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleState {
    /// xyz: position (simulation space), w: age in seconds.
    pub pos_age: [f32; 4],
    /// xyz: velocity, w: lifetime in seconds.
    pub vel_life: [f32; 4],
    /// x: start size, y: start rotation (radians), z: angular velocity
    /// (radians per second), w: a per-particle random value in 0..1.
    pub misc: [f32; 4],
}

impl ParticleState {
    pub fn alive(&self) -> bool {
        self.pos_age[3] < self.vel_life[3]
    }
}

/// Points and normals on a mesh's surface, area weighted, for
/// [`EmitterShape::Mesh`] (the same table on both paths, and what a host
/// passes [`CpuParticles::advance`] as `points`): two entries a point.
pub fn mesh_points(mesh: &MeshData) -> Vec<[f32; 4]> {
    let tris: Vec<[Vec3; 3]> = mesh
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .filter_map(|t| {
            let v = |i: u32| {
                mesh.vertices
                    .get(i as usize)
                    .map(|v| Vec3::from(v.position))
            };
            Some([v(t[0])?, v(t[1])?, v(t[2])?])
        })
        .collect();
    let areas: Vec<f32> = tris
        .iter()
        .map(|[a, b, c]| (*b - *a).cross(*c - *a).length() * 0.5)
        .collect();
    let total: f32 = areas.iter().sum();
    let mut out = Vec::with_capacity(MESH_SAMPLE_POINTS * 2);
    if tris.is_empty() || total <= 0.0 {
        for _ in 0..MESH_SAMPLE_POINTS {
            out.push([0.0; 4]);
            out.push([0.0, 1.0, 0.0, 0.0]);
        }
        return out;
    }
    let mut cdf = Vec::with_capacity(areas.len());
    let mut acc = 0.0;
    for a in &areas {
        acc += a / total;
        cdf.push(acc);
    }
    for i in 0..MESH_SAMPLE_POINTS as u32 {
        let r = |s: u32| rand(0x6d65_7368, i, s);
        let pick = r(0);
        let t = cdf.partition_point(|&c| c < pick).min(tris.len() - 1);
        let [a, b, c] = tris[t];
        let (mut u, mut v) = (r(1), r(2));
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let p = a + (b - a) * u + (c - a) * v;
        let n = (b - a).cross(c - a).normalize_or(Vec3::Y);
        out.push(p.extend(1.0).to_array());
        out.push(n.extend(0.0).to_array());
    }
    out
}

/// A unit vector within `angle` radians of `axis` (uniform over the cap).
fn cone_dir(axis: Vec3, angle: f32, r0: f32, r1: f32) -> Vec3 {
    let cos_max = math::cos(angle.min(std::f32::consts::PI));
    let cos_t = 1.0 - r0 * (1.0 - cos_max);
    let sin_t = (1.0 - cos_t * cos_t).max(0.0).sqrt();
    let (phi_sin, phi_cos) = math::sin_cos(r1 * std::f32::consts::TAU);
    let helper = if axis.y.abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let t = helper.cross(axis).normalize();
    let b = axis.cross(t);
    (t * (phi_cos * sin_t) + b * (phi_sin * sin_t) + axis * cos_t).normalize_or(axis)
}

/// The emitter shape's (position, direction) for random values `r`.
fn shape_sample(shape: &EmitterShape, r: [f32; 4], points: &[[f32; 4]]) -> (Vec3, Vec3) {
    match *shape {
        EmitterShape::Point => (Vec3::ZERO, Vec3::Y),
        EmitterShape::Sphere { radius, surface } => {
            let d = cone_dir(Vec3::Y, std::f32::consts::PI, r[0], r[1]);
            let k = if surface {
                1.0
            } else {
                math::pow(r[2], 1.0 / 3.0)
            };
            (d * radius * k, d)
        }
        EmitterShape::Cone {
            angle_degrees,
            radius,
        } => {
            let d = cone_dir(Vec3::Y, angle_degrees.to_radians(), r[0], r[1]);
            let (sin, cos) = math::sin_cos(r[2] * std::f32::consts::TAU);
            let rr = r[3].sqrt() * radius;
            (Vec3::new(cos * rr, 0.0, sin * rr), d)
        }
        EmitterShape::Box { half_extents } => (
            (Vec3::new(r[0], r[1], r[2]) * 2.0 - Vec3::ONE) * half_extents,
            Vec3::Y,
        ),
        EmitterShape::Mesh { .. } | EmitterShape::Spline { .. } => {
            let count = points.len() / 2;
            if count == 0 {
                return (Vec3::ZERO, Vec3::Y);
            }
            let i = ((r[0] * count as f32) as usize).min(count - 1);
            (
                Vec4::from(points[i * 2]).truncate(),
                Vec4::from(points[i * 2 + 1]).truncate(),
            )
        }
    }
}

/// A newly spawned particle `n` (the `k`-th of `count` this tick).
#[allow(clippy::too_many_arguments)]
fn spawn(
    e: &ParticleEmitter,
    seed: u32,
    n: u32,
    k: u32,
    step: &TickStep,
    dt: f32,
    points: &[[f32; 4]],
) -> ParticleState {
    let r = |s: u32| rand(seed, n, s);
    let lerp = |range: [f32; 2], t: f32| range[0] + (range[1] - range[0]) * t;
    let life = lerp(e.lifetime, r(0)).max(1e-3);
    let speed = lerp(e.speed, r(1));
    let size = lerp(e.size, r(2)) * transform_scale(step.transform);
    let rot = lerp(e.rotation_degrees, r(3)).to_radians();
    let spin = lerp(e.angular_velocity_degrees, r(4)).to_radians();
    let (mut pos, dir) = shape_sample(&e.shape, [r(6), r(7), r(8), r(11)], points);
    let dir = if e.spread_degrees > 0.0 {
        cone_dir(dir, e.spread_degrees.to_radians(), r(9), r(10))
    } else {
        dir
    };
    // Continuous spawns spread over the tick; bursts start at age 0.
    let (age, frac) = if k < step.bursts {
        (0.0, 1.0)
    } else {
        let c = (step.count - step.bursts).max(1) as f32;
        let u = ((k - step.bursts) as f32 + 0.5) / c;
        (u * dt, 1.0 - u)
    };
    let mut vel = dir * speed;
    if e.space == SimulationSpace::World {
        if let Some(pm) = step.prev_transform.filter(|_| k >= step.bursts) {
            let p0 = pm.transform_point3(pos);
            let p1 = step.transform.transform_point3(pos);
            let v0 = pm.transform_vector3(vel);
            let v1 = step.transform.transform_vector3(vel);
            pos = p0.lerp(p1, frac);
            vel = v0.lerp(v1, frac) + step.velocity * e.inherit_velocity;
        } else {
            pos = step.transform.transform_point3(pos);
            vel = step.transform.transform_vector3(vel) + step.velocity * e.inherit_velocity;
        }
    }
    pos += vel * age;
    // What the emission leaves out takes its slot unborn (its age its life).
    let age = if r(EMISSION_STREAM) < e.emission {
        age
    } else {
        life
    };
    ParticleState {
        pos_age: pos.extend(age).to_array(),
        vel_life: vel.extend(life).to_array(),
        misc: [size, rot, spin, r(5)],
    }
}

/// The random stream that picks which particles
/// [`ParticleEmitter::emission`] leaves out (particles_sim.wgsl `spawn`).
const EMISSION_STREAM: u32 = 15;

/// A solid shape of the scene (og-render's `RenderScene::colliders`): what
/// CPU-simulated particles bounce off and visual bodies meet, the scene's
/// ground and solid shapes as the host knows them (og-runtime gives its
/// og.Collider boxes, spheres, capsules and terrains' height fields). A particle
/// bounces when a step carries it from outside a shape to inside: one born
/// inside (sparks from a brazier's box) leaves freely.
#[derive(Clone, Debug, PartialEq)]
pub enum SceneCollider {
    /// Solid below the plane `normal · p = offset` (`normal` need not be
    /// normalized): an endless ground.
    Plane {
        normal: Vec3,
        offset: f32,
    },
    /// A box of `half_extents` placed by `transform` (rotation, translation
    /// and scale).
    Box {
        transform: Mat4,
        half_extents: Vec3,
    },
    Sphere {
        center: Vec3,
        radius: f32,
    },
    /// The points within `radius` of the segment `a`..`b` (og.Collider's
    /// SHAPE_CAPSULE: a character).
    Capsule {
        a: Vec3,
        b: Vec3,
        radius: f32,
    },
    /// A height field placed as og-render's `Terrain::transform` places it
    /// (centred, along its x and z): solid below its surface, within its
    /// rectangle.
    HeightField {
        data: crate::TerrainData,
        transform: Mat4,
    },
}

/// A [`SceneCollider`] with its inverse transform and unit normal precomputed
/// once per step batch instead of inverting per particle per tick.
enum PreparedCollider<'a> {
    Plane {
        normal: Vec3,
        offset: f32,
    },
    Sphere {
        center: Vec3,
        radius_sq: f32,
    },
    Capsule {
        a: Vec3,
        b: Vec3,
        radius_sq: f32,
    },
    Box {
        inv: Mat4,
        inv_t: Mat4,
        half_extents: Vec3,
    },
    HeightField {
        data: &'a crate::TerrainData,
        inv: Mat4,
        inv_t: Mat4,
    },
}

impl<'a> PreparedCollider<'a> {
    fn of(c: &'a SceneCollider) -> Self {
        match c {
            SceneCollider::Plane { normal, offset } => {
                let len = normal.length();
                if len > 0.0 {
                    PreparedCollider::Plane {
                        normal: *normal / len,
                        offset: *offset / len,
                    }
                } else {
                    PreparedCollider::Plane {
                        normal: Vec3::ZERO,
                        offset: 0.0,
                    }
                }
            }
            SceneCollider::Sphere { center, radius } => PreparedCollider::Sphere {
                center: *center,
                radius_sq: radius * radius,
            },
            SceneCollider::Capsule { a, b, radius } => PreparedCollider::Capsule {
                a: *a,
                b: *b,
                radius_sq: radius * radius,
            },
            SceneCollider::Box {
                transform,
                half_extents,
            } => {
                let inv = transform.inverse();
                PreparedCollider::Box {
                    inv,
                    inv_t: inv.transpose(),
                    half_extents: *half_extents,
                }
            }
            SceneCollider::HeightField { data, transform } => {
                let inv = transform.inverse();
                PreparedCollider::HeightField {
                    data,
                    inv,
                    inv_t: inv.transpose(),
                }
            }
        }
    }

    /// Whether `p` is inside, and the outward normal there (for a box, of
    /// the face nearest `from`, where the particle came from).
    fn hit(&self, from: Vec3, p: Vec3) -> Option<Vec3> {
        match self {
            PreparedCollider::Plane { normal, offset } => {
                (*normal != Vec3::ZERO && normal.dot(p) < *offset).then_some(*normal)
            }
            PreparedCollider::Sphere { center, radius_sq } => {
                let d = p - *center;
                (d.length_squared() < *radius_sq).then(|| d.normalize_or(Vec3::Y))
            }
            PreparedCollider::Capsule { a, b, radius_sq } => {
                let d = p - og_core::math::closest_on_segment(p, *a, *b);
                (d.length_squared() < *radius_sq).then(|| d.normalize_or(Vec3::Y))
            }
            PreparedCollider::Box {
                inv,
                inv_t,
                half_extents,
            } => {
                let l = inv.transform_point3(p);
                if l.abs().cmpge(*half_extents).any() {
                    return None;
                }
                // The face the step came through: the axis it was most outside on.
                let f = inv.transform_point3(from);
                let out = f.abs() - *half_extents;
                let axis = if out.x >= out.y && out.x >= out.z {
                    Vec3::X * f.x.signum()
                } else if out.y >= out.z {
                    Vec3::Y * f.y.signum()
                } else {
                    Vec3::Z * f.z.signum()
                };
                Some(inv_t.transform_vector3(axis).normalize_or(Vec3::Y))
            }
            PreparedCollider::HeightField { data, inv, inv_t } => {
                let l = inv.transform_point3(p);
                let half = data.size * 0.5;
                if l.x.abs() > half.x || l.z.abs() > half.y {
                    return None;
                }
                let at = glam::Vec2::new(l.x, l.z);
                (l.y < data.height(at)).then(|| {
                    let n = data.normal(at);
                    inv_t.transform_vector3(n).normalize_or(Vec3::Y)
                })
            }
        }
    }
}

/// The CPU path's collision response, matching `particles_sim.wgsl`:
/// returns `(touched, impact)` where `touched` is any inward contact (`vn < 0`,
/// used for `collision die`) and `impact` exceeds `IMPACT_SPEED` (used for
/// collision events and sub-emitters so settling particles do not spam events).
fn collide(
    colliders: &[PreparedCollider<'_>],
    c: ParticleCollision,
    before: Vec3,
    pos: &mut Vec3,
    vel: &mut Vec3,
    dt: f32,
) -> (bool, bool) {
    for collider in colliders {
        if collider.hit(before, before).is_some() {
            continue;
        }
        let Some(n) = collider.hit(before, *pos) else {
            continue;
        };
        let vn = vel.dot(n);
        if vn < 0.0 {
            let normal_part = n * vn;
            *vel = (*vel - normal_part) * (1.0 - c.friction) - normal_part * c.bounce;
            *pos = before + *vel * dt;
            return (true, -vn > IMPACT_SPEED);
        }
        return (false, false);
    }
    (false, false)
}

/// The turbulence's lattice at the render origin `origin` for noise of
/// `frequency` (cells per metre): its whole cells and the rest, in f64, so
/// a world-space system samples the field it would at the world's zero
/// (`origin.rs`). Zero at a zero origin.
pub fn noise_origin(origin: Vec3, frequency: f32) -> ([i32; 3], Vec3) {
    let at = |o: f32| {
        let x = f64::from(o) * f64::from(frequency);
        let w = x.floor();
        (w as i32, (x - w) as f32)
    };
    let (x, y, z) = (at(origin.x), at(origin.y), at(origin.z));
    ([x.0, y.0, z.0], Vec3::new(x.1, y.1, z.1))
}

/// The pull of the attractors (`attractors`: in the simulation space) at
/// `p`.
fn attract(attractors: &[Attractor], p: Vec3) -> Vec3 {
    let mut a = Vec3::ZERO;
    for at in attractors.iter().take(MAX_ATTRACTORS) {
        let d = at.position - p;
        let dist = d.length();
        if dist < 1e-4 {
            continue;
        }
        let f = if at.radius > 0.0 {
            at.strength * (1.0 - dist / at.radius).max(0.0)
        } else {
            at.strength / (1.0 + dist * dist)
        };
        a += d * (f / dist);
    }
    a
}

/// Curl noise at lattice point `q` (`whole` cells added): the curl of
/// three value noise fields by central differences, about -1 to 1 on each
/// axis. `curl_noise` in particles_sim.wgsl.
pub fn curl_noise(q: Vec3, whole: [i32; 3], seed: u32) -> Vec3 {
    const H: f32 = 0.05;
    let psi = |q: Vec3, k: u32| value_noise3_at(q, whole, seed ^ (0x4444 + k * 0x1111));
    let d = |k: u32, axis: Vec3| (psi(q + axis * H, k) - psi(q - axis * H, k)) / (2.0 * H);
    Vec3::new(
        d(2, Vec3::Y) - d(1, Vec3::Z),
        d(0, Vec3::Z) - d(2, Vec3::X),
        d(1, Vec3::X) - d(0, Vec3::Y),
    ) * 0.25
}

/// A grid field's vector at `g` (in cells, tiling), trilinear.
pub fn grid_sample(cells: u32, vectors: &[Vec3], g: Vec3) -> Vec3 {
    let n = cells.max(1) as i32;
    let fl = g.floor();
    let f = g - fl;
    let at = |dx: i32, dy: i32, dz: i32| {
        let w = |c: f32, d: i32| (c as i32 + d).rem_euclid(n) as usize;
        let (x, y, z) = (w(fl.x, dx), w(fl.y, dy), w(fl.z, dz));
        vectors
            .get((z * n as usize + y) * n as usize + x)
            .copied()
            .unwrap_or(Vec3::ZERO)
    };
    let x00 = at(0, 0, 0).lerp(at(1, 0, 0), f.x);
    let x10 = at(0, 1, 0).lerp(at(1, 1, 0), f.x);
    let x01 = at(0, 0, 1).lerp(at(1, 0, 1), f.x);
    let x11 = at(0, 1, 1).lerp(at(1, 1, 1), f.x);
    x00.lerp(x10, f.y).lerp(x01.lerp(x11, f.y), f.z)
}

/// Where a field's lattice stands at the render origin `origin`: a curl
/// field's whole cells and the rest (as [`noise_origin`]), a grid's
/// offset in cells within one tile.
pub fn field_origin(field: &VectorField, origin: Vec3) -> ([i32; 3], Vec3) {
    match field {
        VectorField::Curl { frequency, .. } => noise_origin(origin, *frequency),
        VectorField::Grid { size, cells, .. } => {
            let at = |o: f32| {
                let x = f64::from(o) / f64::from(*size);
                ((x - x.floor()) * f64::from(*cells)) as f32
            };
            ([0; 3], Vec3::new(at(origin.x), at(origin.y), at(origin.z)))
        }
    }
}

/// How much a system's transform scales its effect: the largest axis
/// scale (an effect spawned at scale 2, or under a scaled parent). Every
/// length of the emitter scales by it, so a scaled effect is the same
/// effect larger: its particles' size where they are born, and in world
/// space (where the transform is applied at birth, not at draw) its
/// accelerations too: gravity, noise and attractors. Velocities already
/// scale through the transform; drag is a rate and does not.
pub fn transform_scale(transform: Mat4) -> f32 {
    let s = transform
        .to_scale_rotation_translation()
        .0
        .abs()
        .max_element();
    if s.is_finite() && s > 0.0 { s } else { 1.0 }
}

/// The scale a world-space system's accelerations take (1 for a local
/// one, whose whole simulation the draw's transform scales).
pub fn force_scale(e: &ParticleEmitter, transform: Mat4) -> f32 {
    match e.space {
        SimulationSpace::World => transform_scale(transform),
        SimulationSpace::Local => 1.0,
    }
}

/// Stack-allocated slice of at most [`MAX_ATTRACTORS`] attractors transformed
/// into simulation space (avoids per-step `Vec` allocations).
#[derive(Clone, Copy, Debug)]
pub struct PlacedAttractors {
    items: [Attractor; MAX_ATTRACTORS],
    len: usize,
}

impl std::ops::Deref for PlacedAttractors {
    type Target = [Attractor];
    fn deref(&self) -> &[Attractor] {
        &self.items[..self.len]
    }
}

/// An emitter's attractors in its simulation space at a tick placed by
/// `transform`: carried into the world for a world-space system, their
/// pull scaled with it.
pub fn placed_attractors(e: &ParticleEmitter, transform: Mat4) -> PlacedAttractors {
    let s = force_scale(e, transform);
    let mut items = [Attractor {
        position: Vec3::ZERO,
        strength: 0.0,
        radius: 0.0,
    }; MAX_ATTRACTORS];
    let mut len = 0;
    for a in e.attractors.iter().take(MAX_ATTRACTORS) {
        items[len] = match e.space {
            SimulationSpace::World => Attractor {
                position: transform.transform_point3(a.position),
                strength: a.strength * s,
                ..*a
            },
            SimulationSpace::Local => *a,
        };
        len += 1;
    }
    PlacedAttractors { items, len }
}

/// The acceleration of `pulls` (attractors in the simulation space) and
/// the field at `p` at `time` seconds.
fn forces(
    e: &ParticleEmitter,
    pulls: &[Attractor],
    p: Vec3,
    time: f32,
    seed: u32,
    origin: Vec3,
) -> Vec3 {
    let mut a = attract(pulls, p);
    match &e.field {
        Some(
            f @ VectorField::Curl {
                strength,
                frequency,
                scroll,
            },
        ) => {
            let (whole, rest) = field_origin(f, origin);
            let q = p * *frequency + rest + Vec3::splat(time * scroll);
            a += curl_noise(q, whole, seed) * *strength;
        }
        Some(
            f @ VectorField::Grid {
                strength,
                size,
                cells,
                vectors,
            },
        ) => {
            let (_, rest) = field_origin(f, origin);
            let g = p / *size * *cells as f32 + rest;
            a += grid_sample(*cells, vectors, g) * *strength;
        }
        None => {}
    }
    a
}

/// A spline shape's table, as [`mesh_points`]'s: points evenly along the
/// Catmull-Rom curve through `points[..count]` with its tangent, two
/// entries a point.
pub fn spline_points(points: &[Vec3], closed: bool) -> Vec<[f32; 4]> {
    let n = points.len();
    let mut out = Vec::with_capacity(MESH_SAMPLE_POINTS * 2);
    if n < 2 {
        for _ in 0..MESH_SAMPLE_POINTS {
            out.push([0.0; 4]);
            out.push([0.0, 1.0, 0.0, 0.0]);
        }
        return out;
    }
    let segments = if closed { n } else { n - 1 };
    let p = |i: isize| {
        if closed {
            points[i.rem_euclid(n as isize) as usize]
        } else {
            points[i.clamp(0, n as isize - 1) as usize]
        }
    };
    let at = |u: f32| {
        let s = ((u * segments as f32) as usize).min(segments - 1);
        let t = u * segments as f32 - s as f32;
        let i = s as isize;
        og_core::math::catmull_rom([p(i - 1), p(i), p(i + 1), p(i + 2)], t)
    };
    // Even by length: a dense polyline, then its length's equal steps.
    const DENSE: usize = 4096;
    let dense: Vec<Vec3> = (0..=DENSE).map(|i| at(i as f32 / DENSE as f32)).collect();
    let mut length = vec![0.0f32; DENSE + 1];
    for i in 1..=DENSE {
        length[i] = length[i - 1] + (dense[i] - dense[i - 1]).length();
    }
    let total = length[DENSE].max(1e-6);
    for k in 0..MESH_SAMPLE_POINTS {
        let want = (k as f32 + 0.5) / MESH_SAMPLE_POINTS as f32 * total;
        let i = length.partition_point(|&l| l < want).clamp(1, DENSE);
        let f = (want - length[i - 1]) / (length[i] - length[i - 1]).max(1e-9);
        let pos = dense[i - 1].lerp(dense[i], f);
        let dir = (dense[i] - dense[i - 1]).normalize_or(Vec3::Y);
        out.push(pos.extend(1.0).to_array());
        out.push(dir.extend(0.0).to_array());
    }
    out
}

/// The surface table of an emitter shape that has one (a mesh, its mesh's
/// `MeshData`; a spline): two entries a point, a placeholder point for
/// the others.
pub fn shape_points(shape: &EmitterShape, mesh: Option<&MeshData>) -> Vec<[f32; 4]> {
    match (shape, mesh) {
        (EmitterShape::Mesh { .. }, Some(m)) => mesh_points(m),
        (
            EmitterShape::Spline {
                points,
                count,
                closed,
            },
            _,
        ) => spline_points(&points[..(*count as usize).min(MAX_SPLINE_POINTS)], *closed),
        _ => vec![[0.0; 4], [0.0, 1.0, 0.0, 0.0]],
    }
}

/// What an emitter's drag closes on, in its simulation space: its `wind`
/// share of the air where its system is (`air`, world), turned into the
/// emitter's space for a local one (`local`: its transform this step).
/// Nothing, as before, for an emitter that takes no wind or has no drag
/// for the air to hold.
pub fn carrying_air(e: &ParticleEmitter, air: Vec3, local: Option<Mat4>) -> Vec3 {
    let share = e.wind.clamp(0.0, 1.0);
    if share == 0.0 || e.drag <= 0.0 || air == Vec3::ZERO {
        return Vec3::ZERO;
    }
    let air = air * share;
    match local {
        Some(m) if m.determinant() != 0.0 => m.inverse().transform_vector3(air),
        _ => air,
    }
}

/// One fixed step of a live particle at `time` seconds; `origin`: the
/// render origin a world-space system is simulated relative to; `air`:
/// what its drag closes on ([`carrying_air`]). Returns whether it had an
/// impact exceeding [`IMPACT_SPEED`].
#[allow(clippy::too_many_arguments)]
fn integrate(
    e: &ParticleEmitter,
    pulls: &[Attractor],
    p: &mut ParticleState,
    dt: f32,
    time: f32,
    seed: u32,
    colliders: &[PreparedCollider<'_>],
    origin: Vec3,
    air: Vec3,
    fields: &Forces,
    scale: f32,
) -> bool {
    let mut pos = Vec3::from_slice(&p.pos_age[..3]);
    let mut vel = Vec3::from_slice(&p.vel_life[..3]);
    vel += e.gravity * (scale * dt);
    if e.force_fields != 0.0 && e.space == SimulationSpace::World && !fields.is_empty() {
        vel += fields.at(pos) * (e.force_fields * dt);
    }
    if e.noise.strength != 0.0 {
        let (whole, rest) = noise_origin(origin, e.noise.frequency);
        let q = pos * e.noise.frequency + rest + Vec3::splat(time * e.noise.scroll);
        let n = Vec3::new(
            value_noise3_at(q, whole, seed ^ 0x1111),
            value_noise3_at(q, whole, seed ^ 0x2222),
            value_noise3_at(q, whole, seed ^ 0x3333),
        ) * 2.0
            - Vec3::ONE;
        vel += n * (e.noise.strength * scale * dt);
    }
    vel += forces(e, pulls, pos, time, seed, origin) * dt;
    // Drag closes on the air's velocity: on rest in still air, bit for bit
    // as before.
    let keep = 1.0 / (1.0 + e.drag.max(0.0) * dt);
    vel = if air == Vec3::ZERO {
        vel * keep
    } else {
        air + (vel - air) * keep
    };
    let before = pos;
    let t = (p.pos_age[3] / p.vel_life[3].max(1e-6)).clamp(0.0, 1.0);
    pos += vel * (dt * e.speed_over_life.as_ref().map_or(1.0, |c| c.eval(t)));
    let (touched, impact) = match e.collision {
        Some(c) => collide(colliders, c, before, &mut pos, &mut vel, dt),
        None => (false, false),
    };
    // One that dies on a hit dies on ANY inward contact (matching GPU `hit`),
    // even below IMPACT_SPEED; only impacts fire collision events/sub-emitters.
    let age = if touched && e.collision.is_some_and(|c| c.die) {
        p.vel_life[3]
    } else {
        p.pos_age[3] + dt
    };
    p.pos_age = pos.extend(age).to_array();
    p.vel_life = vel.extend(p.vel_life[3]).to_array();
    impact
}

/// For a slot, the particle index spawned into it by `step` (the newest of
/// the step's spawns that maps to this slot), if any.
#[inline]
pub fn slot_spawn(slot: u32, max: u32, first: u64, count: u32) -> Option<(u32, u32)> {
    let first_mod = (first % max as u64) as u32;
    let off = (slot + max - first_mod) % max;
    if off >= count {
        return None;
    }
    let k = off + ((count - 1 - off) / max) * max;
    Some(((first as u32).wrapping_add(k), k))
}

/// The particles one [`SubEmitter`] fired, simulated with its emitter in
/// the world (its spawn order's own ring of the child's `max_particles`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChildParticles {
    pub slots: Vec<ParticleState>,
    /// The particles born into it so far: the next one's index.
    pub born: u64,
}

/// The CPU path of one system: its slots and where its clock stood.
#[derive(Clone, Debug, Default)]
pub struct CpuParticles {
    pub slots: Vec<ParticleState>,
    pub last_tick: Option<u64>,
    pub prev_transform: Option<Mat4>,
    pub clock: ParticleClock,
    /// The render origin a world-space system's slots are relative to
    /// (`origin.rs`): its turbulence samples the field there.
    pub origin: Vec3,
    /// Per [`ParticleEmitter::sub_emitters`] entry, the particles it fired.
    pub children: Vec<ChildParticles>,
    /// With [`ParticleEmitter::trail`], each slot's path: its
    /// [`TRAIL_POINTS`] samples in a ring (slot `i`'s at `i * TRAIL_POINTS`;
    /// xyz in the simulation space), the newest at [`trail_newest`].
    pub trail: Vec<[f32; 4]>,
    /// The system's time at its last step (seconds since it started).
    pub time: f32,
    /// The ticks of the last [`Self::advance`], kept for the next one's
    /// plan: a system stepped every tick allocates nothing.
    pub steps: Vec<TickStep>,
}

/// The ring index a trail's sample goes to in a tick that ends at `time`
/// after `dt` seconds of particle time, when the tick crosses a sample
/// (each `interval` seconds; [`ParticleTrail::interval`]).
pub fn trail_sample(time: f32, dt: f32, interval: f32) -> Option<usize> {
    let n = (time / interval).floor();
    (n > ((time - dt) / interval).floor())
        .then(|| (n as i64).rem_euclid(TRAIL_POINTS as i64) as usize)
}

/// A trail's newest sample at `time`: its ring index and its age in
/// seconds (the age of the sample before it is `interval` more).
pub fn trail_newest(time: f32, interval: f32) -> (usize, f32) {
    let n = (time / interval).floor();
    (
        (n as i64).rem_euclid(TRAIL_POINTS as i64) as usize,
        (time - n * interval).max(0.0),
    )
}

/// The index of the particle in `slot` while `step` integrates it: the
/// slot's latest spawn before the step (particles fill the ring in order).
fn index_of(step: &TickStep, slot: u32, max: u32) -> u32 {
    let (last, max, slot) = (step.first.saturating_sub(1), max as u64, slot as u64);
    let back = (last % max + max - slot) % max;
    last.saturating_sub(back) as u32
}

/// The seed of a system's `k`-th sub-emitter's particles.
pub fn child_seed(seed: u32, k: usize) -> u32 {
    pcg(seed ^ pcg(0x5ab_e317 ^ k as u32))
}

/// A sub-emitter's particle `n`, born at `at` (world) with `inherited`
/// velocity added to its own start velocity. A mesh shape is a point here.
fn spawn_child(e: &ParticleEmitter, seed: u32, n: u32, at: Vec3, inherited: Vec3) -> ParticleState {
    let r = |s: u32| rand(seed, n, s);
    let lerp = |range: [f32; 2], t: f32| range[0] + (range[1] - range[0]) * t;
    let life = lerp(e.lifetime, r(0)).max(1e-3);
    let speed = lerp(e.speed, r(1));
    let size = lerp(e.size, r(2));
    let rot = lerp(e.rotation_degrees, r(3)).to_radians();
    let spin = lerp(e.angular_velocity_degrees, r(4)).to_radians();
    let shape = match e.shape {
        EmitterShape::Mesh { .. } | EmitterShape::Spline { .. } => EmitterShape::Point,
        s => s,
    };
    let (offset, dir) = shape_sample(&shape, [r(6), r(7), r(8), r(11)], &[]);
    let dir = if e.spread_degrees > 0.0 {
        cone_dir(dir, e.spread_degrees.to_radians(), r(9), r(10))
    } else {
        dir
    };
    ParticleState {
        pos_age: (at + offset).extend(0.0).to_array(),
        vel_life: (dir * speed + inherited).extend(life).to_array(),
        misc: [size, rot, spin, r(5)],
    }
}

/// A living parent particle for [`SubTrigger::Life`]: its age before the
/// step (`before`) and after, its position and velocity in the world.
fn placed(p: &ParticleState, before: f32, local: bool, m: Mat4) -> (f32, f32, Vec3, Vec3) {
    let (mut at, mut vel) = (
        Vec3::from_slice(&p.pos_age[..3]),
        Vec3::from_slice(&p.vel_life[..3]),
    );
    if local {
        at = m.transform_point3(at);
        vel = m.transform_vector3(vel);
    }
    let age = p.pos_age[3].min(p.vel_life[3]);
    (before.min(age), age, at, vel)
}

impl CpuParticles {
    /// Steps the system to `sys.tick`; returns the ticks simulated.
    /// `colliders`: what its particles bounce off (empty: nothing). `subs`:
    /// the emitters of `e.sub_emitters`, in order (`None`: not fired).
    /// `events`: where to report every birth, death and collision of the
    /// system's own particles (not its sub-emitters'), in slot order per
    /// tick. `fields`: the og.ForceFields, placed in the particles' world
    /// space (relative to [`Self::origin`]).
    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        e: &ParticleEmitter,
        sys: &ParticleSystem,
        tick_rate: f32,
        points: &[[f32; 4]],
        colliders: &[SceneCollider],
        subs: &[Option<&ParticleEmitter>],
        mut events: Option<&mut Vec<ParticleEvent>>,
        history: Option<&ParticleHistory>,
        fields: &Forces,
    ) -> u32 {
        let max = e.max_particles;
        if self.slots.len() != max as usize {
            self.slots = vec![ParticleState::default(); max as usize];
            self.last_tick = None;
        }
        let ring = if e.trail.is_some() {
            max as usize * TRAIL_POINTS
        } else {
            0
        };
        if self.trail.len() != ring {
            // A ring new to these particles (a trail added, a restored
            // snapshot, which keeps none): each path starts where it is.
            self.trail = self
                .slots
                .iter()
                .flat_map(|p| [[p.pos_age[0], p.pos_age[1], p.pos_age[2], 0.0]; TRAIL_POINTS])
                .take(ring)
                .collect();
        }
        let interval = e.trail.map(|t| t.interval(tick_rate));
        let firing: Vec<(usize, &SubEmitter, &ParticleEmitter)> = e
            .sub_emitters
            .iter()
            .enumerate()
            .filter_map(|(k, s)| Some((k, s, (*subs.get(k)?)?)))
            .collect();
        self.children
            .resize_with(e.sub_emitters.len(), Default::default);
        for &(k, _, ce) in &firing {
            let c = &mut self.children[k];
            if c.slots.len() != ce.max_particles as usize {
                c.slots = vec![ParticleState::default(); ce.max_particles as usize];
                c.born = 0;
            }
        }
        let mut steps = std::mem::take(&mut self.steps);
        let reset = plan_ticks(
            e,
            sys,
            self.last_tick,
            self.prev_transform,
            tick_rate,
            &mut self.clock,
            history,
            &mut steps,
        );
        if reset {
            self.slots.fill(ParticleState::default());
            for c in &mut self.children {
                c.slots.fill(ParticleState::default());
                c.born = 0;
            }
        }
        let seed = hash_seed(sys.seed);
        let want = |kind| {
            events.is_some()
                || firing
                    .iter()
                    .any(|f| f.1.trigger == SubTrigger::Event(kind))
        };
        // Parents that run a child's emission while they live: each one's
        // age before and after the step, where it is and how fast it goes.
        let lives = firing.iter().any(|f| f.1.trigger == SubTrigger::Life);
        let mut living: Vec<(f32, f32, Vec3, Vec3)> = Vec::new();
        let (births, deaths, hits) = (
            want(ParticleEventKind::Birth),
            want(ParticleEventKind::Death),
            want(ParticleEventKind::Collision),
        );
        let local = e.space == SimulationSpace::Local;
        let needs_colliders =
            e.collision.is_some() || firing.iter().any(|&(_, _, ce)| ce.collision.is_some());
        let prepared_colliders: Vec<PreparedCollider<'_>> = if needs_colliders {
            colliders.iter().map(PreparedCollider::of).collect()
        } else {
            Vec::new()
        };
        let mut fired: Vec<ParticleEvent> = Vec::new();
        for step in &steps {
            let dt = step.dt;
            let pulls = placed_attractors(e, step.transform);
            let air = carrying_air(e, sys.air, local.then_some(step.transform));
            fired.clear();
            let mut report = |kind, n: u32, p: &ParticleState| {
                let (mut at, mut vel) = (
                    Vec3::from_slice(&p.pos_age[..3]),
                    Vec3::from_slice(&p.vel_life[..3]),
                );
                if local {
                    at = step.transform.transform_point3(at);
                    vel = step.transform.transform_vector3(vel);
                }
                fired.push(ParticleEvent {
                    kind,
                    tick: step.tick,
                    index: n,
                    position: at,
                    velocity: vel,
                });
            };
            living.clear();
            let sample = interval.and_then(|iv| trail_sample(step.time, dt, iv));
            for (slot, p) in self.slots.iter_mut().enumerate() {
                let before = p.pos_age[3];
                let path = (!self.trail.is_empty())
                    .then(|| &mut self.trail[slot * TRAIL_POINTS..(slot + 1) * TRAIL_POINTS]);
                match slot_spawn(slot as u32, max, step.first, step.count) {
                    Some((n, k)) => {
                        *p = spawn(e, seed, n, k, step, dt, points);
                        if !p.alive() {
                            continue;
                        }
                        // Its path starts where it is born.
                        if let Some(path) = path {
                            path.fill([p.pos_age[0], p.pos_age[1], p.pos_age[2], 0.0]);
                        }
                        if births {
                            report(ParticleEventKind::Birth, n, p);
                        }
                        if lives {
                            living.push(placed(p, 0.0, local, step.transform));
                        }
                    }
                    None if p.alive() => {
                        let hit = integrate(
                            e,
                            &pulls,
                            p,
                            dt,
                            step.time,
                            seed,
                            &prepared_colliders,
                            self.origin,
                            air,
                            fields,
                            force_scale(e, step.transform),
                        );
                        if let (Some(path), Some(i)) = (path, sample)
                            && p.alive()
                        {
                            path[i] = [p.pos_age[0], p.pos_age[1], p.pos_age[2], 0.0];
                        }
                        if hit && hits {
                            report(
                                ParticleEventKind::Collision,
                                index_of(step, slot as u32, max),
                                p,
                            );
                        }
                        if !p.alive() && deaths {
                            report(
                                ParticleEventKind::Death,
                                index_of(step, slot as u32, max),
                                p,
                            );
                        }
                        // Its last step too: up to the end of its life.
                        if lives {
                            living.push(placed(p, before, local, step.transform));
                        }
                    }
                    None => {}
                }
            }
            // The sub-emitters: their particles step, then each event's are born.
            for &(k, sub, ce) in &firing {
                let cseed = child_seed(seed, k);
                let c = &mut self.children[k];
                let cmax = ce.max_particles.max(1) as u64;
                for q in c.slots.iter_mut().filter(|q| q.alive()) {
                    integrate(
                        ce,
                        &ce.attractors,
                        q,
                        dt,
                        step.time,
                        cseed,
                        &prepared_colliders,
                        self.origin,
                        carrying_air(ce, sys.air, None),
                        fields,
                        1.0,
                    );
                }
                if sub.trigger == SubTrigger::Life {
                    // Each parent's share of the child's own schedule over
                    // the ages this step covered.
                    for &(a0, a1, at, vel) in &living {
                        let spawned = |age: f32| {
                            let (c, b) = spawned_before(ce, f64::from(age), f64::INFINITY);
                            c + b
                        };
                        for _ in spawned(a0)..spawned(a1) {
                            let n = c.born;
                            c.born += 1;
                            c.slots[(n % cmax) as usize] =
                                spawn_child(ce, cseed, n as u32, at, vel * sub.inherit_velocity);
                        }
                    }
                    continue;
                }
                for ev in fired
                    .iter()
                    .filter(|ev| SubTrigger::Event(ev.kind) == sub.trigger)
                {
                    for _ in 0..sub.count {
                        let n = c.born;
                        c.born += 1;
                        c.slots[(n % cmax) as usize] = spawn_child(
                            ce,
                            cseed,
                            n as u32,
                            ev.position,
                            ev.velocity * sub.inherit_velocity,
                        );
                    }
                }
            }
            if let Some(out) = events.as_deref_mut() {
                out.extend_from_slice(&fired);
            }
            self.time = step.time;
        }
        self.last_tick = Some(sys.tick);
        self.prev_transform = Some(sys.transform);
        let stepped = steps.len() as u32;
        self.steps = steps;
        stepped
    }

    pub fn alive(&self) -> usize {
        self.slots.iter().filter(|p| p.alive()).count()
    }
}

/// What a particle looks like at its age: (width and height, colour,
/// rotation, frame of `frames`: the sheet's, or an array emitter's layers).
pub fn appearance(
    e: &ParticleEmitter,
    p: &ParticleState,
    frames: u32,
) -> ([f32; 2], [f32; 4], f32, u32) {
    let frames = frames.max(1);
    let t = (p.pos_age[3] / p.vel_life[3].max(1e-6)).clamp(0.0, 1.0);
    let width = p.misc[0] * e.size_over_life.eval(t);
    let size = [
        width,
        e.height_over_life
            .as_ref()
            .map_or(width, |h| p.misc[0] * h.eval(t)),
    ];
    let mut c = e.color_over_life.eval(t);
    if let Some(other) = &e.random_color_over_life {
        let o = other.eval(t);
        let u = color_share(p.misc[3]);
        c = std::array::from_fn(|i| c[i] + (o[i] - c[i]) * u);
    }
    let color = [
        c[0] * e.color[0],
        c[1] * e.color[1],
        c[2] * e.color[2],
        c[3] * e.color[3],
    ];
    let rotation = particle_rotation(e, p, t);
    let frame = match e.sheet.mode {
        SheetMode::Off => 0,
        SheetMode::OverLife { cycles } => (t * cycles * frames as f32) as u32 % frames,
        SheetMode::Fps(fps) => (p.pos_age[3] * fps) as u32 % frames,
        SheetMode::Random => ((p.misc[3] * frames as f32) as u32).min(frames - 1),
    };
    (size, color, rotation, frame)
}

/// Where a particle's colour sits between its emitter's two colour curves,
/// 0 to 1, from its random value `r` (`color_share` in
/// particle_common.wgsl): another draw of it than its frame's and its
/// tumble's.
fn color_share(r: f32) -> f32 {
    (r * 61.0).fract()
}

/// A particle's rotation at life `t`: its start plus its angular velocity
/// times its age, or with [`ParticleEmitter::spin_over_life`] times the
/// curve's integral over its life (`particle_rotation` in
/// particle_common.wgsl).
pub fn particle_rotation(e: &ParticleEmitter, p: &ParticleState, t: f32) -> f32 {
    match &e.spin_over_life {
        None => p.misc[1] + p.misc[2] * p.pos_age[3],
        Some(c) => p.misc[1] + p.misc[2] * p.vel_life[3] * c.integral(t),
    }
}

/// The frames an array emitter's particles go through: `sheet.frames` of
/// the array's `layers` (0: every layer).
pub fn array_frames(sheet: &SpriteSheet, layers: u32) -> u32 {
    match sheet.frames {
        0 => layers,
        n => n.min(layers),
    }
    .max(1)
}

pub fn sheet_frames(sheet: &SpriteSheet) -> u32 {
    let all = sheet.columns.max(1) * sheet.rows.max(1);
    if sheet.frames == 0 {
        all
    } else {
        sheet.frames.min(all)
    }
}
