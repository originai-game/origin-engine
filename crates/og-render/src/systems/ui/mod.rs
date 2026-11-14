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
use glam::Vec2;
pub use og_render_data::ui::*;

impl crate::RenderScene {
    /// Adds a cooked vector for the overlay's [`UiVectorDraw`]s.
    pub fn add_ui_vector(&mut self, vector: UiVectorData) -> UiVectorHandle {
        self.ui.ui_vectors.push(vector);
        UiVectorHandle(self.ui.ui_vectors.len() as u32 - 1)
    }

    /// Adds a custom UI material for layers ([`UiMaterial::Custom`]).
    pub fn add_ui_material(&mut self, material: UiMaterialShader) -> UiMaterialHandle {
        self.ui.ui_materials.push(material);
        UiMaterialHandle(self.ui.ui_materials.len() as u32 - 1)
    }
}

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    // The UI pass (src/ui): its shapes, the built-in materials' composite
    // and a custom material's.
    add("ui", crate::ui::pass::ui_source());
    add("ui builtin materials", crate::ui::pass::builtin_source());
    add("ui mass", crate::ui::pass::mass_source());
    add(
        "ui material (duotone)",
        crate::ui::ui_material_module_source(&crate::ui::example_ui_material()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{ParamKind, ShaderParam};

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
