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
pub(crate) mod overlay;
pub(crate) mod paint;
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
    add("overlay", crate::overlay::overlay_source());
    // The UI pass (src/ui): its shapes, the built-in materials' composite
    // and a custom material's.
    add("ui", crate::ui::pass::ui_source());
    add("ui builtin materials", crate::ui::pass::builtin_source());
    add("ui mass", crate::ui::pass::mass_source());
    add("ui paint", crate::ui::paint::source());
    add("ui filter", crate::ui::filter::source());
    add(
        "ui material (duotone)",
        crate::ui::ui_material_module_source(&crate::ui::example_ui_material()),
    );
}
