//! Full custom shaders (the S8 depth stream, `docs/depth.md` §8): a
//! [`MaterialShader`] of [`ShaderMode::Full`] replaces the lit shader's
//! vertex and fragment stages with its own `vertex` and `fragment` entries
//! over a fixed binding contract (`shaders/custom_common.wgsl`):
//!
//! ```wgsl
//! fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput
//! fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput
//! ```
//!
//! - camera: `camera()` (the pass's view projection, position, forward
//!   axis and clock; a shadow pass's is the light's);
//! - model: the instance's transform, a skinned mesh's joints folded in and
//!   its morph targets already applied to `v`;
//! - params: `p`, the parameters the asset declares (at most
//!   [`crate::MAX_SHADER_PARAMS`]), set with `og.render.setShaderParam`;
//! - textures: `tex0(uv)` .. `tex7(uv)`, the material's eight texture slots
//!   (map, metalnessMap, roughnessMap, normalMap, emissiveMap, aoMap,
//!   gradientMap, dissolveMap), so any texture, a render target's colour
//!   included, reaches the shader through the material that draws it;
//! - helpers: `standard_vertex`, `standard_surface`, `noise3`.
//!
//! An entry left out takes the built-in one (`standard_vertex`,
//! `standard_surface`). `lit` shaders return a surface (albedo and alpha,
//! normal, metalness, roughness, emission) lit by the sun with its
//! cascades, the clustered point and spot lights with their shadows, the
//! image-based light and fog; `unlit` ones return the colour drawn. The
//! engine projects `VertexOutput.world` with each pass's camera, so the
//! depth prepass, every shadow pass, the water's reflection and every view
//! draw the mesh through the same vertex entry, and a surface alpha below
//! the material's `alphaTest` is cut in each of them (a dissolve's shadow
//! follows its cut).
//!
//! The modules keep the built-in shaders' entry names and vertex buffers,
//! so a full shader draws through the hooked materials' pipelines; the
//! cook validates each tier's modules with naga (WGSL, SPIR-V, MSL and
//! GLSL ES 3.0) as it does the hooks'.

use crate::hooks::{MaterialShader, ShaderMode, defines, hooks_section};
use crate::{clusters::LightTables, skin::PaletteMode};

/// The entries a full shader may define (both optional, at least one).
pub const FULL_SHADER_ENTRIES: [&str; 2] = ["vertex", "fragment"];

const COMMON_WGSL: &str = wgsl!("custom_common.wgsl");
const LIT_WGSL: &str = wgsl!("custom_lit.wgsl");
const SHADOW_WGSL: &str = wgsl!("custom_shadow.wgsl");

const DEFAULT_VERTEX: &str = "fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput { return standard_vertex(v, model); }\n";
const DEFAULT_FRAGMENT: &str =
    "fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput { return standard_surface(f); }\n";

/// The palette and light tables of a tier (`full`: storage buffers).
pub(crate) fn tier(full: bool) -> (PaletteMode, LightTables) {
    if full {
        (PaletteMode::Storage, LightTables::Storage)
    } else {
        (PaletteMode::Texture, LightTables::Uniform)
    }
}

/// The contract, the shader's parameters and code, and the defaults of the
/// entries it leaves out.
fn user_section(shader: &MaterialShader) -> String {
    // The hooks section declares `Params` and `shader_params()` (its
    // built-in hooks are unused here).
    let mut s = format!("{COMMON_WGSL}{}", hooks_section(&shader.params, ""));
    s.push_str("// The shader's code (custom.rs).\n");
    s.push_str(&shader.code);
    s.push('\n');
    if !defines(&shader.code, "vertex") {
        s.push_str(DEFAULT_VERTEX);
    }
    if !defines(&shader.code, "fragment") {
        s.push_str(DEFAULT_FRAGMENT);
    }
    s
}

/// The lit (and prepass) module of a full shader.
pub(crate) fn lit_source(
    shader: &MaterialShader,
    lit: bool,
    palette: PaletteMode,
    tables: LightTables,
) -> String {
    format!(
        "const CUSTOM_LIT: bool = {lit};\n{}{}{}{}{}{}{}{}{}{}{}{LIT_WGSL}",
        crate::FRAME_WGSL,
        tables.wgsl(),
        crate::particles_gpu::NOISE_WGSL,
        crate::MATERIAL_WGSL,
        crate::MATERIAL_MAPS_WGSL,
        crate::MATERIAL_DISSOLVE_WGSL,
        palette.wgsl(3),
        crate::LIGHTING_WGSL,
        crate::INSTANCE_WGSL,
        fx_section(),
        user_section(shader),
    )
}

// The lit module's fx group (pbr.wgsl's declarations, which the prepass
// reads), kept apart from pbr.wgsl's entry points.
fn fx_section() -> &'static str {
    "struct DecalGpu { inv: mat4x4<f32>, axis: vec4<f32>, uv_rect: vec4<f32>, color: vec4<f32>, emissive: vec4<f32> };
struct Fx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,
  ao: vec4<f32>,
  decal_info: vec4<f32>,
  reserved: vec4<f32>,
  decals: array<DecalGpu, 32>,
};
@group(2) @binding(0) var<uniform> fx: Fx;
"
}

/// The shadow casters' module of a full shader.
pub(crate) fn shadow_source(shader: &MaterialShader, palette: PaletteMode) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{SHADOW_WGSL}",
        crate::particles_gpu::NOISE_WGSL,
        crate::MATERIAL_WGSL,
        crate::MATERIAL_MAPS_WGSL,
        crate::MATERIAL_DISSOLVE_WGSL,
        palette.wgsl(2),
        crate::SHADOW_PASS_WGSL,
        crate::INSTANCE_WGSL,
        user_section(shader),
    )
}

/// The lit and shadow module sources of any material shader, as the
/// renderer compiles them.
pub(crate) fn module_sources(
    shader: &MaterialShader,
    palette: PaletteMode,
    tables: LightTables,
) -> (String, String) {
    match shader.mode {
        ShaderMode::Hooks => {
            let hooks = shader.hooks_wgsl();
            (
                crate::lit_source_with(&hooks, palette, tables),
                crate::shadow_source(&hooks, palette),
            )
        }
        ShaderMode::Full { lit } => (
            lit_source(shader, lit, palette, tables),
            shadow_source(shader, palette),
        ),
    }
}

/// A dissolve as a full shader: the material's surface, cut where value
/// noise over the object falls below `amount`, a glowing `glow` edge
/// `edge` wide above the cut (validated with the engine's modules,
/// `shaders.rs`, and drawn by og-tests' `shader-full-dissolve`).
pub fn example() -> MaterialShader {
    use crate::{ParamKind, ShaderParam};
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    MaterialShader {
        name: "dissolve-full".into(),
        code: EXAMPLE_WGSL.into(),
        params: vec![
            param("amount", ParamKind::Float, [0.4, 0.0, 0.0, 0.0]),
            param("edge", ParamKind::Float, [0.06, 0.0, 0.0, 0.0]),
            param("scale", ParamKind::Float, [3.0, 0.0, 0.0, 0.0]),
            param("glow", ParamKind::Vec4, [4.0, 1.2, 0.2, 1.0]),
        ],
        mode: ShaderMode::Full { lit: true },
    }
}

const EXAMPLE_WGSL: &str =
    "fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput {
  // A slow sway, so the shadow passes follow the vertex entry too.
  var o = standard_vertex(v, model);
  o.world.x += 0.08 * sin(camera().time * 2.0 + v.position.y * 3.0);
  return o;
}

fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput {
  var s = standard_surface(f);
  let n = noise3(f.local * p.scale);
  let cut = n < p.amount;
  let edge = 1.0 - clamp((n - p.amount) / max(p.edge, 1e-4), 0.0, 1.0);
  s.color.a = select(1.0, 0.0, cut);
  s.emissive += p.glow.rgb * edge * edge;
  return s;
}
";
