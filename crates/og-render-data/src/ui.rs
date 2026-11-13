use crate::hooks::{ParamKind, ShaderParam};
use og_draw::ui::UI_MATERIAL_PARAMS;

/// A custom UI material (`type ui-material`): WGSL defining
///
/// ```wgsl
/// fn material(m: MaterialInput, p: Params) -> vec4<f32>
/// struct MaterialInput { uv: vec2<f32>, pixel: vec2<f32>, size: vec2<f32>, time: f32 }
/// ```
///
/// returning the composite's straight-alpha linear colour at `m.uv` (0..1
/// over the layer's rectangle, `size` CSS px, `pixel` the CSS px within it).
/// It reads the layer with `layer(uv)` (straight alpha, linear) and
/// `layer_texel()` (the uv of one CSS px), and its parameters (at most
/// [`UI_MATERIAL_PARAMS`], set per layer in [`UiLayer::params`]) from `p`.
/// The cook validates it with naga for Metal, Vulkan and DX12.
#[derive(Clone, Debug, PartialEq)]
pub struct UiMaterialShader {
    pub name: String,
    pub code: String,
    pub params: Vec<ShaderParam>,
}

impl UiMaterialShader {
    /// Parameter `name`'s slot and type.
    pub fn param(&self, name: &str) -> Option<(usize, ParamKind)> {
        self.params
            .iter()
            .position(|p| p.name == name)
            .map(|i| (i, self.params[i].kind))
    }

    /// The parameters at their defaults.
    pub fn defaults(&self) -> [[f32; 4]; UI_MATERIAL_PARAMS] {
        let mut out = [[0.0; 4]; UI_MATERIAL_PARAMS];
        for (slot, p) in out.iter_mut().zip(&self.params) {
            *slot = p.default;
        }
        out
    }

    /// The parameters' count and names, and that `material` is defined.
    pub fn check(&self) -> Result<(), String> {
        if self.params.len() > UI_MATERIAL_PARAMS {
            return Err(format!(
                "a UI material has at most {UI_MATERIAL_PARAMS} params, not {}",
                self.params.len()
            ));
        }
        for p in &self.params {
            let ok = p
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
                && p.name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !ok {
                return Err(format!("param `{}`: not a WGSL identifier", p.name));
            }
        }
        if !crate::hooks::defines(&self.code, "material") {
            return Err(
                "a UI material defines `fn material(m: MaterialInput, p: Params) -> vec4<f32>`"
                    .into(),
            );
        }
        Ok(())
    }
}

pub const UI_MATERIAL_WGSL: &str = wgsl!("ui_material.wgsl");
pub const UI_BLEND_WGSL: &str = wgsl!("ui_blend.wgsl");
pub const UI_BLEND_MODES_WGSL: &str = wgsl!("ui_blend_modes.wgsl");

/// The module a custom UI material compiles to: the UI pass's common code,
/// the composite's prelude, its `Params` and its code.
pub fn ui_material_module_source(material: &UiMaterialShader) -> String {
    let mut s = String::from(crate::HASH_WGSL);
    s.push_str(UI_COMMON_WGSL);
    s.push_str(UI_BLEND_MODES_WGSL);
    s.push_str(UI_BLEND_WGSL);
    s.push_str(UI_MATERIAL_WGSL);
    s.push_str("struct Params {\n");
    if material.params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in &material.params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\nfn material_params() -> Params {\n  var p: Params;\n");
    for (i, p) in material.params.iter().enumerate() {
        let swizzle = match p.kind {
            ParamKind::Float => ".x",
            ParamKind::Vec2 => ".xy",
            ParamKind::Vec3 => ".xyz",
            ParamKind::Vec4 => "",
        };
        s.push_str(&format!(
            "  p.{} = layer_params.params[{i}]{swizzle};\n",
            p.name
        ));
    }
    s.push_str("  return p;\n}\n// The material's code (ui/mod.rs).\n");
    s.push_str(&material.code);
    s.push('\n');
    s
}

/// A sample custom material (`tests/shaders.rs` validates it; og-tests'
/// `ui-layers` draws it): the layer's colours turned into a duotone of two
/// parameters by their luminance, with a moving band.
pub fn example_ui_material() -> UiMaterialShader {
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    UiMaterialShader {
        name: "duotone".into(),
        code: DUOTONE_WGSL.into(),
        params: vec![
            param("dark", ParamKind::Vec4, [0.02, 0.05, 0.2, 1.0]),
            param("light", ParamKind::Vec4, [1.0, 0.75, 0.3, 1.0]),
            param("band", ParamKind::Float, [0.25, 0.0, 0.0, 0.0]),
        ],
    }
}

const DUOTONE_WGSL: &str = "fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let y = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  var rgb = mix(p.dark.rgb, p.light.rgb, smoothstep(0.0, 0.6, y));
  // A soft band across the layer, moving with the UI clock.
  let s = fract(m.uv.x + m.uv.y * 0.5 - m.time * 0.25);
  rgb += vec3<f32>(p.band) * smoothstep(0.1, 0.0, abs(s - 0.5));
  return vec4<f32>(rgb, c.a);
}
";

/// The code every UI module starts with: the frame uniform, the clip and
/// gradient tables, the signed distance functions and the paints.
pub const UI_COMMON_WGSL: &str = wgsl!("ui_common.wgsl");
