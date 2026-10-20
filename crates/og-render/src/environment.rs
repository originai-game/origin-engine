//! Cooked environment maps: image-based light from an HDR image instead of
//! the procedural sky (three.js `scene.environment`), shown behind the scene
//! through [`crate::Background::Sky`] (three.js `scene.background`).
//!
//! A map is prefiltered ahead of time (og-cook's `type environment`, or
//! [`bake`] with the `bake` feature): a cube whose level `m` is the
//! radiance GGX-prefiltered at roughness `m / (ENVIRONMENT_MAP_LEVELS - 1)`
//! (level 0 the radiance itself), the convention of the procedural sky's
//! own prefiltered cube, in RGB9E5; and the diffuse irradiance as nine
//! spherical-harmonic coefficients. When a frame sets one
//! ([`crate::RenderScene::environment_map`]) the IBL passes copy it, turned
//! and scaled, into the renderer's own sky, irradiance and prefiltered
//! cubes instead of drawing the sky, so every shader that reads the image
//! based light reads the map unchanged.

/// Levels of a cooked environment map: the procedural sky's prefiltered
/// cube's, so level `m` holds roughness `m / (ENVIRONMENT_MAP_LEVELS - 1)`.
pub const ENVIRONMENT_MAP_LEVELS: u32 = crate::ibl::PREFILTER_MIPS;

/// The smallest and largest face size of an environment map's level 0.
pub const ENVIRONMENT_MAP_MIN_SIZE: u32 = 1 << (ENVIRONMENT_MAP_LEVELS - 1);
pub const ENVIRONMENT_MAP_MAX_SIZE: u32 = 2048;

/// A prefiltered environment map (an asset: [`crate::RenderScene::add_environment_map`]).
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentMapData {
    /// Level 0's face size in texels: a power of two from
    /// [`ENVIRONMENT_MAP_MIN_SIZE`] to [`ENVIRONMENT_MAP_MAX_SIZE`].
    pub size: u32,
    /// RGB9E5 texels ([`rgb9e5`]): level after level
    /// ([`ENVIRONMENT_MAP_LEVELS`], each half the size of the one before),
    /// each the six faces in wgpu's order (+x, -x, +y, -y, +z, -z), each
    /// face's rows from the top. Texel `(i, j)` of a face of size `n` looks
    /// along [`face_direction`] at `x = 2 (i + 0.5) / n - 1`,
    /// `y = 1 - 2 (j + 0.5) / n`.
    pub texels: Vec<u32>,
    /// The diffuse irradiance over pi (what a white Lambertian surface
    /// facing `n` reflects) as order-2 spherical harmonics, already
    /// convolved: `sum(irradiance[k] * Y_k(n))` with the real basis
    /// `Y_0 = 0.282095`, `Y_1..3 = 0.488603 (y, z, x)`, `Y_4..8 =
    /// 1.092548 xy, 1.092548 yz, 0.315392 (3z² - 1), 1.092548 xz,
    /// 0.546274 (x² - y²)`.
    pub irradiance: [[f32; 3]; 9],
}

impl EnvironmentMapData {
    /// Texels in a map of level 0 size `size`.
    pub fn texel_count(size: u32) -> usize {
        (0..ENVIRONMENT_MAP_LEVELS)
            .map(|m| 6 * ((size >> m) as usize).pow(2))
            .sum()
    }

    /// Checks the size and the texel count.
    pub fn validate(&self) -> Result<(), String> {
        if !self.size.is_power_of_two()
            || !(ENVIRONMENT_MAP_MIN_SIZE..=ENVIRONMENT_MAP_MAX_SIZE).contains(&self.size)
        {
            return Err(format!(
                "an environment map's size is a power of two from {ENVIRONMENT_MAP_MIN_SIZE} to {ENVIRONMENT_MAP_MAX_SIZE}, not {}",
                self.size
            ));
        }
        let want = Self::texel_count(self.size);
        if self.texels.len() != want {
            return Err(format!(
                "a {0}² environment map holds {want} texels, not {1}",
                self.size,
                self.texels.len()
            ));
        }
        if self.irradiance.iter().flatten().any(|v| !v.is_finite()) {
            return Err("an environment map's irradiance is not finite".into());
        }
        Ok(())
    }
}

/// How a frame uses an environment map ([`crate::RenderScene::environment_map`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentMap {
    pub map: crate::EnvironmentMapHandle,
    /// Turns the map about +y, in radians (counter-clockwise seen from
    /// above: what was at +x moves towards -z), as three.js
    /// `scene.environmentRotation.y`.
    pub rotation: f32,
    /// Multiplies the map where it is drawn as the background
    /// ([`crate::Background::Sky`]), as three.js `backgroundIntensity`; the
    /// light's strength is [`crate::Environment::intensity`].
    pub background_intensity: f32,
}

impl EnvironmentMap {
    /// `map` unturned, its background as cooked.
    pub fn new(map: crate::EnvironmentMapHandle) -> EnvironmentMap {
        EnvironmentMap {
            map,
            rotation: 0.0,
            background_intensity: 1.0,
        }
    }
}

/// The direction texel `(x, y)` of cube face `face` looks along, `x` and
/// `y` in -1..1 (y up): wgpu's (WebGPU's) cube convention, as the IBL
/// shaders draw the faces.
pub fn face_direction(face: u32, x: f32, y: f32) -> glam::Vec3 {
    let d = match face {
        0 => glam::Vec3::new(1.0, y, -x),
        1 => glam::Vec3::new(-1.0, y, x),
        2 => glam::Vec3::new(x, 1.0, -y),
        3 => glam::Vec3::new(x, -1.0, y),
        4 => glam::Vec3::new(x, y, 1.0),
        _ => glam::Vec3::new(-x, y, -1.0),
    };
    d.normalize()
}

/// Packs a linear RGB colour into RGB9E5 (a 5-bit exponent shared by three
/// 9-bit mantissas: `wgpu::TextureFormat::Rgb9e5Ufloat`), rounding to
/// nearest as the EXT_texture_shared_exponent specification does; negative
/// and non-finite channels become 0, values above 65408 clamp.
pub fn rgb9e5(rgb: [f32; 3]) -> u32 {
    const MANTISSA: i32 = 9;
    const BIAS: i32 = 15;
    const MAX: f64 = 65408.0;
    let c = rgb.map(|v| {
        let v = v as f64;
        if v.is_finite() {
            v.clamp(0.0, MAX)
        } else {
            0.0
        }
    });
    let max = c[0].max(c[1]).max(c[2]);
    let mut exp = (max.log2().floor() as i32).max(-BIAS - 1) + 1 + BIAS;
    let scale = |e: i32| 2f64.powi(e - BIAS - MANTISSA);
    if (max / scale(exp) + 0.5).floor() as i32 == 1 << MANTISSA {
        exp += 1;
    }
    let m = c.map(|v| ((v / scale(exp) + 0.5).floor() as u32).min(511));
    m[0] | (m[1] << 9) | (m[2] << 18) | ((exp as u32) << 27)
}

/// Unpacks [`rgb9e5`].
pub fn rgb9e5_decode(v: u32) -> [f32; 3] {
    let exp = (v >> 27) as i32;
    let scale = 2f32.powi(exp - 15 - 9);
    [v & 511, (v >> 9) & 511, (v >> 18) & 511].map(|m| m as f32 * scale)
}

/// The real spherical-harmonic basis of order 2 at unit direction `d`
/// (see [`EnvironmentMapData::irradiance`]).
pub fn sh9(d: glam::Vec3) -> [f32; 9] {
    [
        0.282095,
        0.488603 * d.y,
        0.488603 * d.z,
        0.488603 * d.x,
        1.092548 * d.x * d.y,
        1.092548 * d.y * d.z,
        0.315392 * (3.0 * d.z * d.z - 1.0),
        1.092548 * d.x * d.z,
        0.546274 * (d.x * d.x - d.y * d.y),
    ]
}

/// The IBL passes' uniform block for a map: (cos, sin, scale, 0) then the
/// 27 irradiance floats.
pub(crate) fn env_params(rotation: f32, scale: f32, irradiance: &[[f32; 3]; 9]) -> [[f32; 4]; 8] {
    let mut out = [[0.0; 4]; 8];
    out[0] = [rotation.cos(), rotation.sin(), scale, 0.0];
    for (k, v) in irradiance.iter().flatten().enumerate() {
        out[1 + k / 4][k % 4] = *v;
    }
    out
}

/// Uploads a map as a cube texture of [`ENVIRONMENT_MAP_LEVELS`] levels.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    map: &EnvironmentMapData,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("environment map"),
        size: wgpu::Extent3d {
            width: map.size,
            height: map.size,
            depth_or_array_layers: 6,
        },
        mip_level_count: ENVIRONMENT_MAP_LEVELS,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgb9e5Ufloat,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut at = 0;
    for level in 0..ENVIRONMENT_MAP_LEVELS {
        let n = map.size >> level;
        let count = 6 * (n * n) as usize;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&map.texels[at..at + count]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * n),
                rows_per_image: Some(n),
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 6,
            },
        );
        at += count;
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("environment map"),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });
    (texture, view)
}

/// Prefiltering an equirectangular HDR image into an [`EnvironmentMapData`]
/// on the CPU: what og-cook's `type environment` runs at cook time (feature
/// `bake`; never in the runtime wasm). It lives here because it must match
/// the renderer's conventions exactly: the face orientation, the roughness
/// of each level and the GGX sampling of the procedural sky's own prefilter
/// (`shaders/ibl.wgsl`).
#[cfg(feature = "bake")]
pub mod bake {
    use super::{
        ENVIRONMENT_MAP_LEVELS, ENVIRONMENT_MAP_MAX_SIZE, ENVIRONMENT_MAP_MIN_SIZE,
        EnvironmentMapData, face_direction, rgb9e5, sh9,
    };
    use glam::Vec3;
    use std::f32::consts::PI;

    /// An equirectangular (latitude-longitude) linear RGB image: rows from
    /// the top (+y) to the bottom (-y); column `u` (0..1 across the width)
    /// looks towards longitude `(u - 0.5) * 2 pi` from +x towards +z, so the
    /// image's centre column is +x and its three-quarter column +z: three.js's
    /// mapping of an equirectangular texture.
    pub struct Equirect<'a> {
        pub width: u32,
        pub height: u32,
        /// `width * height * 3` floats.
        pub rgb: &'a [f32],
    }

    /// A cube map of linear RGB: six square faces in [`face_direction`]'s
    /// order and orientation (+x −x +y −y +z −z, each with rows from the
    /// top, as KTX2 and WebGPU lay a cube out), face after face.
    pub struct CubeFaces<'a> {
        /// Each face's width and height.
        pub size: u32,
        /// `6 * size * size * 3` floats.
        pub rgb: &'a [f32],
    }

    /// A cube map with its own prefiltered mip chain (cmgen's, IBLBaker's,
    /// the glTF IBL sampler's): `levels[k]` holds six faces of
    /// `max(1, size >> k)` texels in [`CubeFaces`]'s layout, the radiance
    /// GGX-prefiltered at roughness `k / (levels.len() - 1)` (level 0 the
    /// radiance itself).
    pub struct PrefilteredCube<'a> {
        /// Level 0's face size.
        pub size: u32,
        /// At least two levels, each `6 * n * n * 3` floats.
        pub levels: &'a [Vec<f32>],
    }

    /// How to bake.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct BakeDesc {
        /// Level 0's face size (a power of two, 16..=2048).
        pub size: u32,
        /// Multiplies the image.
        pub intensity: f32,
        /// Turns the image about +y, in degrees (as
        /// [`super::EnvironmentMap::rotation`]).
        pub rotation_degrees: f32,
        /// GGX samples per texel of each prefiltered level.
        pub samples: u32,
    }

    impl Default for BakeDesc {
        fn default() -> Self {
            BakeDesc {
                size: 128,
                intensity: 1.0,
                rotation_degrees: 0.0,
                samples: 256,
            }
        }
    }

    /// A float cube with a box-filtered mip chain (the prefilter's source).
    struct Cube {
        /// Per level: face size and `6 * n * n` texels.
        levels: Vec<(u32, Vec<Vec3>)>,
    }

    impl Equirect<'_> {
        fn texel(&self, x: i64, y: i64) -> Vec3 {
            let w = self.width as i64;
            let x = x.rem_euclid(w) as usize;
            let y = y.clamp(0, self.height as i64 - 1) as usize;
            let i = (y * self.width as usize + x) * 3;
            Vec3::new(self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
        }

        /// Bilinear radiance along unit direction `d` (wrapping in longitude).
        fn sample(&self, d: Vec3) -> Vec3 {
            let u = d.z.atan2(d.x) / (2.0 * PI) + 0.5;
            let v = 0.5 - d.y.clamp(-1.0, 1.0).asin() / PI;
            let x = u * self.width as f32 - 0.5;
            let y = v * self.height as f32 - 0.5;
            let (x0, y0) = (x.floor(), y.floor());
            let (fx, fy) = (x - x0, y - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let top = self.texel(x0, y0).lerp(self.texel(x0 + 1, y0), fx);
            let bottom = self.texel(x0, y0 + 1).lerp(self.texel(x0 + 1, y0 + 1), fx);
            top.lerp(bottom, fy)
        }
    }

    fn texel_direction(face: u32, n: u32, i: u32, j: u32, di: f32, dj: f32) -> Vec3 {
        let x = 2.0 * (i as f32 + di) / n as f32 - 1.0;
        let y = 1.0 - 2.0 * (j as f32 + dj) / n as f32;
        face_direction(face, x, y)
    }

    /// The face and texel coordinates (in texels, continuous) of direction `d`.
    fn locate(d: Vec3, n: u32) -> (usize, f32, f32) {
        let a = d.abs();
        let (face, x, y) = if a.x >= a.y && a.x >= a.z {
            if d.x > 0.0 {
                (0, -d.z / a.x, d.y / a.x)
            } else {
                (1, d.z / a.x, d.y / a.x)
            }
        } else if a.y >= a.z {
            if d.y > 0.0 {
                (2, d.x / a.y, -d.z / a.y)
            } else {
                (3, d.x / a.y, d.z / a.y)
            }
        } else if d.z > 0.0 {
            (4, d.x / a.z, d.y / a.z)
        } else {
            (5, -d.x / a.z, d.y / a.z)
        };
        let n = n as f32;
        (face, (x + 1.0) * 0.5 * n - 0.5, (1.0 - y) * 0.5 * n - 0.5)
    }

    impl Cube {
        fn bilinear(&self, level: usize, d: Vec3) -> Vec3 {
            let (n, texels) = &self.levels[level];
            let (face, x, y) = locate(d, *n);
            let last = *n as i64 - 1;
            let at = |x: i64, y: i64| {
                let (x, y) = (x.clamp(0, last) as usize, y.clamp(0, last) as usize);
                texels[(face * *n as usize + y) * *n as usize + x]
            };
            let (x0, y0) = (x.floor(), y.floor());
            let (fx, fy) = (x - x0, y - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let top = at(x0, y0).lerp(at(x0 + 1, y0), fx);
            let bottom = at(x0, y0 + 1).lerp(at(x0 + 1, y0 + 1), fx);
            top.lerp(bottom, fy)
        }

        /// Trilinear between levels at fractional `lod`.
        fn sample(&self, d: Vec3, lod: f32) -> Vec3 {
            let top = self.levels.len() - 1;
            let lod = lod.clamp(0.0, top as f32);
            let l0 = lod.floor() as usize;
            let l1 = (l0 + 1).min(top);
            self.bilinear(l0, d)
                .lerp(self.bilinear(l1, d), lod - l0 as f32)
        }
    }

    impl CubeFaces<'_> {
        /// The faces as a [`Cube`] with its box-filtered mips.
        fn cube(&self) -> Cube {
            let texels = self
                .rgb
                .as_chunks::<3>()
                .0
                .iter()
                .map(|c| Vec3::from_array(*c))
                .collect();
            mips(self.size, texels)
        }
    }

    impl Cube {
        /// Bilinear radiance along `d` from the level a cube of face size
        /// `n` sampled 2x2 per texel reads without skipping texels: the
        /// largest whose face is at most `2n` (the base when it is smaller).
        fn resample(&self, d: Vec3, n: u32) -> Vec3 {
            let level = self
                .levels
                .iter()
                .position(|(m, _)| *m <= 2 * n)
                .unwrap_or(self.levels.len() - 1);
            self.bilinear(level, d)
        }
    }

    /// A cube's mips down to 1x1 by 2x2 box filtering (an odd face drops
    /// its last row and column).
    fn mips(n: u32, base: Vec<Vec3>) -> Cube {
        let mut levels = vec![(n, base)];
        while levels.last().expect("base").0 > 1 {
            let (m, above) = levels.last().expect("base");
            let (m, h) = (*m as usize, *m as usize / 2);
            let mut next = Vec::with_capacity(6 * h * h);
            for face in 0..6 {
                for y in 0..h {
                    for x in 0..h {
                        let at = |x: usize, y: usize| above[(face * m + y) * m + x];
                        next.push(
                            (at(2 * x, 2 * y)
                                + at(2 * x + 1, 2 * y)
                                + at(2 * x, 2 * y + 1)
                                + at(2 * x + 1, 2 * y + 1))
                                * 0.25,
                        );
                    }
                }
            }
            levels.push((h as u32, next));
        }
        Cube { levels }
    }

    /// The radiance cube at `n` (2x2 samples per texel of `src`'s radiance
    /// along a direction), and its mips down to 1x1 by 2x2 box filtering.
    fn radiance_cube(src: &dyn Fn(Vec3) -> Vec3, n: u32, turn: f32, intensity: f32) -> Cube {
        let (c, s) = (turn.cos(), turn.sin());
        let mut base = Vec::with_capacity(6 * (n * n) as usize);
        for face in 0..6 {
            for j in 0..n {
                for i in 0..n {
                    let mut sum = Vec3::ZERO;
                    for (di, dj) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                        let d = texel_direction(face, n, i, j, di, dj);
                        // The map turned by `turn`: look the image up at
                        // the direction turned back.
                        let d = Vec3::new(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
                        sum += src(d);
                    }
                    base.push(sum * 0.25 * intensity);
                }
            }
        }
        mips(n, base)
    }

    fn radical_inverse(mut bits: u32) -> f32 {
        bits = bits.reverse_bits();
        bits as f32 * 2.328_306_4e-10
    }

    fn tangent_frame(n: Vec3) -> (Vec3, Vec3) {
        let up = if n.y.abs() > 0.999 { Vec3::X } else { Vec3::Y };
        let t = up.cross(n).normalize();
        (t, n.cross(t))
    }

    /// One prefiltered texel: ibl.wgsl's `fs_prefilter` (GGX importance
    /// sampling with the source level picked from each sample's solid angle).
    fn prefilter(cube: &Cube, n: Vec3, roughness: f32, count: u32) -> Vec3 {
        let (t, b) = tangent_frame(n);
        let a = roughness * roughness;
        let a2 = a * a;
        let size = cube.levels[0].0 as f32;
        let sa_texel = 4.0 * PI / (6.0 * size * size);
        let mut sum = Vec3::ZERO;
        let mut weight = 0.0;
        for i in 0..count {
            let xi = (i as f32 / count as f32, radical_inverse(i));
            let phi = 2.0 * PI * xi.0;
            let cos_theta = ((1.0 - xi.1) / (1.0 + (a2 - 1.0) * xi.1)).sqrt();
            let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
            let h = (t * (phi.cos() * sin_theta) + b * (phi.sin() * sin_theta) + n * cos_theta)
                .normalize();
            let l = (2.0 * n.dot(h) * h - n).normalize();
            let n_dot_l = n.dot(l);
            if n_dot_l > 0.0 {
                let n_dot_h = n.dot(h).max(0.0);
                let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
                let ggx = a2 / (PI * d * d + 1e-7);
                let pdf = ggx * n_dot_h / (4.0 * n_dot_h.max(1e-4)) + 1e-4;
                let sa_sample = 1.0 / (count as f32 * pdf + 1e-4);
                let lod = 0.5 * (sa_sample / sa_texel).log2() + 1.0;
                sum += cube.sample(l, lod.max(0.0)) * n_dot_l;
                weight += n_dot_l;
            }
        }
        sum / weight.max(1e-4)
    }

    /// Runs `f` for each of the six faces, on threads where there are any.
    fn per_face<T: Send>(f: impl Fn(u32) -> T + Sync) -> Vec<T> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::scope(|s| {
                let f = &f;
                let handles: Vec<_> = (0..6).map(|face| s.spawn(move || f(face))).collect();
                handles
                    .into_iter()
                    .map(|h| h.join().expect("bake thread"))
                    .collect()
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            (0..6).map(f).collect()
        }
    }

    /// Bakes `src` into a prefiltered map: level 0 the radiance at
    /// `desc.size`, level `m` GGX-prefiltered at roughness `m / 4`, and the
    /// irradiance projected from level 0 onto order-2 harmonics (each
    /// texel weighted by its solid angle) and convolved with the cosine lobe.
    pub fn bake(src: &Equirect, desc: &BakeDesc) -> Result<EnvironmentMapData, String> {
        if src.width < 2 || src.height < 1 {
            return Err(format!("a {}x{} image is too small", src.width, src.height));
        }
        if src.rgb.len() != (src.width * src.height * 3) as usize {
            return Err("the image's pixels do not match its size".into());
        }
        bake_from(&|d| src.sample(d), desc)
    }

    /// [`bake`] from a cube map: the same radiance cube (2x2 samples per
    /// texel, from the source level of at most twice its face size, so a
    /// larger source is box-filtered first), prefiltered levels and
    /// irradiance.
    pub fn bake_cube(src: &CubeFaces, desc: &BakeDesc) -> Result<EnvironmentMapData, String> {
        if src.size == 0 {
            return Err("a cube map of 0x0 faces".into());
        }
        if src.rgb.len() != 6 * (src.size * src.size * 3) as usize {
            return Err("the cube's texels do not match its face size".into());
        }
        let cube = src.cube();
        bake_from(&|d| cube.resample(d, desc.size), desc)
    }

    /// A map from a cube's own prefiltered levels, without filtering them
    /// again: level `m` of the map (roughness `m / 4`) reads the source at
    /// lod `m / 4 * (levels - 1)`, trilinearly between its two nearest
    /// levels, at each texel's centre, turned by `desc.rotation_degrees`
    /// and times `desc.intensity`; a level whose lod is a whole level of
    /// the source's size, unturned, is copied texel for texel (so a chain
    /// of five levels from `desc.size` is taken exactly). The irradiance
    /// is projected from level 0 as [`bake`]'s. `desc.size` is the source's
    /// level 0 size and `desc.samples` is not used.
    pub fn from_prefiltered(
        src: &PrefilteredCube,
        desc: &BakeDesc,
    ) -> Result<EnvironmentMapData, String> {
        check_desc(desc)?;
        if src.size != desc.size {
            return Err(format!(
                "the source's faces are {0}², the map's {1}²: a prefiltered chain is taken at its own size",
                src.size, desc.size
            ));
        }
        if src.levels.len() < 2 {
            return Err(format!(
                "{} level(s): a prefiltered cube brings its mip chain (at least two levels)",
                src.levels.len()
            ));
        }
        let mut levels = Vec::with_capacity(src.levels.len());
        for (k, rgb) in src.levels.iter().enumerate() {
            let n = (src.size >> k).max(1);
            if rgb.len() != 6 * (n * n * 3) as usize {
                return Err(format!(
                    "level {k} holds {} floats; six {n}² faces take {}",
                    rgb.len(),
                    6 * n * n * 3
                ));
            }
            let texels = rgb
                .as_chunks::<3>()
                .0
                .iter()
                .map(|c| Vec3::from_array(*c) * desc.intensity)
                .collect();
            levels.push((n, texels));
        }
        let cube = Cube { levels };
        let top = (cube.levels.len() - 1) as f32;
        let turn = desc.rotation_degrees.to_radians();
        let (c, s) = (turn.cos(), turn.sin());
        let mut out: Vec<Vec<Vec3>> = Vec::with_capacity(ENVIRONMENT_MAP_LEVELS as usize);
        for m in 0..ENVIRONMENT_MAP_LEVELS {
            let n = desc.size >> m;
            let lod = m as f32 / (ENVIRONMENT_MAP_LEVELS - 1) as f32 * top;
            let whole = lod.fract() == 0.0 && cube.levels[lod as usize].0 == n;
            if whole && turn == 0.0 {
                out.push(cube.levels[lod as usize].1.clone());
                continue;
            }
            let faces = per_face(|face| {
                let mut o = Vec::with_capacity((n * n) as usize);
                for j in 0..n {
                    for i in 0..n {
                        let d = texel_direction(face, n, i, j, 0.5, 0.5);
                        let d = Vec3::new(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
                        o.push(cube.sample(d, lod));
                    }
                }
                o
            });
            out.push(faces.into_iter().flatten().collect());
        }
        let texels = out.iter().flatten().map(|c| rgb9e5(c.to_array())).collect();
        let map = EnvironmentMapData {
            size: desc.size,
            texels,
            irradiance: irradiance_of(desc.size, &out[0]),
        };
        map.validate()?;
        Ok(map)
    }

    fn check_desc(desc: &BakeDesc) -> Result<(), String> {
        let size = desc.size;
        if !size.is_power_of_two()
            || !(ENVIRONMENT_MAP_MIN_SIZE..=ENVIRONMENT_MAP_MAX_SIZE).contains(&size)
        {
            return Err(format!(
                "size {size}: a power of two from {ENVIRONMENT_MAP_MIN_SIZE} to {ENVIRONMENT_MAP_MAX_SIZE}"
            ));
        }
        if !(desc.intensity.is_finite() && desc.intensity >= 0.0) {
            return Err(format!("intensity {}: 0 or more", desc.intensity));
        }
        Ok(())
    }

    fn bake_from(
        src: &dyn Fn(Vec3) -> Vec3,
        desc: &BakeDesc,
    ) -> Result<EnvironmentMapData, String> {
        check_desc(desc)?;
        let size = desc.size;
        let cube = radiance_cube(
            src,
            size,
            desc.rotation_degrees.to_radians(),
            desc.intensity,
        );
        let mut texels: Vec<u32> = cube.levels[0]
            .1
            .iter()
            .map(|c| rgb9e5(c.to_array()))
            .collect();
        for level in 1..ENVIRONMENT_MAP_LEVELS {
            let n = size >> level;
            let roughness = level as f32 / (ENVIRONMENT_MAP_LEVELS - 1) as f32;
            let faces = per_face(|face| {
                let mut out = Vec::with_capacity((n * n) as usize);
                for j in 0..n {
                    for i in 0..n {
                        let d = texel_direction(face, n, i, j, 0.5, 0.5);
                        out.push(rgb9e5(
                            prefilter(&cube, d, roughness, desc.samples.max(1)).to_array(),
                        ));
                    }
                }
                out
            });
            texels.extend(faces.into_iter().flatten());
        }
        let (n, base) = &cube.levels[0];
        let map = EnvironmentMapData {
            size,
            texels,
            irradiance: irradiance_of(*n, base),
        };
        map.validate()?;
        Ok(map)
    }

    /// The irradiance over pi of a radiance cube of face size `n`: the
    /// radiance projected with each texel's solid angle, then the cosine
    /// lobe's band factors (1, 2/3, 1/4) for E / pi.
    fn irradiance_of(n: u32, base: &[Vec3]) -> [[f32; 3]; 9] {
        let n = &n;
        let mut coeffs = [Vec3::ZERO; 9];
        for face in 0..6 {
            for j in 0..*n {
                for i in 0..*n {
                    let x = 2.0 * (i as f32 + 0.5) / *n as f32 - 1.0;
                    let y = 1.0 - 2.0 * (j as f32 + 0.5) / *n as f32;
                    let texel = 2.0 / *n as f32;
                    let solid = texel * texel / (1.0 + x * x + y * y).powf(1.5);
                    let d = face_direction(face, x, y);
                    let l = base[((face * *n + j) * *n + i) as usize];
                    for (c, y) in coeffs.iter_mut().zip(sh9(d)) {
                        *c += l * (y * solid);
                    }
                }
            }
        }
        let band = [
            1.0,
            2.0 / 3.0,
            2.0 / 3.0,
            2.0 / 3.0,
            0.25,
            0.25,
            0.25,
            0.25,
            0.25,
        ];
        std::array::from_fn(|k| (coeffs[k] * band[k]).to_array())
    }

    /// The irradiance over pi of `map` facing unit `n` (what the renderer's
    /// irradiance cube holds there, before any rotation).
    pub fn irradiance(map: &EnvironmentMapData, n: Vec3) -> Vec3 {
        map.irradiance
            .iter()
            .zip(sh9(n))
            .map(|(c, y)| Vec3::from_array(*c) * y)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb9e5_round_trips_within_its_precision() {
        for rgb in [
            [0.0, 0.0, 0.0],
            [1.0, 0.5, 0.25],
            [100.0, 3.0, 0.01],
            [65408.0, 1.0, 0.0],
            [0.18, 0.18, 0.18],
            [1e-6, 2e-6, 0.0],
        ] {
            let back = rgb9e5_decode(rgb9e5(rgb));
            let max = rgb.iter().cloned().fold(0.0f32, f32::max);
            for (a, b) in rgb.iter().zip(back) {
                // Below 2^-15 the step is the smallest exponent's, 2^-24.
                assert!((a - b).abs() <= max / 256.0 + 6e-8, "{rgb:?} -> {back:?}");
            }
        }
        assert_eq!(
            rgb9e5_decode(rgb9e5([1e9, -1.0, f32::NAN])),
            [65408.0, 0.0, 0.0]
        );
        // Exactly representable values survive.
        assert_eq!(rgb9e5_decode(rgb9e5([1.0, 0.5, 2.0])), [1.0, 0.5, 2.0]);
    }

    #[test]
    fn face_directions_follow_the_cube_convention() {
        use glam::Vec3;
        assert_eq!(face_direction(0, 0.0, 0.0), Vec3::X);
        assert_eq!(face_direction(1, 0.0, 0.0), -Vec3::X);
        assert_eq!(face_direction(2, 0.0, 0.0), Vec3::Y);
        assert_eq!(face_direction(3, 0.0, 0.0), -Vec3::Y);
        assert_eq!(face_direction(4, 0.0, 0.0), Vec3::Z);
        assert_eq!(face_direction(5, 0.0, 0.0), -Vec3::Z);
        // +x face: right is -z, up is +y.
        assert!(face_direction(0, 1.0, 0.0).z < 0.0);
        assert!(face_direction(0, 0.0, 1.0).y > 0.0);
    }

    /// A cube's own prefiltered chain is taken, not filtered again: five
    /// levels from the map's size are copied texel for texel (the bake's
    /// own levels read back come out as they went in), and a shorter chain
    /// is read by roughness, level m at lod m / 4 * (levels - 1).
    #[cfg(feature = "bake")]
    #[test]
    fn a_prefiltered_chain_is_taken_as_is() {
        use super::bake::{
            BakeDesc, Equirect, PrefilteredCube, bake, from_prefiltered, irradiance,
        };
        let (w, h) = (32u32, 16u32);
        let rgb: Vec<f32> = (0..w * h)
            .flat_map(|i| {
                let (x, y) = ((i % w) as f32 / w as f32, (i / w) as f32 / h as f32);
                [0.2 + 2.0 * x, 1.5 * (1.0 - y), 0.3 + x * y]
            })
            .collect();
        let desc = BakeDesc {
            size: 16,
            samples: 32,
            ..Default::default()
        };
        let baked = bake(
            &Equirect {
                width: w,
                height: h,
                rgb: &rgb,
            },
            &desc,
        )
        .unwrap();
        let mut levels = Vec::new();
        let mut at = 0;
        for m in 0..ENVIRONMENT_MAP_LEVELS {
            let count = 6 * ((16u32 >> m) as usize).pow(2);
            levels.push(
                baked.texels[at..at + count]
                    .iter()
                    .flat_map(|t| rgb9e5_decode(*t))
                    .collect::<Vec<f32>>(),
            );
            at += count;
        }
        let src = PrefilteredCube {
            size: 16,
            levels: &levels,
        };
        let taken = from_prefiltered(&src, &desc).unwrap();
        assert_eq!(taken.texels, baked.texels, "five levels are copied");
        for n in [glam::Vec3::Y, glam::Vec3::X, -glam::Vec3::Z] {
            let (a, b) = (irradiance(&taken, n), irradiance(&baked, n));
            assert!(
                (a - b).abs().max_element() < 0.01 * b.max_element(),
                "{a} {b}"
            );
        }
        // Three levels, each one value: level m reads lod m / 2.
        let flat: Vec<Vec<f32>> = (0..3u32)
            .map(|k| vec![k as f32 + 1.0; 6 * ((16u32 >> k) as usize).pow(2) * 3])
            .collect();
        let src = PrefilteredCube {
            size: 16,
            levels: &flat,
        };
        let turned = BakeDesc {
            rotation_degrees: 30.0,
            intensity: 2.0,
            ..desc
        };
        let map = from_prefiltered(&src, &turned).unwrap();
        let mut at = 0;
        for m in 0..ENVIRONMENT_MAP_LEVELS {
            let count = 6 * ((16u32 >> m) as usize).pow(2);
            let want = 2.0 * (1.0 + m as f32 / 2.0);
            for t in &map.texels[at..at + count] {
                for c in rgb9e5_decode(*t) {
                    assert!((c - want).abs() < want / 200.0, "level {m}: {c} not {want}");
                }
            }
            at += count;
        }
        // One level is not a chain; another size is not this chain's.
        assert!(
            from_prefiltered(
                &PrefilteredCube {
                    size: 16,
                    levels: &flat[..1]
                },
                &desc
            )
            .is_err()
        );
        let other = BakeDesc { size: 32, ..desc };
        assert!(from_prefiltered(&src, &other).unwrap_err().contains("16²"));
    }

    #[cfg(feature = "bake")]
    #[test]
    fn a_uniform_sky_bakes_to_itself() {
        use super::bake::{BakeDesc, Equirect, bake, irradiance};
        let rgb = vec![0.5f32; 16 * 8 * 3];
        let map = bake(
            &Equirect {
                width: 16,
                height: 8,
                rgb: &rgb,
            },
            &BakeDesc {
                size: 16,
                samples: 64,
                ..Default::default()
            },
        )
        .expect("bake");
        assert_eq!(map.texels.len(), EnvironmentMapData::texel_count(16));
        for t in &map.texels {
            for c in rgb9e5_decode(*t) {
                assert!((c - 0.5).abs() < 0.01, "{c}");
            }
        }
        // A white surface under a uniform radiance of 0.5 reflects 0.5.
        for n in [glam::Vec3::Y, glam::Vec3::X, -glam::Vec3::Z] {
            let e = irradiance(&map, n);
            assert!(
                (e - glam::Vec3::splat(0.5)).abs().max_element() < 0.01,
                "{e}"
            );
        }
    }

    #[cfg(feature = "bake")]
    #[test]
    fn the_sky_above_lights_upward_faces_and_rotation_turns_the_map() {
        use super::bake::{BakeDesc, Equirect, bake, irradiance};
        // Bright upper half, dark lower half; a red patch at u = 0.5 (+x).
        let (w, h) = (64u32, 32u32);
        let mut rgb = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let up = y < h / 2;
                let red = (28..36).contains(&x) && (12..20).contains(&y);
                rgb.extend(if red {
                    [8.0, 0.0, 0.0]
                } else if up {
                    [1.0, 1.0, 1.0]
                } else {
                    [0.1, 0.1, 0.1]
                });
            }
        }
        let src = Equirect {
            width: w,
            height: h,
            rgb: &rgb,
        };
        let desc = BakeDesc {
            size: 32,
            samples: 64,
            ..Default::default()
        };
        let map = bake(&src, &desc).expect("bake");
        let up = irradiance(&map, glam::Vec3::Y);
        let down = irradiance(&map, -glam::Vec3::Y);
        assert!(up.y > 4.0 * down.y, "{up} {down}");
        let red = |m: &EnvironmentMapData, face: u32| {
            // The centre texel of `face` at level 0.
            let n = m.size as usize;
            rgb9e5_decode(m.texels[face as usize * n * n + (n / 2) * n + n / 2])[0]
        };
        assert!(red(&map, 0) > 4.0, "the patch faces +x: {}", red(&map, 0));
        // Turned 90 degrees, +x moves towards -z.
        let turned = bake(
            &src,
            &BakeDesc {
                rotation_degrees: 90.0,
                ..desc
            },
        )
        .expect("bake");
        assert!(red(&turned, 5) > 4.0, "turned to -z: {}", red(&turned, 5));
        assert!(red(&turned, 0) < 1.5);
    }

    /// A smooth sky: brighter above, warm, a hot lobe towards +x.
    #[cfg(feature = "bake")]
    fn sky(d: glam::Vec3) -> [f32; 3] {
        let lobe = 4.0 * d.x.max(0.0).powi(8);
        let up = 1.0 + d.y;
        [up + lobe, 0.8 * up, 0.6 * up]
    }

    /// The same sky given as cube faces bakes as it does given as an
    /// equirectangular image: within 2 % in irradiance and a few percent
    /// per texel of every level; a source cube larger than twice the bake
    /// is box-filtered first, so 256² faces bake like 64² ones.
    #[cfg(feature = "bake")]
    #[test]
    fn a_cube_bakes_as_its_equirectangular_twin() {
        use super::bake::{BakeDesc, CubeFaces, Equirect, bake, bake_cube, irradiance};
        use std::f32::consts::PI;
        let (w, h) = (256u32, 128u32);
        let mut equirect = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * 2.0 * PI;
                let lat = (0.5 - (y as f32 + 0.5) / h as f32) * PI;
                let d = glam::Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
                equirect.extend(sky(d));
            }
        }
        let cube = |n: u32| {
            let mut rgb = Vec::new();
            for face in 0..6 {
                for j in 0..n {
                    for i in 0..n {
                        let x = 2.0 * (i as f32 + 0.5) / n as f32 - 1.0;
                        let y = 1.0 - 2.0 * (j as f32 + 0.5) / n as f32;
                        rgb.extend(sky(face_direction(face, x, y)));
                    }
                }
            }
            rgb
        };
        let desc = BakeDesc {
            size: 32,
            samples: 64,
            rotation_degrees: 30.0,
            intensity: 1.5,
        };
        let a = bake(
            &Equirect {
                width: w,
                height: h,
                rgb: &equirect,
            },
            &desc,
        )
        .unwrap();
        let faces64 = cube(64);
        let b = bake_cube(
            &CubeFaces {
                size: 64,
                rgb: &faces64,
            },
            &desc,
        )
        .unwrap();
        for n in [
            glam::Vec3::Y,
            -glam::Vec3::Y,
            glam::Vec3::X,
            glam::Vec3::Z,
            -glam::Vec3::X,
        ] {
            let (ea, eb) = (irradiance(&a, n), irradiance(&b, n));
            assert!(
                ((ea - eb).abs() / ea.max(glam::Vec3::splat(0.05))).max_element() < 0.02,
                "{n}: {ea} {eb}"
            );
        }
        let diff = |x: &EnvironmentMapData, y: &EnvironmentMapData| {
            let mut worst = 0f32;
            let mut sum = 0f32;
            for (p, q) in x.texels.iter().zip(&y.texels) {
                let (p, q) = (rgb9e5_decode(*p), rgb9e5_decode(*q));
                for c in 0..3 {
                    let d = (p[c] - q[c]).abs() / p[c].max(q[c]).max(0.1);
                    worst = worst.max(d);
                    sum += d;
                }
            }
            (sum / (3 * x.texels.len()) as f32, worst)
        };
        let (mean, worst) = diff(&a, &b);
        assert!(mean < 0.01 && worst < 0.1, "mean {mean} worst {worst}");
        // A 256² source is filtered down to the 64² level the bake reads.
        let faces256 = cube(256);
        let c = bake_cube(
            &CubeFaces {
                size: 256,
                rgb: &faces256,
            },
            &desc,
        )
        .unwrap();
        let (mean, worst) = diff(&b, &c);
        assert!(mean < 0.005 && worst < 0.05, "mean {mean} worst {worst}");
        // Faces of the wrong size are refused.
        assert!(
            bake_cube(
                &CubeFaces {
                    size: 63,
                    rgb: &faces64
                },
                &desc
            )
            .is_err()
        );
    }
}
