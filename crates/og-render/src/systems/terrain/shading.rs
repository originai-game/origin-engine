use super::*;

/// Hexagons per texture repeat along the lattice (terrain_lit.wgsl
/// `HEX_SCALE`, 2√3 as Mikkelsen's demo).
pub(super) const HEX_SCALE: f64 = 3.464_101_615_137_754_6;

/// Where the hex-tiling lattice of a layer of `tile` metres stands at the
/// scene's `origin`, per projection plane (xz, zy, xy): the origin's place
/// in its skewed cell and that cell's centre in uv (whole repeats
/// dropped), and the cell's ids. Computed in f64 from the world's zero,
/// so a hexagon covers the same ground wherever the origin is: the
/// shader adds the point's uv from the origin to these.
pub(super) fn hex_lattice(origin: Vec3, tile: f32) -> ([[f32; 4]; 3], [[i32; 4]; 2]) {
    let k = f64::from(1.0 / tile.max(1e-3));
    let o = [
        f64::from(origin.x),
        f64::from(origin.y),
        f64::from(origin.z),
    ];
    let mut hex = [[0.0; 4]; 3];
    let mut ids = [[0i32; 2]; 3];
    for (plane, (a, b)) in [(0, 2), (2, 1), (0, 1)].into_iter().enumerate() {
        let st = [o[a] * k * HEX_SCALE, o[b] * k * HEX_SCALE];
        let skewed = [st[0] - st[1] / 3f64.sqrt(), st[1] * 2.0 / 3f64.sqrt()];
        let base = skewed.map(f64::floor);
        let centre = [
            ((base[0] + 0.5 * base[1]) / HEX_SCALE).rem_euclid(1.0),
            (base[1] * 3f64.sqrt() * 0.5 / HEX_SCALE).rem_euclid(1.0),
        ];
        hex[plane] = [
            (skewed[0] - base[0]) as f32,
            (skewed[1] - base[1]) as f32,
            centre[0] as f32,
            centre[1] as f32,
        ];
        // Wrapping: the shader adds its cells in i32 the same way.
        ids[plane] = base.map(|b| b.rem_euclid(4_294_967_296.0) as u32 as i32);
    }
    (
        hex,
        [
            [ids[0][0], ids[0][1], ids[1][0], ids[1][1]],
            [ids[2][0], ids[2][1], 0, 0],
        ],
    )
}

/// A layer's anti-tiling, blend and macro fields at the scene's origin.
pub(super) fn layer_look(l: &TerrainLayer, origin: Vec3) -> LayerGpu {
    let (hex, hex_base) = hex_lattice(origin, l.tile);
    LayerGpu {
        look: [
            l.anti_tiling.clamp(0.0, 1.0),
            l.height_contrast.clamp(0.0, 1.0),
            l.project_slope.clamp(0.0, 90.0),
            l.macro_variation.clamp(0.0, 1.0),
        ],
        hex,
        hex_base,
        coarse: {
            let mut c = uv_offset(origin, l.tile * 4.0);
            c[3] = l.cell_variation.clamp(0.0, 1.0);
            c
        },
        ..LayerGpu::default()
    }
}

/// How far lying snow lifts the drawn ground, in world metres
/// (terrain_vertex.wgsl terrain_snow_lift): the weather's lying depth
/// (`SurfaceWeather::snow_depth`) once its cover has built up, at most
/// 10 cm. Only the drawn ground moves (collision, `og.terrain.sample`,
/// picking and what stands on the ground keep the field), so it stays
/// small enough that nothing visibly floats or sinks.
pub(crate) fn snow_lift(scene: &crate::RenderScene) -> f32 {
    scene.atmosphere.surface_weather.as_ref().map_or(0.0, |w| {
        let t = (w.snow / 0.3).clamp(0.0, 1.0);
        w.snow_depth.clamp(0.0, 0.1) * t * t * (3.0 - 2.0 * t)
    })
}

/// The layers' sampling (after [`TERRAIN_WGSL`]): the lit pass's shading
/// and the displacement's heights.
const TERRAIN_LAYERS_WGSL: &str = wgsl3d!("terrain_layers.wgsl");
/// The clipmap's mesh with its displacement (after [`TERRAIN_LAYERS_WGSL`]):
/// the lit and shadow modules' `terrain_local_vertex`.
const TERRAIN_VERTEX_WGSL: &str = wgsl3d!("terrain_vertex.wgsl");
/// What the ground takes of the shared shading: the weather, and the
/// decals over its own bindings (terrain_lit.wgsl declares them; it has no
/// effect group, so no screen AO).
const SHADING: crate::Shading = crate::Shading::lit().weather().decals();

/// The terrain's lit module: the frame, the shared shading, then the
/// ground.
pub(crate) fn terrain_lit_source() -> String {
    crate::assemble(
        &[
            crate::FRAME_WGSL,
            &SHADING.text(),
            crate::wind::WIND_COMMON_WGSL,
            crate::trample::READ_WGSL,
            GRASS_COMMON_WGSL,
            TERRAIN_WGSL,
            TERRAIN_LAYERS_WGSL,
            TERRAIN_VERTEX_WGSL,
            wgsl3d!("terrain_lit.wgsl"),
        ],
        &crate::Capability::OPAQUE_SURFACE,
    )
}

/// A terrain module (`standard`: [`terrain_lit_source`] or
/// [`terrain_shadow_source`]) as its layer-array variant: the arrays
/// declared, and `layer_albedo` and `layer_normal_map` reading layer
/// `i`'s array layer (slope.w and altitude.w) instead of its texture. The
/// rest is the same text, so the shading and the displacement are the
/// standard path's.
pub(crate) fn terrain_array_source(standard: &str) -> String {
    let s = terrain_array_declarations(standard);
    let start = s
        .find("fn layer_albedo(")
        .expect("terrain_layers.wgsl's layer_albedo");
    let end = s[start..]
        .find("fn band(")
        .map(|e| start + e)
        .expect("terrain_layers.wgsl's layer_normal_map ends before band");
    format!(
        "{}{}{}",
        &s[..start],
        "fn layer_albedo(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  return textureSampleGrad(albedo_array, layer_sampler, uv, i32(terrain.layers[i].slope.w), dx, dy);
}

fn layer_dimensions(i: u32) -> vec2<f32> {
  return vec2<f32>(textureDimensions(albedo_array));
}

fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  let t = textureSampleGrad(normal_array, layer_sampler, uv, i32(terrain.layers[i].altitude.w), dx, dy);
  return vec4<f32>(t.xyz * 2.0 - 1.0, t.w);
}

",
        &s[end..]
    )
}

/// The terrain's shadow module for the layer-array variant's layout (the
/// same entry points; the layer arrays declared).
pub(crate) fn terrain_shadow_array_source() -> String {
    terrain_array_source(&terrain_shadow_source())
}

/// The corner cache's writer module (its layer-array variant:
/// [`terrain_array_source`] of it).
pub(crate) fn terrain_corners_source() -> String {
    format!(
        "{}{}{GRASS_COMMON_WGSL}{TERRAIN_WGSL}{TERRAIN_LAYERS_WGSL}{TERRAIN_VERTEX_WGSL}{}",
        crate::LIBRARY_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        wgsl3d!("terrain_corners.wgsl")
    )
}

/// The terrain's shadow module.
pub(crate) fn terrain_shadow_source() -> String {
    format!(
        "{}{}{GRASS_COMMON_WGSL}{TERRAIN_WGSL}{TERRAIN_LAYERS_WGSL}{TERRAIN_VERTEX_WGSL}{}{}",
        crate::LIBRARY_WGSL,
        crate::wind::WIND_COMMON_WGSL,
        crate::SHADOW_PASS_WGSL,
        wgsl3d!("terrain_shadow.wgsl")
    )
}
