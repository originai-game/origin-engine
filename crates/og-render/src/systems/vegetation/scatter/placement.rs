use super::*;

/// Value noise over the lattice of `seed`, three octaves turned off the
/// axes, normalised to 0..1 (the clusters of `TerrainScatter::noise_threshold`).
fn cluster_noise(p: Vec2, seed: u32) -> f32 {
    let lattice = |q: Vec2, salt: u32| {
        let i = q.floor();
        let f = q - i;
        let u = f * f * (Vec2::splat(3.0) - 2.0 * f);
        let at = |dx: i32, dz: i32| {
            let x = (i.x as i32).wrapping_add(dx) as u32;
            let z = (i.y as i32).wrapping_add(dz) as u32;
            unit(hash(
                hash(x ^ salt).wrapping_add(z.wrapping_mul(0x9e37_79b9)) ^ seed,
            ))
        };
        let a = at(0, 0) + (at(1, 0) - at(0, 0)) * u.x;
        let b = at(0, 1) + (at(1, 1) - at(0, 1)) * u.x;
        a + (b - a) * u.y
    };
    let turn = Mat3::from_rotation_z(0.6);
    let (mut q, mut amp, mut sum, mut total) = (p, 1.0, 0.0, 0.0);
    for octave in 0..3u32 {
        sum += amp * lattice(q, octave.wrapping_mul(0x632b_e5ab));
        total += amp;
        q = turn.transform_vector2(q) * 2.03;
        amp *= 0.5;
    }
    sum / total
}

/// The placement's hash (scatter_place.wgsl draws from the same one).
use og_core::hash::{lowbias32 as hash, unit24 as unit};

/// The hash a scatter's `seed` places its cells and clusters by.
pub(crate) fn seed_hash(seed: u32) -> u32 {
    hash(seed ^ 0x9e37_79b9)
}

impl Terrain {
    /// The copy of cell (`x`, `z`) of `s`'s grid under every rule: its
    /// own ([`Self::copy_at`]), then clear of the water by `avoid_water`
    /// and of the copies of the scatters placed `before` it by the larger
    /// footprint. `Some(None)` where none grows, `None` where the ground is
    /// not resident yet.
    pub(super) fn copy_ruled(
        &self,
        g: &dyn Ground,
        s: &TerrainScatter,
        before: &[TerrainScatter],
        water: &[(Mat4, Vec2)],
        x: u32,
        z: u32,
    ) -> Option<Option<(u32, Mat4, Vec2)>> {
        let Some(copy) = self.copy_at(g, s, x, z)? else {
            return Some(None);
        };
        if s.avoid_water > 0.0 {
            let at = copy.1.w_axis.truncate();
            for (to_local, half) in water {
                let l = to_local.transform_point3(at);
                if l.x.abs() <= half.x && l.z.abs() <= half.y && l.y < s.avoid_water {
                    return Some(None);
                }
            }
        }
        let along = horizontal_scale(self.transform).max(1e-6);
        let p = copy.2;
        let size = g.size();
        for (i, b) in before.iter().enumerate() {
            let reach = s.footprint.max(b.footprint) / along;
            if reach <= 0.0 {
                continue;
            }
            let cell = b.cell();
            let cells = (size / cell).ceil();
            let lo = ((p - reach + size * 0.5) / cell).floor().max(Vec2::ZERO);
            let hi = ((p + reach + size * 0.5) / cell).floor().min(cells - 1.0);
            for bz in lo.y as u32..=hi.y.max(0.0) as u32 {
                for bx in lo.x as u32..=hi.x.max(0.0) as u32 {
                    if let Some((_, _, q)) = self.copy_ruled(g, b, &before[..i], water, bx, bz)?
                        && (q - p).length() < reach
                    {
                        return Some(None);
                    }
                }
            }
        }
        Some(Some(copy))
    }

    /// The copy of cell (`x`, `z`) of `s`'s grid by its own rules (layer,
    /// slope, altitude, holes, clusters): `Some(None)` where none grows,
    /// `None` where the ground is not resident yet.
    fn copy_at(
        &self,
        g: &dyn Ground,
        s: &TerrainScatter,
        x: u32,
        z: u32,
    ) -> Option<Option<(u32, Mat4, Vec2)>> {
        let cell = s.cell();
        let size = g.size();
        let half = size * 0.5;
        let cells_x = (size.x / cell).ceil() as u32;
        let key = z.wrapping_mul(cells_x).wrapping_add(x);
        let seed = seed_hash(s.seed);
        let h0 = hash(key ^ seed);
        let (h1, h2, h3, h4) = (hash(h0), hash(h0 ^ 1), hash(h0 ^ 2), hash(h0 ^ 3));
        let p = Vec2::new(x as f32 + unit(h1), z as f32 + unit(h2)) * cell - half;
        if p.x > half.x || p.y > half.y {
            return Some(None);
        }
        let weight = self.weights_on(g, p)?[s.layer as usize];
        if unit(h3) >= weight || g.hole(p)? {
            return Some(None);
        }
        if s.noise_threshold > 0.0 {
            // Patches in the world's metres; a short band dithered by the
            // copy's own value, so a cluster's edge thins out.
            let along = horizontal_scale(self.transform);
            let n = cluster_noise(p * along / s.noise_scale.max(1e-3), seed);
            let edge = smoothstep(s.noise_threshold - 0.04, s.noise_threshold + 0.04, n);
            if unit(hash(h0 ^ 5)) >= edge {
                return Some(None);
            }
        }
        let model = self.transform;
        let n3 = Mat3::from_mat4(model).inverse().transpose();
        let size = s.scale[0] + (s.scale[1] - s.scale[0]) * unit(h4);
        let normal = (n3 * g.normal(p)?).normalize_or(Vec3::Y);
        let up = Vec3::Y
            .lerp(normal, s.align.clamp(0.0, 1.0))
            .normalize_or(Vec3::Y);
        let turn = glam::Quat::from_rotation_y(unit(hash(h0 ^ 4)) * std::f32::consts::TAU);
        let rotation = glam::Quat::from_rotation_arc(Vec3::Y, up) * turn;
        let at = model.transform_point3(Vec3::new(p.x, g.height(p)?, p.y));
        Some(Some((
            key,
            Mat4::from_scale_rotation_translation(Vec3::splat(size), rotation, at),
            p,
        )))
    }

    /// Places `s`'s copies for a camera at `camera` on the CPU: each a cell
    /// key (stable while the cell is in range) and its local-to-world
    /// transform, at full size within `distance` of the camera along x and
    /// z (the renderer fades the outer `fade` of it), at most
    /// [`MAX_SCATTER_INSTANCES`], nearest rows first. `data` is this
    /// terrain's data.
    pub fn scatter(
        &self,
        data: &TerrainData,
        s: &TerrainScatter,
        camera: Vec3,
    ) -> Vec<(u32, Mat4)> {
        self.scatter_among(data, s, &ScatterContext::default(), camera)
    }

    /// [`Self::scatter`] with the scatters placed before `s` and the
    /// water (`ctx`): the copies the renderer draws for it in that scene.
    pub fn scatter_among(
        &self,
        data: &TerrainData,
        s: &TerrainScatter,
        ctx: &ScatterContext,
        camera: Vec3,
    ) -> Vec<(u32, Mat4)> {
        let mut out = Vec::new();
        if !s.usable(self.layers.len()) {
            return out;
        }
        let cell = s.cell();
        let half = data.size * 0.5;
        // Placed in the terrain's own space; distances are the world's.
        let along = horizontal_scale(self.transform);
        let local_camera = self.transform.inverse().transform_point3(camera);
        let eye = Vec2::new(local_camera.x, local_camera.z);
        let reach = s.distance / along;
        let cells = (data.size / cell).ceil();
        let lo = ((eye - reach + half) / cell).floor().max(Vec2::ZERO);
        let hi = ((eye + reach + half) / cell).ceil().min(cells);
        if lo.x >= hi.x || lo.y >= hi.y {
            return out;
        }
        let (x0, x1, z0, z1) = (lo.x as u32, hi.x as u32, lo.y as u32, hi.y as u32);
        // Rows outwards from the camera's, so the cap drops the farthest.
        let centre =
            (((eye.y + half.y) / cell).floor() as i64).clamp(z0 as i64, z1 as i64 - 1) as u32;
        let rows = (0..(z1 - z0)).map(|k| {
            let step = (k as i64 + 1) / 2;
            if k % 2 == 0 {
                centre as i64 + step
            } else {
                centre as i64 - step
            }
        });
        for z in rows
            .filter(|&z| z >= z0 as i64 && z < z1 as i64)
            .map(|z| z as u32)
        {
            for x in x0..x1 {
                if let Some(Some((key, m, p))) =
                    self.copy_ruled(data, s, &ctx.before, &ctx.water, x, z)
                    && (p - eye).length() <= reach
                {
                    out.push((key, m));
                    if out.len() >= MAX_SCATTER_INSTANCES {
                        return out;
                    }
                }
            }
        }
        out
    }
}
