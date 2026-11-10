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

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `band` in terrain_lit.wgsl.
fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
    let f = fade.max(1e-4);
    smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x))
}

/// A 32-bit integer hash (lowbias32).
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn unit(x: u32) -> f32 {
    (x >> 8) as f32 / 16_777_216.0
}

/// The hash a scatter's `seed` places its cells and clusters by.
pub(crate) fn seed_hash(seed: u32) -> u32 {
    hash(seed ^ 0x9e37_79b9)
}

/// What placing reads at a point of the terrain's space: its height,
/// normal and control channels (`None`: not resident yet).
pub(crate) trait Ground {
    fn size(&self) -> Vec2;
    fn height(&self, p: Vec2) -> Option<f32>;
    fn normal(&self, p: Vec2) -> Option<Vec3>;
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]>;
    /// Whether `p` is over a hole (no copy grows there).
    fn hole(&self, p: Vec2) -> Option<bool>;
}

impl Ground for TerrainData {
    fn size(&self) -> Vec2 {
        self.size
    }
    fn height(&self, p: Vec2) -> Option<f32> {
        Some(TerrainData::height(self, p))
    }
    fn normal(&self, p: Vec2) -> Option<Vec3> {
        Some(TerrainData::normal(self, p))
    }
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        Some(self.control_at(p))
    }
    fn hole(&self, p: Vec2) -> Option<bool> {
        Some(TerrainData::hole(self, p))
    }
}

/// A streamed terrain's resident level-0 tiles as ground.
pub(crate) struct TileGround<'a> {
    pub layout: TerrainLayout,
    pub tiles: &'a IdMap<u32, Arc<TerrainTile>>,
}

impl TileGround<'_> {
    /// The level-0 tile under `p` and its place.
    fn tile(&self, p: Vec2) -> Option<(&TerrainTile, u32, u32)> {
        let l = &self.layout;
        let s = ((p + l.size * 0.5) / l.cell()).max(Vec2::ZERO);
        let (tx, tz) = l.tiles(0);
        let x = ((s.x as u32) / l.tile).min(tx - 1);
        let z = ((s.y as u32) / l.tile).min(tz - 1);
        Some((self.tiles.get(&l.index(0, x, z))?.as_ref(), x, z))
    }

    fn sampler<'b>(
        &self,
        t: &'b TerrainTile,
        x: u32,
        z: u32,
    ) -> crate::TerrainSampler<impl Fn(u32, u32) -> f32 + 'b> {
        let l = self.layout;
        let side = l.heights_side() as i64;
        let (bx, bz) = ((x * l.tile) as i64, (z * l.tile) as i64);
        crate::TerrainSampler {
            size: l.size,
            columns: l.columns,
            rows: l.rows,
            sample: move |c: u32, r: u32| {
                let i = (c as i64 - bx + 1).clamp(0, side - 1);
                let j = (r as i64 - bz + 1).clamp(0, side - 1);
                t.heights[(j * side + i) as usize]
            },
        }
    }
}

impl Ground for TileGround<'_> {
    fn size(&self) -> Vec2 {
        self.layout.size
    }
    fn hole(&self, p: Vec2) -> Option<bool> {
        if !self.layout.holes {
            return Some(false);
        }
        let (t, x, z) = self.tile(p)?;
        let s = self.sampler(t, x, z);
        Some(
            s.cell_of(p)
                .is_some_and(|(c, r)| crate::terrain_tiles::hole_bit((s.sample)(c, r))),
        )
    }
    fn height(&self, p: Vec2) -> Option<f32> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).height(p))
    }
    fn normal(&self, p: Vec2) -> Option<Vec3> {
        let (t, x, z) = self.tile(p)?;
        Some(self.sampler(t, x, z).normal(p))
    }
    fn control(&self, p: Vec2) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        let (t, x, z) = self.tile(p)?;
        let l = &self.layout;
        let Some(texels) = &t.control else {
            return Some([1.0; MAX_TERRAIN_LAYERS]);
        };
        let page = l.control_page_bytes();
        let c = l.control as f32;
        let side = l.control_side() as i64;
        let s = (p + l.size * 0.5) / l.cell();
        // Texel coordinates within the tile (its border at 0).
        let g = (s - Vec2::new((x * l.tile) as f32, (z * l.tile) as f32)) / l.tile as f32 * c + 0.5;
        let i = g.floor();
        let f = g - i;
        // Channel k is page k / 4's (each page's first mip), 1 past the pages.
        let texel = |a: f32, b: f32, k: usize| {
            let a = (a as i64).clamp(0, side - 1);
            let b = (b as i64).clamp(0, side - 1);
            texels[(k / 4) * page + ((b * side + a) * 4) as usize + k % 4] as f32 / 255.0
        };
        let pages = l.pages.max(1) as usize;
        Some(std::array::from_fn(|k| {
            if k / 4 >= pages {
                return 1.0;
            }
            let lo = texel(i.x, i.y, k) + (texel(i.x + 1.0, i.y, k) - texel(i.x, i.y, k)) * f.x;
            let hi = texel(i.x, i.y + 1.0, k)
                + (texel(i.x + 1.0, i.y + 1.0, k) - texel(i.x, i.y + 1.0, k)) * f.x;
            lo + (hi - lo) * f.y
        }))
    }
}

impl TerrainData {
    /// The control map's channels at `local` (four per page; 1 past its
    /// pages), bilinear between texel centres (as the renderer's linear
    /// sampler reads it), 0..1; all 1 without one.
    pub fn control_at(&self, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        let Some(c) = &self.control else {
            return [1.0; MAX_TERRAIN_LAYERS];
        };
        let pages = c.pages() as usize;
        let page = (c.width * c.height * 4) as usize;
        let uv = ((local + self.size * 0.5) / self.size).clamp(Vec2::ZERO, Vec2::ONE);
        let t = uv * Vec2::new(c.width as f32, c.height as f32) - 0.5;
        let i = t.floor();
        let f = t - i;
        let texel = |x: f32, y: f32| -> [f32; MAX_TERRAIN_LAYERS] {
            let x = (x as i64).clamp(0, c.width as i64 - 1) as usize;
            let y = (y as i64).clamp(0, c.height as i64 - 1) as usize;
            let at = (y * c.width as usize + x) * 4;
            std::array::from_fn(|k| {
                if k / 4 < pages {
                    c.rgba[(k / 4) * page + at + k % 4] as f32 / 255.0
                } else {
                    1.0
                }
            })
        };
        let (a, b) = (texel(i.x, i.y), texel(i.x + 1.0, i.y));
        let (d, e) = (texel(i.x, i.y + 1.0), texel(i.x + 1.0, i.y + 1.0));
        std::array::from_fn(|k| {
            let lo = a[k] + (b[k] - a[k]) * f.x;
            let hi = d[k] + (e[k] - d[k]) * f.x;
            lo + (hi - lo) * f.y
        })
    }
}

/// Every weight on the first layer.
fn first_layer() -> [f32; MAX_TERRAIN_LAYERS] {
    std::array::from_fn(|k| if k == 0 { 1.0 } else { 0.0 })
}

impl Terrain {
    /// The layers' weights at `local` (summing to 1), as the renderer weighs
    /// them before the height blend: control map times the slope and
    /// altitude rules; the first layer where every weight is 0.
    pub fn layer_weights(&self, data: &TerrainData, local: Vec2) -> [f32; MAX_TERRAIN_LAYERS] {
        self.weights_on(data, local).unwrap_or_else(first_layer)
    }

    pub(crate) fn weights_on(
        &self,
        g: &dyn Ground,
        local: Vec2,
    ) -> Option<[f32; MAX_TERRAIN_LAYERS]> {
        let c = g.control(local)?;
        let n = g.normal(local)?;
        let angle = n.y.clamp(-1.0, 1.0).acos().to_degrees();
        let altitude = g.height(local)?;
        let mut w = [0.0f32; MAX_TERRAIN_LAYERS];
        for (i, l) in self.layers.iter().take(MAX_TERRAIN_LAYERS).enumerate() {
            w[i] = c[i]
                * band(angle, l.slope[0], l.slope[1], l.slope_fade)
                * band(altitude, l.altitude[0], l.altitude[1], l.altitude_fade);
        }
        let sum: f32 = w.iter().sum();
        if sum < 1e-4 {
            return Some(first_layer());
        }
        Some(w.map(|x| x / sum))
    }

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
