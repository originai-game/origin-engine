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
    /// The sun the bake took out of the image ([`bake::BakeDesc::sun`]);
    /// intensity 0: none.
    pub sun: EnvironmentSun,
}

/// The sun a bake took out of an environment map, to put back as a
/// directional light: the texels and the irradiance no longer hold it, so
/// the light is counted once, casts shadows, and the prefiltered levels
/// carry no speckle from it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EnvironmentSun {
    /// The unit direction towards it, in the map's own frame (the bake's
    /// turn applied, a draw's rotation not).
    pub direction: [f32; 3],
    /// Its colour, the largest channel 1.
    pub color: [f32; 3],
    /// With `color`, the irradiance it gave a surface facing it (the
    /// radiance taken out times its solid angle): a directional light's
    /// intensity. 0: the image has no sun.
    pub intensity: f32,
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

/// An equirectangular HDR image the renderer prefilters itself, on the GPU
/// when the map is uploaded ([`crate::RenderScene::add_environment_equirect`]:
/// uncooked environments loaded at run time). The same levels as a cooked
/// map's (level `m` GGX-prefiltered at roughness `m / 4`, by the passes the
/// procedural sky's own prefilter runs), in RGBA16F instead of RGB9E5.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvironmentEquirect {
    pub width: u32,
    pub height: u32,
    /// RGB9E5 texels ([`rgb9e5`]), rows from the top (+y): column `u`
    /// looks towards longitude `(u - 0.5) * 2 pi` from +x towards +z, as
    /// three.js maps an equirectangular texture (the cook's `Equirect`).
    pub texels: Vec<u32>,
    /// Level 0's face size: a power of two from
    /// [`ENVIRONMENT_MAP_MIN_SIZE`] to [`ENVIRONMENT_MAP_MAX_SIZE`].
    pub size: u32,
}

impl EnvironmentEquirect {
    /// Checks the sizes and the texel count.
    pub fn validate(&self) -> Result<(), String> {
        if !self.size.is_power_of_two()
            || !(ENVIRONMENT_MAP_MIN_SIZE..=ENVIRONMENT_MAP_MAX_SIZE).contains(&self.size)
        {
            return Err(format!(
                "an environment map's size is a power of two from {ENVIRONMENT_MAP_MIN_SIZE} to {ENVIRONMENT_MAP_MAX_SIZE}, not {}",
                self.size
            ));
        }
        if self.width < 2 || self.height < 1 {
            return Err(format!(
                "a {}x{} image is too small",
                self.width, self.height
            ));
        }
        if self.texels.len() != (self.width as usize) * (self.height as usize) {
            return Err("the image's texels do not match its size".into());
        }
        Ok(())
    }
}

/// The diffuse irradiance of an equirectangular image as
/// [`EnvironmentMapData::irradiance`] holds it, gathered a row at a time
/// (so a loader can spread it over frames): each pixel's radiance
/// projected onto the nine harmonics with its solid angle, then the cosine
/// lobe's band factors, as the cook's bake projects its radiance cube.
#[derive(Clone, Debug)]
pub struct EquirectIrradiance {
    width: u32,
    height: u32,
    sum: [[f64; 3]; 9],
}

impl EquirectIrradiance {
    pub fn new(width: u32, height: u32) -> EquirectIrradiance {
        EquirectIrradiance {
            width,
            height,
            sum: [[0.0; 3]; 9],
        }
    }

    /// Row `y` (from the top), `width * 3` linear RGB floats.
    pub fn add_row(&mut self, y: u32, rgb: &[f32]) {
        use std::f64::consts::PI;
        let (w, h) = (self.width as f64, self.height as f64);
        let lat = (0.5 - (y as f64 + 0.5) / h) * PI;
        let solid = (2.0 * PI / w) * (PI / h) * lat.cos();
        for (x, c) in rgb.as_chunks::<3>().0.iter().enumerate() {
            let lon = ((x as f64 + 0.5) / w - 0.5) * 2.0 * PI;
            let d = glam::Vec3::new(
                (lat.cos() * lon.cos()) as f32,
                lat.sin() as f32,
                (lat.cos() * lon.sin()) as f32,
            );
            for (sum, y) in self.sum.iter_mut().zip(sh9(d)) {
                for (s, v) in sum.iter_mut().zip(c) {
                    *s += *v as f64 * y as f64 * solid;
                }
            }
        }
    }

    /// The harmonics once every row is in.
    pub fn finish(&self) -> [[f32; 3]; 9] {
        const BAND: [f64; 9] = [
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
        std::array::from_fn(|k| std::array::from_fn(|c| (self.sum[k][c] * BAND[k]) as f32))
    }
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
        /// Takes the sun out of the image into
        /// [`EnvironmentMapData::sun`]: within a few degrees of the
        /// brightest texel, the radiance above sixteen times the rest of
        /// the image's mean is removed (its colour kept) and summed as
        /// the sun's light. An image without so bright a spot keeps
        /// everything, and its sun's intensity is 0.
        pub sun: bool,
    }

    impl Default for BakeDesc {
        fn default() -> Self {
            BakeDesc {
                size: 128,
                intensity: 1.0,
                rotation_degrees: 0.0,
                samples: 256,
                sun: false,
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

    /// The radiance cube's level 0 at `n`: each texel the mean of
    /// `per_axis`² samples of `src`'s radiance, on a grid across it.
    fn radiance_cube(
        src: &(dyn Fn(Vec3) -> Vec3 + Sync),
        n: u32,
        per_axis: u32,
        turn: f32,
        intensity: f32,
    ) -> Vec<Vec3> {
        let (c, s) = (turn.cos(), turn.sin());
        let step = 1.0 / per_axis as f32;
        let scale = intensity * step * step;
        per_face(|face| {
            let mut out = Vec::with_capacity((n * n) as usize);
            for j in 0..n {
                for i in 0..n {
                    let mut sum = Vec3::ZERO;
                    for b in 0..per_axis {
                        for a in 0..per_axis {
                            let (di, dj) = ((a as f32 + 0.5) * step, (b as f32 + 0.5) * step);
                            let d = texel_direction(face, n, i, j, di, dj);
                            // The map turned by `turn`: look the image up
                            // at the direction turned back.
                            let d = Vec3::new(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
                            sum += src(d);
                        }
                    }
                    out.push(sum * scale);
                }
            }
            out
        })
        .into_iter()
        .flatten()
        .collect()
    }

    /// A cube texel's solid angle at face coordinates `x`, `y` (-1..1).
    fn solid_angle(n: u32, x: f32, y: f32) -> f32 {
        let texel = 2.0 / n as f32;
        texel * texel / (1.0 + x * x + y * y).powf(1.5)
    }

    const LUMA: Vec3 = Vec3::new(0.2126, 0.7152, 0.0722);
    /// How far from the brightest texel the sun's light is gathered (its
    /// disc and the glare around it), in degrees, and how many times the
    /// rest of the image's mean luminance a texel keeps.
    const SUN_CONE_DEGREES: f32 = 5.0;
    const SUN_OVER_MEAN: f32 = 16.0;

    /// [`BakeDesc::sun`]: takes the sun out of a radiance cube's level 0.
    fn take_sun(n: u32, base: &mut [Vec3]) -> super::EnvironmentSun {
        let at = |k: usize| {
            let (face, j, i) = (k as u32 / (n * n), k as u32 / n % n, k as u32 % n);
            let x = 2.0 * (i as f32 + 0.5) / n as f32 - 1.0;
            let y = 1.0 - 2.0 * (j as f32 + 0.5) / n as f32;
            (face_direction(face, x, y), solid_angle(n, x, y))
        };
        let Some(peak) =
            (0..base.len()).max_by(|&a, &b| base[a].dot(LUMA).total_cmp(&base[b].dot(LUMA)))
        else {
            return Default::default();
        };
        // A texel's centre is up to its diagonal away from the light in it.
        let reach = (SUN_CONE_DEGREES + 1.5 * 90.0 / n as f32)
            .to_radians()
            .cos();
        let (towards, _) = at(peak);
        let inside = |k: usize| at(k).0.dot(towards) >= reach;
        let (mut rest, mut area) = (0.0, 0.0);
        for k in (0..base.len()).filter(|&k| !inside(k)) {
            let solid = at(k).1;
            rest += base[k].dot(LUMA) * solid;
            area += solid;
        }
        let keep = SUN_OVER_MEAN * rest / area;
        let (mut light, mut aim) = (Vec3::ZERO, Vec3::ZERO);
        for k in (0..base.len()).filter(|&k| inside(k)) {
            let luma = base[k].dot(LUMA);
            if luma > keep && luma > 0.0 {
                let (d, solid) = at(k);
                let taken = base[k] * ((1.0 - keep / luma) * solid);
                light += taken;
                aim += d * taken.dot(LUMA);
                base[k] *= keep / luma;
            }
        }
        let intensity = light.max_element();
        if intensity <= 0.0 {
            return Default::default();
        }
        super::EnvironmentSun {
            direction: aim.try_normalize().unwrap_or(towards).to_array(),
            color: (light / intensity).to_array(),
            intensity,
        }
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
        // Samples closer together than the image's pixels, so a sun of a
        // few pixels gives its light wherever it falls: a texel at a
        // face's centre spans `2 / size` radians and a pixel at the
        // horizon `2 pi / width`, less by its latitude's cosine above it
        // (the half again covers pixels up to 48 degrees).
        let per_axis = (1.5 * src.width as f32 / (PI * desc.size.max(1) as f32)).ceil() as u32;
        bake_from(&|d| src.sample(d), per_axis.max(2), desc)
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
        bake_from(&|d| cube.resample(d, desc.size), 2, desc)
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
            sun: Default::default(),
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
        src: &(dyn Fn(Vec3) -> Vec3 + Sync),
        per_axis: u32,
        desc: &BakeDesc,
    ) -> Result<EnvironmentMapData, String> {
        check_desc(desc)?;
        let size = desc.size;
        let mut base = radiance_cube(
            src,
            size,
            per_axis,
            desc.rotation_degrees.to_radians(),
            desc.intensity,
        );
        let sun = if desc.sun {
            take_sun(size, &mut base)
        } else {
            Default::default()
        };
        let cube = mips(size, base);
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
            sun,
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
                    let solid = solid_angle(*n, x, y);
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
