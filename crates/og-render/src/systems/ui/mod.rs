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
    fn ui_list_retained_shapes_order_count_and_sharing() {
        use std::sync::Arc;

        let mut list = UiList::default();
        assert!(list.is_empty());
        assert_eq!(list.shape_count(), 0);
        assert_eq!(list.shapes().count(), 0);
        list.retained_shapes = Some(Arc::new(Vec::new()));
        assert!(list.is_empty());
        assert_eq!(list.shape_count(), 0);
        assert_eq!(list.shapes().count(), 0);

        let shape = |x| UiShape::rect([x, 0.0, 2.0, 2.0], 0.0, [1.0; 4]);
        list.shapes.push(shape(20.0));
        assert!(!list.is_empty());
        assert_eq!(list.shape_count(), 1);
        assert!(std::ptr::eq(list.shapes().next().unwrap(), &list.shapes[0]));

        list.shapes.clear();
        let retained = Arc::new(vec![shape(0.0), shape(10.0)]);
        list.retained_shapes = Some(retained.clone());
        assert!(!list.is_empty());
        assert_eq!(list.shape_count(), 2);
        assert_eq!(list.shapes().count(), 2);
        assert!(std::ptr::eq(list.shapes().next().unwrap(), &retained[0]));

        list.shapes.extend([shape(20.0), shape(30.0)]);
        assert_eq!(list.shape_count(), 4);
        assert_eq!(
            list.shapes().map(|s| s.center.x).collect::<Vec<_>>(),
            vec![1.0, 11.0, 21.0, 31.0]
        );
        let cloned = list.clone();
        assert!(Arc::ptr_eq(
            cloned.retained_shapes.as_ref().unwrap(),
            &retained
        ));
        assert_eq!(cloned.shape_count(), 4);
        assert_eq!(cloned.shapes().count(), 4);
        list.retained_shapes = None;
        assert_eq!(list.shape_count(), 2);
        assert_eq!(list.shapes().count(), 2);
    }

    fn upload_ranges(previous: Option<&[u8]>, bytes: &[u8]) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        crate::gpu::buffers::changed_ranges(
            previous,
            bytemuck::cast_slice::<u8, [u8; 4]>(bytes),
            |range| ranges.push(range),
        );
        ranges
    }

    #[test]
    fn retained_upload_ranges_coalesce_adjacent_structured_instances() {
        let old = [build::UiGpu::default(); 6];
        let mut new = old;
        // Separated transform and opacity fields would produce several
        // dirty word runs per instance, but each instance is one element.
        for instance in &mut new[1..4] {
            instance.xform[0] = 1.0;
            instance.color[3] = 0.5;
        }
        let stride = std::mem::size_of::<build::UiGpu>();
        let mut ranges = Vec::new();
        crate::gpu::buffers::changed_ranges(Some(bytemuck::cast_slice(&old)), &new, |range| {
            ranges.push(range);
        });
        assert_eq!(ranges, vec![stride..4 * stride]);
        // One untouched instance separates a genuinely disjoint edit.
        new[5].extra[3] = 2.0;
        ranges.clear();
        crate::gpu::buffers::changed_ranges(Some(bytemuck::cast_slice(&old)), &new, |range| {
            ranges.push(range);
        });
        assert_eq!(ranges, vec![stride..4 * stride, 5 * stride..6 * stride]);
        // A shortened live prefix regrows in whole instance units, and a
        // replacement must upload even the unchanged leading instance.
        ranges.clear();
        crate::gpu::buffers::changed_ranges(Some(bytemuck::cast_slice(&old[..1])), &old, |r| {
            ranges.push(r);
        });
        assert_eq!(ranges, vec![stride..6 * stride]);
        ranges.clear();
        crate::gpu::buffers::changed_ranges(None, &old, |range| ranges.push(range));
        assert_eq!(ranges, vec![0..6 * stride]);
    }

    #[test]
    fn retained_upload_ranges_cover_only_changed_aligned_spans() {
        let old = [0u8; 32];
        assert!(upload_ranges(Some(&old), &old).is_empty());
        let mut new = old;
        // Changes within words expand to aligned writes; adjacent dirty
        // words merge, without swallowing the unchanged gap.
        new[9] = 1;
        new[14] = 2;
        assert_eq!(upload_ranges(Some(&old), &new), vec![8..16]);
        new[31] = 3;
        assert_eq!(upload_ranges(Some(&old), &new), vec![8..16, 28..32]);
        new[0] = 4;
        assert_eq!(upload_ranges(Some(&old), &new), vec![0..4, 8..16, 28..32]);
    }

    #[test]
    fn retained_upload_ranges_shrink_regrow_and_initialize_replacements() {
        let full = [7u8; 32];
        let short = &full[..12];
        assert_eq!(upload_ranges(None, &full), vec![0..32]);
        assert!(upload_ranges(Some(&full), short).is_empty());
        // The discarded tail is no longer part of the snapshot, even if
        // regrowth supplies exactly the same bytes as two frames ago.
        assert_eq!(upload_ranges(Some(short), &full), vec![12..32]);
        assert!(upload_ranges(Some(&full), &[]).is_empty());
        assert_eq!(upload_ranges(Some(&[]), &full), vec![0..32]);
        // Growth or any other buffer replacement invalidates the snapshot:
        // identical prefixes must still be written to the new allocation.
        assert_eq!(upload_ranges(None, short), vec![0..12]);
        assert!(upload_ranges(None, &[]).is_empty());
    }

    #[test]
    fn retained_upload_ranges_reconstruct_live_bytes_across_edits() {
        let mut snapshot = Vec::new();
        let mut device_bytes = [0xcc; 64];
        let mut previous_valid = false;
        for (length, changes, replacement) in [
            (32, vec![(3, 1), (17, 2)], false),
            (32, vec![(3, 1), (17, 3), (31, 4)], false),
            (12, vec![(3, 1)], false),
            (32, vec![(3, 1), (17, 3), (31, 4)], false),
            (48, vec![(3, 1)], true),
            (48, vec![(3, 1)], true),
            (0, vec![], false),
            (16, vec![], false),
        ] {
            let mut bytes = vec![0; length];
            for (offset, value) in changes {
                bytes[offset] = value;
            }
            if replacement {
                device_bytes.fill(0xcc);
            }
            let previous = (previous_valid && !replacement).then_some(snapshot.as_slice());
            for range in upload_ranges(previous, &bytes) {
                assert_eq!(range.start % 4, 0);
                assert_eq!(range.end % 4, 0);
                device_bytes[range.clone()].copy_from_slice(&bytes[range]);
            }
            assert_eq!(&device_bytes[..length], bytes);
            snapshot = bytes;
            previous_valid = true;
        }
    }

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
