//! Material shader hooks (the E5 round): a [`MaterialShader`] is a WGSL
//! snippet that defines any of four fixed hooks of the lit shader, with
//! uniforms it declares. A material given the shader draws with a pipeline
//! built from the lit shader and the snippet; every other material keeps
//! the built-in hooks (identities, so its frames are unchanged).
//!
//! ```wgsl
//! // Every hook sees the fragment (or vertex) and the shader's parameters.
//! struct Hook { world: vec3<f32>, local: vec3<f32>, normal: vec3<f32>,
//!               uv: vec2<f32>, color: vec4<f32>, time: f32, eye: vec3<f32>,
//!               front: f32, origin: vec3<f32>, wind: vec4<f32> }
//! fn vertex_offset(h: Hook, p: Params) -> vec3<f32>              // world-space offset (0)
//! fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> // base colour and alpha (c)
//! fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32> // world normal (n)
//! fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> // emitted light (e)
//! ```
//!
//! `Params` holds the declared parameters by name (`p.amount`), at most
//! [`MAX_SHADER_PARAMS`], read from the material's uniform; `h.time` is the
//! scene clock in seconds; `h.eye` the camera's position (in the shadow
//! passes the origin: no view there); `h.origin` where the zero of `world`
//! and `eye` is in the world (the render origin, `origin.rs`: zero until
//! the camera is kilometres from the world's zero; add it for a pattern
//! laid in the world); `h.front` +1 on a triangle's front face and -1
//! on its back (three.js's `gl_FrontFacing`; the lit normal `n` is already turned
//! towards the eye, which differs from it for normals bent away from the face); `h.wind` og.Wind as the
//! frame draws it (`foliage::FoliageWind`'s first row, which every pass already has): xyz where it
//! blows (horizontal, normalised), w its `strength` (0 without og.Wind), the previous drawn frame's
//! where the prepass evaluates the previous position. An alpha below the material's `alphaTest`
//! discards the fragment (a dissolve). The hooks apply to static and
//! skinned meshes alike, and the shadow passes and the depth prepass see
//! the mesh where `vertex_offset` moved it and cut where `surface_color`'s
//! alpha falls below the alpha test (the shadow shader is built with the
//! same hooks, `shadow_source`). The prepass evaluates `vertex_offset` again
//! with the previous drawn frame's parameters, position and clock for motion
//! vectors; unchanged parameters settle on the next frame even when no tick
//! advances. `og-cook` compiles every shader asset with naga
//! (WGSL, SPIR-V and MSL) and refuses the asset
//! with naga's message when it does not.

/// The most parameters a shader declares.
pub const MAX_SHADER_PARAMS: usize = 8;

/// The hooks a snippet may define, in the order the module documentation
/// lists them.
pub const SHADER_HOOKS: [&str; 4] = [
    "vertex_offset",
    "surface_color",
    "surface_normal",
    "surface_emissive",
];

/// A parameter's WGSL type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParamKind {
    Float,
    Vec2,
    Vec3,
    /// A vec4; also a linear rgba colour.
    Vec4,
}

impl ParamKind {
    pub fn wgsl(self) -> &'static str {
        match self {
            ParamKind::Float => "f32",
            ParamKind::Vec2 => "vec2<f32>",
            ParamKind::Vec3 => "vec3<f32>",
            ParamKind::Vec4 => "vec4<f32>",
        }
    }

    /// How many numbers it takes.
    pub fn count(self) -> usize {
        match self {
            ParamKind::Float => 1,
            ParamKind::Vec2 => 2,
            ParamKind::Vec3 => 3,
            ParamKind::Vec4 => 4,
        }
    }

    fn swizzle(self) -> &'static str {
        match self {
            ParamKind::Float => ".x",
            ParamKind::Vec2 => ".xy",
            ParamKind::Vec3 => ".xyz",
            ParamKind::Vec4 => "",
        }
    }
}

/// A declared uniform: its name in `Params`, its type, its default.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderParam {
    pub name: String,
    pub kind: ParamKind,
    pub default: [f32; 4],
}

/// A material shader. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialShader {
    pub name: String,
    /// The WGSL that defines the hooks (and any helpers they call), or a
    /// full shader's `vertex` and `fragment` entries (`custom.rs`).
    pub code: String,
    pub params: Vec<ShaderParam>,
    /// Hooks of the built-in lit shader, or a full shader (`custom.rs`).
    pub mode: ShaderMode,
}

/// How a material shader's code is used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ShaderMode {
    /// The four hooks of the built-in lit shader (this module).
    #[default]
    Hooks,
    /// Full `vertex` and `fragment` entries over the binding contract of
    /// `custom.rs`; `lit`: the surface they return is lit by the frame's
    /// lights, else its colour is drawn as returned.
    Full { lit: bool },
    /// The hooks of the sprite shader only (`sprite_hooks.rs`): the code
    /// may also call `sprite_texel(uv)`, the sprite's texture at `uv`
    /// (distortion, chromatic splits). A mesh given one draws plain.
    Sprite,
}

impl MaterialShader {
    /// The parameter slots with their defaults.
    pub fn defaults(&self) -> [[f32; 4]; MAX_SHADER_PARAMS] {
        let mut out = [[0.0; 4]; MAX_SHADER_PARAMS];
        for (slot, p) in out.iter_mut().zip(&self.params) {
            *slot = p.default;
        }
        out
    }

    /// The index of parameter `name`.
    pub fn param(&self, name: &str) -> Option<(usize, ParamKind)> {
        self.params
            .iter()
            .position(|p| p.name == name)
            .map(|i| (i, self.params[i].kind))
    }

    /// Checks what can be checked without compiling: parameter names and
    /// count, and that the code defines at least one hook.
    pub fn check(&self) -> Result<(), String> {
        if self.params.len() > MAX_SHADER_PARAMS {
            return Err(format!(
                "{} parameters (at most {MAX_SHADER_PARAMS})",
                self.params.len()
            ));
        }
        for (i, p) in self.params.iter().enumerate() {
            let ok = p
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
                && p.name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !ok {
                return Err(format!(
                    "parameter '{}' is not a name (a letter, then letters, digits and _)",
                    p.name
                ));
            }
            if self.params[..i].iter().any(|q| q.name == p.name) {
                return Err(format!("parameter '{}' is declared twice", p.name));
            }
        }
        if let ShaderMode::Full { .. } = self.mode {
            if !["vertex", "fragment"]
                .iter()
                .any(|h| defines(&self.code, h))
            {
                return Err("a full shader defines neither `vertex` nor `fragment`".into());
            }
            return Ok(());
        }
        if !SHADER_HOOKS.iter().any(|h| defines(&self.code, h)) {
            return Err(format!(
                "the code defines none of the hooks ({})",
                SHADER_HOOKS.join(", ")
            ));
        }
        Ok(())
    }

    /// The hooks section of the lit shader for this snippet.
    pub fn hooks_wgsl(&self) -> String {
        hooks_section(&self.params, &self.code)
    }
}

/// Whether `code` defines function `name`.
pub fn defines(code: &str, name: &str) -> bool {
    code.match_indices(name).any(|(i, _)| {
        let before = code[..i].trim_end();
        let after = code[i + name.len()..].trim_start();
        before.ends_with("fn")
            && before[..before.len() - 2]
                .chars()
                .last()
                .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
            && after.starts_with('(')
    })
}

const HOOK_STRUCT: &str = "struct Hook {
  world: vec3<f32>,
  local: vec3<f32>,
  normal: vec3<f32>,
  uv: vec2<f32>,
  color: vec4<f32>,
  time: f32,
  eye: vec3<f32>,
  front: f32,
  origin: vec3<f32>,
  wind: vec4<f32>,
};
// The fragment's facing (+1 front, -1 back), set by the fragment entry points
// that know it; 1 elsewhere (vertex stages, shadow passes).
var<private> hook_front: f32 = 1.0;
// An effect material's inputs (effect.rs), set before its hook runs: the
// surface's life (0 to 1) and random value (0 to 1), and its custom data.
// 0 in a lit material and a sprite.
var<private> hook_effect: vec2<f32>;
var<private> hook_custom1: vec4<f32>;
var<private> hook_custom2: vec4<f32>;
fn effect_life() -> f32 { return hook_effect.x; }
fn effect_random() -> f32 { return hook_effect.y; }
fn effect_custom1() -> vec4<f32> { return hook_custom1; }
fn effect_custom2() -> vec4<f32> { return hook_custom2; }
";

const DEFAULT_HOOKS: [&str; 4] = [
    "fn vertex_offset(h: Hook, p: Params) -> vec3<f32> { return vec3<f32>(0.0); }\n",
    "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }\n",
    "fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32> { return n; }\n",
    "fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> { return e; }\n",
];

/// The `Hook` and `Params` structs, `shader_params()`, the snippet, the
/// built-in hooks it leaves out and the lit vertex stage's `moved_clip`.
/// Placed after material.wgsl (it reads `material.custom`, and the previous
/// parameters only in the frame `material.stamp` names: `params_moved`,
/// frame.wgsl, false in the casters' shadow_pass.wgsl; engine s100).
pub fn hooks_section(params: &[ShaderParam], code: &str) -> String {
    let mut s = hook_functions_from(
        params,
        code,
        "material.custom",
        Some(("material.previous_custom", "params_moved(material.stamp.x)")),
    );
    s.push_str(if defines(code, "vertex_offset") {
        MOVED_CLIP_OFFSET
    } else {
        MOVED_CLIP
    });
    s
}

/// The hook functions of a module that declares a `material` uniform with
/// `custom` and draws no motion (the sprite module, `sprite_hooks.rs`):
/// [`hooks_section`]'s without `moved_clip`, its previous parameters the
/// current ones.
pub fn hook_functions(params: &[ShaderParam], code: &str) -> String {
    hook_functions_from(params, code, "material.custom", None)
}

/// The one hook contract every kind of draw takes (lit materials, sprites,
/// effect materials): the `Hook` and `Params` structs, the effect inputs,
/// `shader_params()` reading `Params` from the array `current`,
/// `previous_shader_params()` from the array of `previous` where its
/// condition holds (the frame they changed in) and from `current` elsewhere
/// (without `previous`, always: a draw without motion), the snippet and
/// the built-in hooks it leaves out. An effect draw reads its entry's
/// parameters (`effect.rs`).
pub fn hook_functions_from(
    params: &[ShaderParam],
    code: &str,
    current: &str,
    previous: Option<(&str, &str)>,
) -> String {
    let mut s = String::from("// Material shader hooks (hooks.rs).\n");
    s.push_str(HOOK_STRUCT);
    s.push_str("struct Params {\n");
    if params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\n");
    s.push_str("fn shader_params() -> Params {\n  var p: Params;\n");
    for (i, p) in params.iter().enumerate() {
        s.push_str(&format!(
            "  p.{} = {current}[{i}]{};\n",
            p.name,
            p.kind.swizzle()
        ));
    }
    s.push_str("  return p;\n}\n");
    match previous {
        Some((field, moved)) if !params.is_empty() => {
            s.push_str(&format!(
                "fn previous_shader_params() -> Params {{\n  var p: Params;\n  let moved = {moved};\n"
            ));
            for (i, p) in params.iter().enumerate() {
                s.push_str(&format!(
                    "  p.{} = select({current}[{i}], {field}[{i}], moved){};\n",
                    p.name,
                    p.kind.swizzle()
                ));
            }
            s.push_str("  return p;\n}\n");
        }
        _ => s.push_str("fn previous_shader_params() -> Params {\n  return shader_params();\n}\n"),
    }
    s.push_str(code);
    s.push('\n');
    for (name, default) in SHADER_HOOKS.iter().zip(DEFAULT_HOOKS) {
        if !defines(code, name) {
            s.push_str(default);
        }
    }
    s
}

/// `moved_clip`: how the lit vertex stage projects the point a material's
/// `vertex_offset` moved (pbr.wgsl `lit_place`). Without an offset the
/// point is the instance matrix's, projected with `mul_point`
/// (instance.wgsl); moved by an offset, its xyz are later values than its
/// w, and the plain product orders the sum as before the instance matrix
/// had a literal fourth row. Either way a lit vertex lands where the full
/// 4x4 put it, bit for bit.
const MOVED_CLIP: &str =
    "fn moved_clip(m: mat4x4<f32>, p: vec4<f32>) -> vec4<f32> { return mul_point(m, p); }\n";
const MOVED_CLIP_OFFSET: &str =
    "fn moved_clip(m: mat4x4<f32>, p: vec4<f32>) -> vec4<f32> { return m * p; }\n";

/// The built-in hooks (identities).
pub fn default_hooks() -> String {
    hooks_section(&[], "")
}

/// The sprite shader module's source with `shader`'s hooks, as the
/// renderer compiles it for sprites drawn with it (`sprite_hooks.rs`). The
/// cook validates a `mode sprite` shader with this.
pub fn sprite_module_source(shader: &MaterialShader) -> String {
    crate::sprite_hooks::source(shader)
}

/// The lit shader module's source with `hooks`, as the renderer compiles
/// it. The cook validates shader assets with this.
pub fn lit_module_source(shader: &MaterialShader) -> String {
    if let ShaderMode::Full { lit } = shader.mode {
        return crate::custom::lit_source(shader, lit);
    }
    crate::lit_source(&shader.hooks_wgsl())
}

/// [`lit_module_source`] for the physical lit shader (`physical.rs`), which
/// a physical material with a material shader draws with.
pub fn physical_module_source(shader: &MaterialShader) -> String {
    // A full shader draws a physical material with its own entries.
    if let ShaderMode::Full { .. } = shader.mode {
        return lit_module_source(shader);
    }
    crate::physical_source(&shader.hooks_wgsl())
}

/// The shadow casters' module with `shader`'s hooks (vertex_offset and
/// surface_color cut the shadow as they cut the lit pass): what every
/// cascade, spot, cube-face and paraboloid pass draws casters with.
pub fn shadow_module_source(shader: &MaterialShader) -> String {
    if let ShaderMode::Full { .. } = shader.mode {
        return crate::custom::shadow_source(shader);
    }
    crate::shadow_source(&shader.hooks_wgsl())
}
