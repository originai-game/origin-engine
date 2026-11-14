//! Full custom shaders (the S8 depth stream, `docs/design.md` §10.8): a
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
//! cascades, the clustered point, spot and rect lights with their
//! shadows (a rect light through linearly transformed cosines and its
//! soft shadow, area.wgsl's `surface_local_light`, as terrain and the
//! scattered copies are lit), the image-based light and fog; `unlit` ones return the colour drawn. The
//! engine projects `VertexOutput.world` with each pass's camera, so the
//! depth prepass, every shadow pass, the water's reflection and every view
//! draw the mesh through the same vertex entry, and a surface alpha below
//! the material's `alphaTest` is cut in each of them (a dissolve's shadow
//! follows its cut).
//!
//! The modules keep the built-in shaders' entry names and vertex buffers,
//! so a full shader draws through the hooked materials' pipelines; the
//! cook validates its modules with naga (WGSL, SPIR-V and MSL) as it does
//! the hooks'.

use crate::hooks::{MaterialShader, ShaderMode, defines, hooks_section};

/// The entries a full shader may define (both optional, at least one).
pub const FULL_SHADER_ENTRIES: [&str; 2] = ["vertex", "fragment"];

const COMMON_WGSL: &str = wgsl!("custom_common.wgsl");
const LIT_WGSL: &str = wgsl!("custom_lit.wgsl");
const SHADOW_WGSL: &str = wgsl!("custom_shadow.wgsl");

const DEFAULT_VERTEX: &str = "fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput { return standard_vertex(v, model); }\n";
const DEFAULT_FRAGMENT: &str =
    "fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput { return standard_surface(f); }\n";

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

/// The lit (and prepass) module of a full shader: the built-in lit
/// shader's groups, with the effect group, the decals, the weather, the
/// screen AO and the shared shading its lit fragment hands its surface to
/// (custom_lit.wgsl).
pub fn lit_source(shader: &MaterialShader, lit: bool) -> String {
    crate::assemble(
        &[
            &format!("const CUSTOM_LIT: bool = {lit};\n"),
            crate::FRAME_WGSL,
            crate::oit::OIT_WGSL,
            crate::oit::ARENA_WGSL,
            crate::particles_gpu::NOISE_WGSL,
            crate::MATERIAL_WGSL,
            crate::MATERIAL_MAPS_WGSL,
            crate::MATERIAL_DISSOLVE_WGSL,
            &crate::skin::palette_wgsl(3),
            &crate::SURFACE.text(),
            crate::INSTANCE_WGSL,
            &user_section(shader),
            LIT_WGSL,
        ],
        &[],
    )
}

/// The shadow casters' module of a full shader.
pub fn shadow_source(shader: &MaterialShader) -> String {
    // `screen_ao` is the lit module's; here a shader's call of it reads
    // no occlusion.
    crate::assemble(
        &[
            crate::LIBRARY_WGSL,
            crate::oit::OIT_WGSL,
            crate::particles_gpu::NOISE_WGSL,
            crate::MATERIAL_WGSL,
            crate::MATERIAL_MAPS_WGSL,
            crate::MATERIAL_DISSOLVE_WGSL,
            &crate::skin::palette_wgsl(2),
            crate::SHADOW_PASS_WGSL,
            crate::INSTANCE_WGSL,
            &user_section(shader),
            SHADOW_WGSL,
        ],
        &[crate::Capability::ScreenAo],
    )
}

/// The lit and shadow module sources of any material shader, as the
/// renderer compiles them (`planar`: its effect group carries the planar
/// reflection, `crate::planar_slot`).
pub fn module_sources(shader: &MaterialShader, planar: bool) -> (String, String) {
    match shader.mode {
        ShaderMode::Hooks => {
            let hooks = shader.hooks_wgsl();
            (
                crate::lit_source_with(&hooks, planar),
                crate::shadow_source(&hooks),
            )
        }
        ShaderMode::Full { lit } => (lit_source(shader, lit), shadow_source(shader)),
        // Only sprites run a sprite shader; a mesh draws plain.
        ShaderMode::Sprite => {
            let hooks = crate::hooks::default_hooks();
            (
                crate::lit_source_with(&hooks, planar),
                crate::shadow_source(&hooks),
            )
        }
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
