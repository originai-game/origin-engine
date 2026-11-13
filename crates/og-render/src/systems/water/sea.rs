use super::*;

/// [`WaterGpu::shore`] of `water` at the tick's `seconds`.
pub(super) fn shore_gpu(water: &Water, seconds: f64) -> [[f32; 4]; 3] {
    let Some(s) = water.shore.as_ref().filter(|_| water.mesh.is_none()) else {
        return [[0.0; 4]; 3];
    };
    let t = &s.trains;
    [
        [s.origin.x, s.origin.y, s.field.cell, 1.0],
        [
            s.field.columns as f32,
            s.field.rows as f32,
            t.k,
            t.amplitude,
        ],
        [
            t.omega,
            og_env::ocean::Ocean::train_phase(t, seconds),
            s.foam.max(0.0),
            0.0,
        ],
    ]
}

/// The tick's coast of `water`'s sea in the scene whose frame's zero is at
/// `origin` (absolute world): its seabed, the trains, and the absolute
/// world xz of the zero of the frame the field was made in.
pub(super) fn coast(water: &Water, origin: Vec3) -> Option<og_env::ocean::Coast<'_>> {
    let s = water.shore.as_ref()?;
    let shift = s.origin - s.field.origin;
    Some(og_env::ocean::Coast {
        shore: &s.field,
        trains: s.trains,
        zero: glam::DVec2::new(f64::from(origin.x), f64::from(origin.z)) + shift.as_dvec2(),
    })
}

/// The texture of `water`'s seabed among this frame's `shores` (made in
/// `prepare`), or `none`.
pub(super) fn shore_view<'a>(
    shores: &'a [(Arc<og_env::ocean::Shore>, wgpu::TextureView)],
    none: &'a wgpu::TextureView,
    water: &Water,
) -> &'a wgpu::TextureView {
    water
        .shore
        .as_ref()
        .filter(|_| water.mesh.is_none())
        .and_then(|ws| shores.iter().find(|(f, _)| Arc::ptr_eq(f, &ws.field)))
        .map_or(none, |(_, v)| v)
}

/// The seabed's texture: per sample its depth, distance and the distance's
/// gradient, rgba32float (read with textureLoad and filtered in water.wgsl
/// `shore_at`, as `Shore::sample` filters them, so both read the same).
pub(super) fn shore_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    field: &og_env::ocean::Shore,
) -> (wgpu::Texture, wgpu::TextureView) {
    let size = wgpu::Extent3d {
        width: field.columns,
        height: field.rows,
        depth_or_array_layers: 1,
    };
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("water shore"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    if field.columns * field.rows > 0 {
        let texels: Vec<[f32; 4]> = field.texels.iter().map(|v| v.to_array()).collect();
        queue.write_texture(
            t.as_image_copy(),
            bytemuck::cast_slice(&texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(field.columns * 16),
                rows_per_image: None,
            },
            size,
        );
    }
    let v = t.create_view(&Default::default());
    (t, v)
}

/// The scene's clock for the sea, in f64 seconds: its tick over its rate,
/// as the simulation's `og.physics.waterHeight` reads it, and on a display
/// frame between ticks ([`crate::RenderScene::interpolation`]) the time
/// between the last two ticks the blended transforms are drawn at, so the
/// sea moves at the display's rate under floaters drawn in step with it.
pub(crate) fn seconds(scene: &crate::RenderScene) -> f64 {
    let rate = if scene.frame.tick_rate > 0.0 {
        scene.frame.tick_rate
    } else {
        60.0
    };
    let ticks = match scene.frame.interpolation {
        Some(a) if scene.frame.tick > 0 => {
            scene.frame.tick as f64 - 1.0 + f64::from(a.clamp(0.0, 1.0))
        }
        _ => scene.frame.tick as f64,
    };
    ticks / f64::from(rate)
}

/// The surface's sea, if any (a grid with a wind: [`Water::wind`]).
pub(crate) fn ocean(water: &Water) -> Option<og_env::ocean::Ocean> {
    if water.mesh.is_some() {
        return None;
    }
    og_env::ocean::Ocean::new(water.wind, water.swell, water.choppiness)
}

/// The sea's block of [`WaterGpu`] at `seconds` and the scene's `origin`:
/// the header and og_core's waves, their phases at the origin in f64 so the
/// pattern stays put in the world as the origin moves. The vertices draw
/// every one of them as the tick floats bodies on them
/// ([`og_env::ocean::Ocean::surface`]), each left out only where the
/// clipmap's own vertices cannot hold it ([`sea_keep`]); the FFT's cascades
/// add the ripples shorter than the shortest of them.
pub(super) fn ocean_gpu(
    water: &Water,
    seconds: f64,
    origin: Vec3,
) -> ([f32; 4], [[f32; 4]; 2 * og_env::ocean::OCEAN_WAVES]) {
    let mut swell = [[0.0; 4]; 2 * og_env::ocean::OCEAN_WAVES];
    let Some(sea) = ocean(water) else {
        return ([0.0; 4], swell);
    };
    let phases = sea.phases(
        seconds,
        glam::DVec2::new(f64::from(origin.x), f64::from(origin.z)),
    );
    for (i, (w, phase)) in sea.waves.iter().zip(phases).enumerate() {
        let k = w.direction * w.k;
        let lean = w.direction * w.lean;
        swell[2 * i] = [k.x, k.y, w.amplitude, phase];
        swell[2 * i + 1] = [lean.x, lean.y, w.k * w.lean, 0.0];
    }
    // z and w: the foam's threshold and gain, [`foam_thresholds`]'s (set
    // with the FFT sea's).
    ([1.0, 0.0, 0.0, FRESH_GAIN], swell)
}

/// How much of a wave of wavenumber `k` samples `cell` metres apart hold:
/// all of one at least eight cells long, none of one under four (water_sea.wgsl
/// `sea_keep`: the clipmap's vertices, a pixel's footprint, a foam texel).
pub(super) fn sea_keep(k: f32, cell: f32) -> f32 {
    let x = ((k * cell - 0.785) / 0.786).clamp(0.0, 1.0);
    1.0 - x * x * (3.0 - 2.0 * x)
}

/// The least a sea's foam grows to full by as the Jacobian falls under its
/// threshold ([`foam_thresholds`], which steepens it for a sea whose
/// surface lies close under its level): in the frame (web-ocean's `foamP`)
/// and into the foam that persists ([`crate::ocean_fft::foam`]; its
/// `FOAM_UPDATE`'s).
pub(super) const FRESH_GAIN: f32 = 2.5;
pub(super) const FOAM_GAIN: f32 = 2.0;

/// The Jacobian of `water`'s drawn surface at seeded points, sorted, its
/// waves resolved at `resolve` metres (0: all of them, as drawn).
pub(super) fn jacobians(
    water: &Water,
    built: &crate::ocean_fft::spectrum::Spectrum,
    resolve: f32,
) -> Vec<f64> {
    let lam = built.choppiness;
    // The covariance of the cascades' (∂Dx/∂x, ∂Dz/∂z, ∂Dx/∂z) and its
    // Cholesky factor.
    let mut c = [[0f64; 3]; 3];
    for cs in &built.cascades {
        let n = cs.resolution as i64;
        let dk = std::f64::consts::TAU / cs.size;
        // Read from the fields' mip at ocean_foam.wgsl's lod: a box of 2^lod
        // texels, whose response to a wave is the product of sincs along
        // its axes (none past the shader's last lod).
        let texel = cs.size / n as f64;
        let lod = (f64::from(resolve) / texel).max(1.0).log2();
        if lod >= 4.5 {
            continue;
        }
        let width = texel * lod.exp2();
        let sinc = |x: f64| if x.abs() < 1e-9 { 1.0 } else { x.sin() / x };
        let signed = |i: i64| if i >= n / 2 { i - n } else { i } as f64 * dk;
        for (t, a) in cs.amplitudes.chunks(4).enumerate() {
            let a2 = f64::from(a[0]).powi(2) + f64::from(a[1]).powi(2);
            let (kx, kz) = (signed(t as i64 % n), signed(t as i64 / n));
            let k = kx.hypot(kz);
            if a2 == 0.0 || k < 1e-9 {
                continue;
            }
            let mip = if resolve > 0.0 {
                sinc(kx * width * 0.5) * sinc(kz * width * 0.5)
            } else {
                1.0
            };
            let v = [
                lam * kx * kx / k * mip,
                lam * kz * kz / k * mip,
                lam * kx * kz / k * mip,
            ];
            for r in 0..3 {
                for q in 0..3 {
                    c[r][q] += a2 * v[r] * v[q];
                }
            }
        }
    }
    let mut l = [[0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..=i {
            let s = c[i][j] - (0..j).map(|k| l[i][k] * l[j][k]).sum::<f64>();
            l[i][j] = if i == j {
                s.max(0.0).sqrt()
            } else if l[j][j] > 0.0 {
                s / l[j][j]
            } else {
                0.0
            };
        }
    }
    let (_, swell) = ocean_gpu(water, 0.0, Vec3::ZERO);
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut uniform = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    };
    const SAMPLES: usize = 32_768;
    let mut jac: Vec<f64> = (0..SAMPLES)
        .map(|_| {
            let p = Vec2::new(
                (uniform() * 512.0 - 256.0) as f32,
                (uniform() * 512.0 - 256.0) as f32,
            );
            let mut j = [0f64; 3];
            for w in swell.chunks(2) {
                let (a, b) = (w[0], w[1]);
                let k = Vec2::new(a[0], a[1]);
                let s = f64::from((k.dot(p) + a[3]).sin() * sea_keep(k.length(), resolve));
                j[0] -= f64::from(b[0] * a[0]) * s;
                j[1] -= f64::from(b[1] * a[1]) * s;
                j[2] -= f64::from(b[0] * a[1]) * s;
            }
            let mut g = [0f64; 3];
            for v in &mut g {
                let (u1, u2) = (uniform(), uniform());
                *v = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
            }
            for (i, row) in l.iter().enumerate() {
                j[i] += (0..=i).map(|k| row[k] * g[k]).sum::<f64>();
            }
            (1.0 + j[0]) * (1.0 + j[1]) - j[2] * j[2]
        })
        .collect();
    jac.sort_by(f64::total_cmp);
    jac
}

/// Levels of a sea's clipmap ([`clipmap`]).
pub(super) const CLIP_LEVELS: usize = 10;
/// Half the edge of the outermost level (m), where the horizon ring takes
/// over.
const CLIP_REACH: f64 = 4096.0;

/// A sea's clipmap about the eye at `eye` (frame coordinates) of `segments`
/// cells along each level's edge, after blue-water's (`geometry.rs`): each
/// level twice the cell and extent of the one inside it, the outermost
/// reaching [`CLIP_REACH`]; each centred on the eye snapped to twice its own
/// cell in the absolute world (the frame plus `origin`, in f64), so its
/// vertices sit on fixed world points as the eye and the origin move, and
/// the level inside it lies on its lattice. Per level: its centre (frame
/// xz) and its cell (m).
pub(super) fn clipmap(eye: Vec3, origin: Vec3, segments: u32) -> [[f32; 4]; CLIP_LEVELS] {
    let first = 2.0 * CLIP_REACH / f64::from(segments) / f64::from(1u32 << (CLIP_LEVELS - 1));
    let (ox, oz) = (f64::from(origin.x), f64::from(origin.z));
    let (x, z) = (f64::from(eye.x) + ox, f64::from(eye.z) + oz);
    std::array::from_fn(|level| {
        let cell = first * f64::from(1u32 << level);
        let step = 2.0 * cell;
        let snap = |v: f64| (v / step).round() * step;
        [
            (snap(x) - ox) as f32,
            (snap(z) - oz) as f32,
            cell as f32,
            0.0,
        ]
    })
}

/// A sea's clipmap cells along a level's edge and its FFT cascades, by the
/// frame's quality: desktop presets 128 and three, the low preset (phones)
/// 64 and two. A vertex d metres from the camera lies at most d / (s / 4)
/// from the next (s the cells along an edge, with the edge's geomorph): on
/// desktop under 0.25 m within 8 m and under 3.5 m within 110 m, so a wave
/// eight times that long is drawn whole there.
pub(super) fn sea_tier(quality: &crate::RenderQuality) -> (u32, u32) {
    if quality.volumetric_step_scale < 1.0 {
        (64, 2)
    } else {
        (128, 3)
    }
}

/// The vertices of a sea's draw of `segments` cells per level edge: every
/// level's square of cells (the hole each holds for the level inside it
/// collapses), then the horizon ring's quads round the outermost.
pub(super) fn sea_vertices(segments: u32) -> u32 {
    CLIP_LEVELS as u32 * segments * segments * 6 + 2 * segments * 6
}

/// Each cascade's [`WaterGpu::cascades`] row at the scene's `origin`: one
/// over its tile, the uv of the frame's origin in it (f64, whole tiles
/// dropped: the pattern stays put in the world as the origin moves) and the
/// tile. Zero past the sea's cascades or without one.
pub(super) fn cascade_uv(sea: Option<FftSea>, origin: Vec3) -> [[f32; 4]; 3] {
    let mut out = [[0.0; 4]; 3];
    let Some(sea) = sea else {
        return out;
    };
    for (row, &tile) in out.iter_mut().zip(&sea.tiles) {
        if tile > 0.0 {
            let at = |o: f32| (f64::from(o) / tile).rem_euclid(1.0) as f32;
            *row = [(1.0 / tile) as f32, at(origin.x), at(origin.z), tile as f32];
        }
    }
    out
}

/// The FFT spectrum of `water`'s sea `sea` in `cascades` cascades: the
/// ripples shorter than the shortest of its waves.
pub(super) fn sea_spectrum(
    water: &Water,
    sea: &og_env::ocean::Ocean,
    cascades: u32,
) -> crate::ocean_fft::spectrum::Spectrum {
    use crate::ocean_fft::spectrum;
    let params =
        spectrum::SeaParameters::from_ocean(water.wind, water.swell, water.choppiness, Some(sea));
    spectrum::build(&spectrum::cascades(cascades as usize, &params), &params)
}

/// The foam micro-structure's uv at the scene's `origin` (water_sea.wgsl
/// `sea_foam`, after web-ocean's `foamPattern`): per read, the origin's
/// world xz times its scale, whole repeats dropped (f64), plus web-ocean's
/// offsets and its lace's slow drift with the tick's `seconds`: the lace
/// (0.23 a metre), the finer lace (0.71), the bubbles (1.9), and the streaks
/// along and across the `wind` (0.035, 0.11).
pub(super) fn foam_uv(origin: Vec3, wind: Vec2, seconds: f64) -> [[f32; 4]; 2] {
    let (x, z) = (f64::from(origin.x), f64::from(origin.z));
    let d = wind.normalize_or_zero();
    let (dx, dz) = (f64::from(d.x), f64::from(d.y));
    let f = |v: f64| v.rem_euclid(1.0) as f32;
    [
        [
            f(x * 0.23 + 0.013 * seconds),
            f(z * 0.23),
            f(x * 0.71 + 0.37),
            f(z * 0.71 + 0.71),
        ],
        [
            f(x * 1.9 + 0.21),
            f(z * 1.9 + 0.53),
            f((x * dx + z * dz) * 0.035),
            f((-x * dz + z * dx) * 0.11),
        ],
    ]
}

/// The IEEE half nearest `v` (rounding half up), for the height fields'
/// rgba16float.
pub(super) fn f16_bits(v: f32) -> u16 {
    let b = v.to_bits();
    let sign = ((b >> 16) & 0x8000) as u16;
    let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
    let man = b & 0x7f_ffff;
    if v.is_nan() {
        0x7e00
    } else if exp >= 31 {
        sign | 0x7c00
    } else if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        let m = man | 0x80_0000;
        let shift = (14 - exp) as u32;
        let h = (m >> shift) + ((m >> (shift - 1)) & 1);
        sign | h as u16
    } else {
        let h = ((exp as u32) << 10 | (man >> 13)) + ((man >> 12) & 1);
        sign | h as u16
    }
}

/// The largest height field accepted.
pub(super) const MAX_RESOLUTION: u32 = 512;

/// The most cells a flat water's grid has per side (256² cells, 131,072
/// triangles per surface). It caps [`GRID_MAX_CELL`]'s split: a surface
/// longer than 65.5 km gets cells longer than 256 m again.
pub(super) const GRID_MAX_SEGMENTS: u32 = 256;
/// The longest a flat water's grid cell may be, in world metres. A
/// rasterizer interpolates a triangle's attributes and depth from its
/// corners in f32, so on a kilometres-long triangle that the near plane
/// clips both land metres off at the far side (og-tests' `terrain-far` lost
/// its shore foam); cells of at most 256 m draw the same thin band
/// everywhere.
pub(super) const GRID_MAX_CELL: f32 = 256.0;

/// Grid cells per side of a flat water (a sea is a clipmap: [`clipmap`]):
/// the surface's own `segments`, more where a cell of its longer side would
/// pass [`GRID_MAX_CELL`], at most [`GRID_MAX_SEGMENTS`].
pub(super) fn grid_segments(water: &Water) -> u32 {
    let scale = |axis: Vec4| axis.truncate().length();
    let extent = (water.size.x * scale(water.transform.x_axis))
        .max(water.size.y * scale(water.transform.z_axis));
    let wanted = if extent.is_finite() {
        (extent / GRID_MAX_CELL).ceil() as u32
    } else {
        1
    };
    water.segments.max(wanted).clamp(1, GRID_MAX_SEGMENTS)
}
