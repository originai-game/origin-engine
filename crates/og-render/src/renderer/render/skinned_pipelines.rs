//! The skinned batches' pipelines for the frame's passes: the lit ones
//! (physical where the main pass draws them so, standard for the planar
//! reflection), the shadow sets and the prepass ones.

use super::*;

/// The pipelines each skinned batch draws with, by pass.
pub(super) struct SkinnedPipelines<'a> {
    pub lit: Vec<&'a wgpu::RenderPipeline>,
    pub lit_standard: Vec<&'a wgpu::RenderPipeline>,
    pub shadow: Vec<&'a ShadowSet>,
    pub prepass: Vec<&'a wgpu::RenderPipeline>,
}

impl Renderer {
    /// Picks the skinned batches' pipelines for the frame's passes.
    pub(super) fn skinned_pipelines<'a>(
        &'a self,
        skinned_visible: &[Batch],
        skinned_batches: &[Batch],
        prepass: bool,
    ) -> SkinnedPipelines<'a> {
        // The main pass draws physical materials with their pipelines, the
        // planar reflection with the standard ones.
        let lit_skinned: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, b.physical))
            .collect();
        let lit_skinned_standard: Vec<&wgpu::RenderPipeline> = skinned_visible
            .iter()
            .map(|b| self.lit_pipeline_of(b, true, false))
            .collect();
        let shadow_skinned: Vec<&ShadowSet> = skinned_batches
            .iter()
            .map(|b| match b.shader {
                Some(sh) => self.cache.hook_shadow_pipelines[&(sh, b.faces, true)].get(),
                None => self.shadows.shadow_pipelines[&(b.faces, true, b.dissolve)].get(),
            })
            .collect();
        let prepass_skinned: Vec<&wgpu::RenderPipeline> = if prepass {
            skinned_visible
                .iter()
                .map(|b| {
                    let hooked = b
                        .shader
                        .and_then(|sh| self.cache.hook_prepass_pipelines.get(&(sh, b.faces, true)));
                    let array = self
                        .lit
                        .array_maps
                        .as_ref()
                        .filter(|_| b.array)
                        .and_then(|a| a.prepass.get(&(b.faces, true)));
                    hooked
                        .or(array)
                        .map(|p| p.get())
                        .or_else(|| {
                            self.prepass
                                .prepass_pipelines
                                .get(&(b.faces, true))
                                .map(|p| p.get(self.frame_features))
                        })
                        .unwrap_or(lit_skinned[0])
                })
                .collect()
        } else {
            Vec::new()
        };
        SkinnedPipelines {
            lit: lit_skinned,
            lit_standard: lit_skinned_standard,
            shadow: shadow_skinned,
            prepass: prepass_skinned,
        }
    }
}
