//! Material shader hooks (the E5 round): a [`MaterialShader`] is a WGSL
//! snippet that defines any of four fixed hooks of the lit shader, with
//! uniforms it declares. A material given the shader draws with a pipeline
//! built from the lit shader and the snippet; every other material keeps
//! the built-in hooks (identities, so its frames are unchanged).
//!
//! ```wgsl
//! // Every hook sees the fragment (or vertex) and the shader's parameters.
//! struct Hook { world: vec3<f32>, local: vec3<f32>, normal: vec3<f32>,
//!               uv: vec2<f32>, color: vec4<f32>, time: f32 }
//! fn vertex_offset(h: Hook, p: Params) -> vec3<f32>              // world-space offset (0)
//! fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> // base colour and alpha (c)
//! fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32> // world normal (n)
//! fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> // emitted light (e)
//! ```
//!
//! `Params` holds the declared parameters by name (`p.amount`), at most
//! [`MAX_SHADER_PARAMS`], read from the material's uniform; `h.time` is the
//! scene clock in seconds. An alpha below the material's `alphaTest`
//! discards the fragment (a dissolve). The hooks apply to static and
//! skinned meshes alike, and the shadow passes and the depth prepass see
//! the mesh where `vertex_offset` moved it and cut where `surface_color`'s
//! alpha falls below the alpha test (the shadow shader is built with the
//! same hooks, `shadow_source`). `og-cook` compiles every shader asset with naga for each tier
//! (WGSL, SPIR-V, MSL, and GLSL ES 3.0 for WebGL2) and refuses the asset
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
    /// The WGSL that defines the hooks (and any helpers they call).
    pub code: String,
    pub params: Vec<ShaderParam>,
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
fn defines(code: &str, name: &str) -> bool {
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
};
";

const DEFAULT_HOOKS: [&str; 4] = [
    "fn vertex_offset(h: Hook, p: Params) -> vec3<f32> { return vec3<f32>(0.0); }\n",
    "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }\n",
    "fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32> { return n; }\n",
    "fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> { return e; }\n",
];

/// The `Hook` and `Params` structs, `shader_params()`, the snippet and the
/// built-in hooks it leaves out. Placed after material.wgsl (it reads
/// `material.custom`).
pub(crate) fn hooks_section(params: &[ShaderParam], code: &str) -> String {
    let mut s = String::from("// Material shader hooks (hooks.rs).\n");
    s.push_str(HOOK_STRUCT);
    s.push_str("struct Params {\n");
    if params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\nfn shader_params() -> Params {\n  var p: Params;\n");
    for (i, p) in params.iter().enumerate() {
        s.push_str(&format!(
            "  p.{} = material.custom[{i}]{};\n",
            p.name,
            p.kind.swizzle()
        ));
    }
    s.push_str("  return p;\n}\n");
    s.push_str(code);
    s.push('\n');
    for (name, default) in SHADER_HOOKS.iter().zip(DEFAULT_HOOKS) {
        if !defines(code, name) {
            s.push_str(default);
        }
    }
    s
}

/// The built-in hooks (identities).
pub(crate) fn default_hooks() -> String {
    hooks_section(&[], "")
}

/// The lit shader module's source with `hooks` for one tier (`full`: the
/// storage-buffer joint palette, else the texture one), as the renderer
/// compiles it. The cook validates shader assets with this.
pub fn lit_module_source(shader: &MaterialShader, full: bool) -> String {
    crate::lit_source(&shader.hooks_wgsl(), full)
}

/// [`lit_module_source`] for the physical lit shader (`physical.rs`), which
/// a physical material with a material shader draws with.
pub fn physical_module_source(shader: &MaterialShader, full: bool) -> String {
    crate::physical_source(&shader.hooks_wgsl(), full)
}
