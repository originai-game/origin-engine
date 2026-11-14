//! The grass's uniforms: what its generate and draw shaders read of a field.

use super::*;
use crate::ground::FarGpu;

/// A field's canopy as the shaders hold it (grass.wgsl `grass_canopy`,
/// grass_common.wgsl `grass_mean`): its extinction per metre of a ray
/// (blades per square metre times their width times [`CANOPY_BLOCKING`])
/// and its top, the field's mean blade height.
pub(super) fn canopy(g: &Grass) -> [f32; 2] {
    [
        g.density.max(0.0) * g.width.max(0.0) * CANOPY_BLOCKING,
        g.height * (1.0 - 0.5 * g.height_variation.clamp(0.0, 1.0)),
    ]
}

/// The seed as the shaders read it: 24 bits, exact in an f32.
pub(super) fn seed_bits(seed: u32) -> f32 {
    (hash(seed ^ 0x9e37_79b9) >> 8) as f32
}

/// The far grass terrain `terrain` of `scene` takes: its first field
/// with `far_tint` (none: colour w -1).
pub(crate) fn far_gpu(scene: &crate::RenderScene, terrain: usize) -> FarGpu {
    let mut gpu = FarGpu {
        color: [0.0, 0.0, 0.0, -1.0],
        fade: [0.0; 4],
        clump: [1.0, 0.0, 0.0, 0.0],
        dry: [0.0; 4],
        root: [0.0; 4],
        canopy: [0.0; 4],
        wind: WindFieldGpu::default(),
    };
    let Some(d) = scene
        .world
        .grass
        .iter()
        .find(|d| d.terrain == terrain && d.grass.far_tint && d.grass.usable())
    else {
        return gpu;
    };
    let g = &d.grass;
    let along = scene
        .world
        .terrains
        .get(terrain)
        .map_or(1.0, |t| horizontal_scale(t.transform));
    // The field's own colours and canopy: the ground under and past the
    // blades takes what they show on average from where it is seen
    // (grass_common.wgsl grass_mean), so it reads as the same field.
    gpu.color = g.tip_color.extend(g.layer as f32).to_array();
    // w: the patches' size (m), as the blades' (look_gpu).
    gpu.root = g.base_color.extend(g.clump_size.max(0.05) * 6.0).to_array();
    let [extinction, top] = canopy(g);
    // z: the blades' spacing (m), the grain of the field from afar.
    gpu.canopy = [
        extinction,
        top,
        g.density.max(1e-3).sqrt().recip(),
        g.translucency.clamp(0.0, 1.0),
    ];
    // Under every blade (gaps show the field's colour, not the layer's),
    // whole by the start of LOD 1; gusts brighten it as they do the blades.
    let lod = DistanceLod::for_radius(g.distance);
    gpu.fade = [0.0, lod.lod_start[0], GUST_SHEEN, 1.0];
    gpu.clump = [
        g.clump_size.max(0.05) / along,
        g.color_variation.clamp(0.0, 1.0),
        g.dryness.clamp(0.0, 1.0),
        seed_bits(g.seed),
    ];
    gpu.dry = g.dry_color.extend(0.0).to_array();
    // The far tint brightens in og.Wind's analytic gusts (the terrain
    // does not read the simulated field).
    gpu.wind = crate::wind::field_gpu(
        scene.atmosphere.wind,
        scene.frame.tick,
        scene.frame.tick,
        scene.frame.tick_rate,
        scene.frame.origin,
        crate::wind_field::NO_GRID,
    );
    gpu
}

/// `GrassGen` in grass_generate.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct GenGpu {
    pub(super) planes: [[f32; 4]; 6],
    pub(super) eye: [f32; 4],
    pub(super) origin: [f32; 4],
    pub(super) lod: [f32; 4],
    pub(super) lod_density: [f32; 4],
    pub(super) field: [f32; 4],
    pub(super) chunk: [f32; 4],
    pub(super) shape: [f32; 4],
    pub(super) clump: [f32; 4],
    pub(super) shadow: [f32; 4],
    pub(super) motion: [f32; 4],
    pub(super) counts: [u32; 4],
    pub(super) trample: [f32; 4],
    pub(super) kinds: [[f32; 4]; 4],
    pub(super) maps: [f32; 4],
    pub(super) wind: WindFieldGpu,
}

/// `GrassUniform` in grass_blade.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct DrawGpu {
    pub(super) view_proj: [[f32; 4]; 4],
    pub(super) prev_view_proj: [[f32; 4]; 4],
    pub(super) eye: [f32; 4],
    pub(super) forward: [f32; 4],
    pub(super) origin: [f32; 4],
    pub(super) limits: [f32; 4],
    pub(super) sun: [f32; 4],
    pub(super) shape: [f32; 4],
    pub(super) base_color: [f32; 4],
    pub(super) tip_color: [f32; 4],
    pub(super) dry_color: [f32; 4],
    pub(super) variation: [f32; 4],
    pub(super) rows: [u32; 4],
    pub(super) canopy: [f32; 4],
    pub(super) kinds: [[f32; 4]; 8],
    pub(super) wind: WindFieldGpu,
}

/// A field's kinds as `GrassGen.kinds` holds them: height, width, where
/// its share ends (summed in order over the kinds the field has) and
/// whether it has it.
pub(super) fn kinds_gen(g: &Grass) -> [[f32; 4]; 4] {
    let kinds = g.kinds();
    let total: f32 = kinds.iter().flatten().map(|k| k.2).sum();
    let mut run = 0.0;
    std::array::from_fn(|i| match kinds[i] {
        Some((h, w, share)) => {
            run += share;
            let end = if total > 0.0 { run / total } else { 1.0 };
            [h, w, if i == 3 { 1.0 } else { end }, 1.0]
        }
        None => [0.0, 0.0, if total > 0.0 { run / total } else { 1.0 }, 0.0],
    })
}

/// A field's kinds as `GrassUniform.kinds` holds them: roots' colour and
/// curve, then tips' colour, the field's own first.
pub(super) fn kinds_draw(g: &Grass) -> [[f32; 4]; 8] {
    let own = GrassType {
        height: g.height,
        width: g.width,
        curve: g.curve,
        base_color: g.base_color,
        tip_color: g.tip_color,
        share: 1.0,
    };
    let mut out = [[0.0; 4]; 8];
    for (k, t) in std::iter::once(Some(own)).chain(g.types).enumerate() {
        let t = t.unwrap_or(own);
        out[2 * k] = t.base_color.extend(t.curve.clamp(0.0, 1.0)).to_array();
        out[2 * k + 1] = t.tip_color.extend(0.0).to_array();
    }
    out
}

/// A field's look as `GrassUniform` holds it: `base_color`, `tip_color`,
/// `dry_color` and `variation`.
pub(super) fn look_gpu(g: &Grass) -> [[f32; 4]; 4] {
    [
        g.base_color
            .extend(g.translucency.clamp(0.0, 1.0))
            .to_array(),
        g.tip_color.extend(g.sheen.clamp(0.0, 1.0)).to_array(),
        g.dry_color.extend(g.roughness.clamp(0.04, 1.0)).to_array(),
        [
            g.color_variation.clamp(0.0, 1.0),
            g.dryness.clamp(0.0, 1.0),
            g.clump_size.max(0.05) * 6.0,
            g.distance,
        ],
    ]
}

/// How far lying snow weighs the blades down, 0..1 (`GrassUniform.limits.w`,
/// grass_blade.wgsl): its cover, as weather_surface.wgsl lays it. A
/// uniform for the whole field, so the lit, prepass and shadow passes and
/// the motion vectors bend every blade alike (a roof's shelter is not in
/// the shadow pass).
pub(super) fn snow_weight(scene: &crate::RenderScene) -> f32 {
    let snow = scene
        .atmosphere
        .surface_weather
        .as_ref()
        .map_or(0.0, |w| w.snow.clamp(0.0, 1.0));
    let t = (snow / 0.6).min(1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The height range blade roots are stored over, either side of the camera (m).
pub(super) const HEIGHT_RANGE: f32 = 512.0;
