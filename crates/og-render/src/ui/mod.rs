//! The UI pass (`docs/ui.md` §3.3): analytic primitives drawn in the overlay's
//! order, after every post effect. Every shape is a signed distance
//! function evaluated per pixel (`shaders/ui.wgsl`), so edges are exact at
//! any scale and transform; every paint, stroke and effect of a shape is
//! one instance of one family of pipelines.
//!
//! What a frame submits is [`UiList`] ([`crate::Overlay::ui`]):
//!
//! - [`UiShape`]s: a geometry (rectangle with four radii and a corner
//!   shape, ellipse, arc, polygon or star, polyline), a paint (solid,
//!   gradient, image, an offscreen layer), a mode (fill, stroke, drop or
//!   inner shadow, backdrop), opacity and blend. A layered element is
//!   several shapes on one painter level, in order.
//! - [`UiClip`]s: a clip rectangle of [`crate::Overlay::clips`] that also
//!   cuts what uses it to a shape, antialiased; sprites and text included.
//! - [`UiLayer`]s: the overlay layers in a range drawn offscreen, then
//!   composited by a [`UiShape`] whose paint is [`UiPaint::Layer`] through a
//!   built-in or custom material ([`UiMaterialShader`]).
//! - [`UiTextEffects`]: outline, gradient fill and shadows of one overlay
//!   text, read from its font's distance field.
//! - [`UiVectorDraw`]s: a cooked vector ([`UiVectorData`], tessellated at
//!   cook with an antialiasing fringe) drawn into a rectangle.
//! - [`UiMassBatch`]es: mass elements ([`mass`], §14), each an instance of
//!   a template whose motion and number formatting run in the shader.
//! - [`UiOccluder`]s: a depth test against the scene for what uses a clip
//!   rectangle (anchored elements behind a wall fade or hide).
//!
//! Sorting and batching (§5.7) are the overlay's: by layer, then by
//! pipeline and texture; consecutive instances of one batch are one draw.
//! A layer starts passes of its own for its region only; a backdrop or a
//! blend that reads what is under it copies the frame drawn so far, as
//! glass does (src/glass.rs).

pub(crate) mod build;
pub mod filter;
pub mod mass;
pub(crate) mod pass;

pub use og_draw::ui::*;

use crate::TextureHandle;
use crate::hooks::{ParamKind, ShaderParam};
use glam::Vec2;

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

impl crate::RenderScene {
    /// Adds a cooked vector for the overlay's [`UiVectorDraw`]s.
    pub fn add_ui_vector(&mut self, vector: UiVectorData) -> UiVectorHandle {
        self.ui_vectors.push(vector);
        UiVectorHandle(self.ui_vectors.len() as u32 - 1)
    }

    /// Adds a custom UI material for layers ([`UiMaterial::Custom`]).
    pub fn add_ui_material(&mut self, material: UiMaterialShader) -> UiMaterialHandle {
        self.ui_materials.push(material);
        UiMaterialHandle(self.ui_materials.len() as u32 - 1)
    }
}

pub(crate) const UI_MATERIAL_WGSL: &str = wgsl!("ui_material.wgsl");
pub(crate) const UI_BLEND_WGSL: &str = wgsl!("ui_blend.wgsl");
pub(crate) const UI_BLEND_MODES_WGSL: &str = wgsl!("ui_blend_modes.wgsl");

/// The module a custom UI material compiles to: the UI pass's common code,
/// the composite's prelude, its `Params` and its code.
pub fn ui_material_module_source(material: &UiMaterialShader) -> String {
    let mut s = String::from(UI_COMMON_WGSL);
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
pub(crate) const UI_COMMON_WGSL: &str = wgsl!("ui_common.wgsl");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklab_round_trips_and_mixes_apart_from_linear_light() {
        for c in [
            [1.0, 0.0, 0.0],
            [0.2, 0.5, 0.9],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
        ] {
            let back = oklab_to_linear(oklab(c));
            for k in 0..3 {
                assert!((back[k] - c[k]).abs() < 1e-4, "{c:?} -> {back:?}");
            }
        }
        // Blue to yellow: half way in OKLab is perceptually half way in
        // lightness (L 0.71), where linear light's middle, grey 0.5, reads
        // far lighter (L 0.79): the two mixes are not the same colour.
        let (a, b) = (oklab([0.0, 0.0, 1.0]), oklab([1.0, 1.0, 0.0]));
        let mid = oklab_to_linear([
            0.5 * (a[0] + b[0]),
            0.5 * (a[1] + b[1]),
            0.5 * (a[2] + b[2]),
        ]);
        let l = |c: [f32; 3]| oklab(c)[0];
        assert!((l(mid) - 0.5 * (a[0] + b[0])).abs() < 1e-3);
        assert!(l([0.5, 0.5, 0.5]) - l(mid) > 0.05, "{mid:?}");
    }

    #[test]
    fn a_cubic_flattens_within_its_tolerance_and_keeps_its_ends() {
        let (p0, p1, p2, p3) = (
            Vec2::ZERO,
            Vec2::new(0.0, 100.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(100.0, 0.0),
        );
        let pts = flatten_cubic(p0, p1, p2, p3, 0.25);
        assert_eq!(pts.first(), Some(&p0));
        assert_eq!(pts.last(), Some(&p3));
        // Midpoints of each chord lie within the tolerance of the curve.
        let curve = |t: f32| {
            let u = 1.0 - t;
            p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
        };
        let n = pts.len() - 1;
        for i in 0..n {
            let mid = (pts[i] + pts[i + 1]) * 0.5;
            let on = curve((i as f32 + 0.5) / n as f32);
            assert!(mid.distance(on) <= 0.3, "chord {i}: {}", mid.distance(on));
        }
    }

    #[test]
    fn a_material_must_define_its_function_and_few_params() {
        assert!(example_ui_material().check().is_ok());
        let mut bad = example_ui_material();
        bad.code = "fn other() {}".into();
        assert!(bad.check().is_err());
        let mut many = example_ui_material();
        many.params = (0..5)
            .map(|i| ShaderParam {
                name: format!("p{i}"),
                kind: ParamKind::Float,
                default: [0.0; 4],
            })
            .collect();
        assert!(many.check().is_err());
    }
}
