use crate::hooks::{MAX_SHADER_PARAMS, ParamKind, ShaderParam};

/// The most custom post passes one frame runs.
pub const MAX_CUSTOM_POSTS: usize = 8;

/// A custom post pass: its WGSL `post` function and parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct PostShader {
    pub name: String,
    pub code: String,
    pub params: Vec<ShaderParam>,
}

impl PostShader {
    /// Parameter `name`'s slot and type.
    pub fn param(&self, name: &str) -> Option<(usize, ParamKind)> {
        self.params
            .iter()
            .position(|p| p.name == name)
            .map(|i| (i, self.params[i].kind))
    }

    /// The parameters at their defaults.
    pub fn defaults(&self) -> [[f32; 4]; MAX_SHADER_PARAMS] {
        let mut out = [[0.0; 4]; MAX_SHADER_PARAMS];
        for (slot, p) in out.iter_mut().zip(&self.params) {
            *slot = p.default;
        }
        out
    }

    /// The parameters' count and names, and that `post` is defined.
    pub fn check(&self) -> Result<(), String> {
        let as_material = crate::MaterialShader {
            name: self.name.clone(),
            code: "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }"
                .into(),
            params: self.params.clone(),
            mode: Default::default(),
        };
        as_material.check()?;
        if !crate::hooks::defines(&self.code, "post") {
            return Err(
                "a post pass defines `fn post(f: PostInput, p: Params) -> vec4<f32>`".into(),
            );
        }
        Ok(())
    }
}

const POST_CUSTOM_WGSL: &str = wgsl!("post_custom.wgsl");

/// A post pass's module source.
pub fn post_module_source(post: &PostShader) -> String {
    let mut s = String::from(crate::LIBRARY_WGSL);
    s.push_str(POST_CUSTOM_WGSL);
    s.push_str("struct Params {\n");
    if post.params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in &post.params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\nfn post_params() -> Params {\n  var p: Params;\n");
    for (i, p) in post.params.iter().enumerate() {
        let swizzle = match p.kind {
            ParamKind::Float => ".x",
            ParamKind::Vec2 => ".xy",
            ParamKind::Vec3 => ".xyz",
            ParamKind::Vec4 => "",
        };
        s.push_str(&format!("  p.{} = u.params[{i}]{swizzle};\n", p.name));
    }
    s.push_str("  return p;\n}\n// The pass's code (post_custom.rs).\n");
    s.push_str(&post.code);
    s.push('\n');
    s
}

/// A CRT look as a post pass: the frame bent onto a curved screen (black
/// outside it), scanlines, an RGB shadow mask and a vignette (validated
/// with the engine's modules, `shaders.rs`; og-tests' `post-crt`).
pub fn example_crt() -> PostShader {
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    PostShader {
        name: "crt".into(),
        code: CRT_WGSL.into(),
        params: vec![
            param("curve", ParamKind::Float, [0.12, 0.0, 0.0, 0.0]),
            param("scanlines", ParamKind::Float, [0.35, 0.0, 0.0, 0.0]),
            param("lines", ParamKind::Float, [240.0, 0.0, 0.0, 0.0]),
            param("vignette", ParamKind::Float, [0.6, 0.0, 0.0, 0.0]),
        ],
    }
}

const CRT_WGSL: &str = "fn post(f: PostInput, p: Params) -> vec4<f32> {
  // The screen bulges: uv pushed outwards with its distance from the centre.
  let c = f.uv * 2.0 - 1.0;
  let bent = c * (1.0 + p.curve * dot(c, c));
  let uv = bent * 0.5 + 0.5;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }
  var color = previous(uv).rgb;
  // Scanlines across the bent image.
  let line = 0.5 + 0.5 * cos(uv.y * p.lines * 6.2831853);
  color *= 1.0 - p.scanlines * line;
  // The shadow mask: each pixel column favours red, green or blue.
  let column = u32(f.pixel.x) % 3u;
  var mask = vec3<f32>(0.85);
  mask[column] = 1.15;
  color *= mask;
  // Darker towards the rounded corners.
  let edge = uv * (1.0 - uv);
  color *= pow(clamp(edge.x * edge.y * 16.0, 0.0, 1.0), p.vignette * 0.5);
  return vec4<f32>(color, 1.0);
}
";
