//! The post chain's frame resources: the colour adjustments' table baked
//! with the grading table when either changed, and the transmission's
//! copy of the opaque scene with its mip chain.

use super::*;

impl Renderer {
    /// Bakes the adjusted grading table; returns whether the frame adjusts.
    pub(super) fn prepare_colour_lut(&mut self, scene: &RenderScene) -> bool {
        let post_fx = &scene.look.post;
        // Colour adjustments: the grading table and the adjustments baked
        // into one (when either changed), bound under scene id 0, which no
        // scene has, so the bindings are made again when it comes or goes.
        let adjust = post_fx.adjust();
        if let Some(a) = &adjust {
            let grade = post_fx.color_grading.map(|g| {
                let i = g.lut.index();
                (
                    &scene.luts()[i],
                    (scene.id(), i, scene.lut_revisions()[i]),
                    g.intensity,
                )
            });
            self.post
                .bake_adjusted_lut(&self.device, &self.queue, a, grade);
        }
        adjust.is_some()
    }

    /// Makes the transmission's scene copy where a drawn surface refracts;
    /// returns whether one does.
    pub(super) fn prepare_transmission(
        &mut self,
        batches: &[Batch],
        blended_list: &DrawList,
        skinned_visible: &[Batch],
    ) -> bool {
        // Transmissive draws refract a copy of the opaque scene with its
        // mip chain: the main pass splits for it, as for water.
        let transmission = blended_list
            .ranges
            .iter()
            .any(|r| batches[r.0 as usize].transmissive)
            || skinned_visible.iter().any(|b| b.transmissive);
        if transmission {
            self.mips.prepare(&self.device, HDR_FORMAT);
            let stale = self
                .transmission
                .as_ref()
                .is_none_or(|(t, ..)| (t.width(), t.height()) != (self.width, self.height));
            if stale {
                let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("transmission"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: mips::mip_count(self.width, self.height),
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                let chain = self.mips.chain(&self.device, &texture);
                self.transmission = Some((texture, view, chain));
            }
        }
        transmission
    }
}
