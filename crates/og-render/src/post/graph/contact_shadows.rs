//! The contact shadows (`contact.rs`): the lights' masks, then the
//! compose of the masks with the AO it passes through (white without the
//! AO passes) into `ao`.

use super::*;

pub(super) struct ContactShadows {
    lights: wgpu::BindGroup,
    compose: wgpu::BindGroup,
    /// The masks both bind.
    masks: wgpu::Buffer,
}

impl ContactShadows {
    pub fn bind(cx: &BindCx) -> Option<ContactShadows> {
        if !cx.has(FramePass::Contact) {
            return None;
        }
        let passes = cx.passes;
        let c = &passes.contact;
        let masks = contact::mask_buffer(cx.device, (cx.cfg.width, cx.cfg.height));
        let dispatch = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &c.buffer,
            offset: 0,
            size: wgpu::BufferSize::new(c.stride as u64),
        });
        let lights = crate::volumetric::bind_at(
            cx.device,
            "contact",
            &c.layout,
            &[
                (0, passes.fx_buffer.as_entire_binding()),
                (1, dispatch),
                (2, tex(cx.depth())),
                (3, tex(cx.gbuffer())),
                (4, masks.as_entire_binding()),
            ],
        );
        let compose = crate::volumetric::bind_at(
            cx.device,
            "contact compose",
            &c.compose_layout,
            &[
                (0, passes.fx_buffer.as_entire_binding()),
                (5, tex(cx.graph.try_view("ao.pre").unwrap_or(&passes.white))),
                (6, masks.as_entire_binding()),
                (9, tex(cx.view("ao"))),
            ],
        );
        Some(ContactShadows {
            lights,
            compose,
            masks,
        })
    }

    pub fn encode(&self, encoder: &mut crate::Encoder, cx: &PostCx) -> u32 {
        cx.passes
            .contact
            .encode(encoder, &self.lights, &self.compose, &self.masks)
    }
}
