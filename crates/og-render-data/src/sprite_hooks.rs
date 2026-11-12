//! Sprites drawn with a material shader ([`crate::SpriteShading`]): a
//! sprite given a material whose shader is of [`ShaderMode::Hooks`] or
//! [`ShaderMode::Sprite`] draws through pipelines built from the sprite
//! module with the shader's hooks (`sprite_hooks.wgsl`), the material's
//! parameters in a uniform of its own at group 3. The same shader asset,
//! material and parameters as a mesh's (og.load.shader,
//! og.render.shaderMaterial, og.render.setShaderParam): a dissolve written
//! for meshes dissolves a sprite. Sprites of one material, texture and
//! blend batch into one draw. Made on the first frame that draws one, so
//! a renderer that never does pays nothing, and every other sprite draws
//! through the sprite pipelines as before.

use crate::hooks::MaterialShader;

/// The module's source as the renderer compiles it for `shader`.
pub fn source(shader: &MaterialShader) -> String {
    format!(
        "{}{}{}{}",
        crate::sprite_source(),
        crate::particles_gpu::NOISE_WGSL,
        crate::hooks::hook_functions(&shader.params, &shader.code),
        wgsl!("sprite_hooks.wgsl")
    )
}

/// A sprite shader using every hook it runs, two parameters and
/// `sprite_texel`: the module the gate validates (`shaders.rs`).
pub fn example() -> MaterialShader {
    use crate::hooks::{ParamKind, ShaderMode, ShaderParam};
    MaterialShader {
        name: "sprite example".into(),
        code: "fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  return vec3<f32>(0.0, sin(h.time + h.local.x * 6.0) * p.amount * 0.05, 0.0);
}
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let shifted = sprite_texel(h.uv + vec2<f32>(0.01 * p.amount, 0.0));
  return vec4<f32>(mix(c.rgb, shifted.rgb * p.tint.rgb, p.amount), c.a);
}
fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> {
  return e + p.tint.rgb * p.amount * h.color.a;
}
"
        .into(),
        params: vec![
            ShaderParam {
                name: "amount".into(),
                kind: ParamKind::Float,
                default: [0.5, 0.0, 0.0, 0.0],
            },
            ShaderParam {
                name: "tint".into(),
                kind: ParamKind::Vec4,
                default: [0.4, 0.8, 1.0, 1.0],
            },
        ],
        mode: ShaderMode::Sprite,
    }
}
