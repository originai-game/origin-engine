//! The depth prepass's form on the device (`docs/design.md` §2.6 Tiers).
//!
//! The prepass writes three targets: normals and velocity (`gbuffer`), the
//! linear depth with the reflectivity (`lindepth`) and the outline ids
//! (`outline-id`). With [`Caps::MRT`] each prepass pipeline writes all
//! three at once ([`PrepassForm::Targets`]). A device without it (the
//! mini-game tier, whose WebGPU takes only `colorAttachments[0]`) draws
//! the same pipelines in the packed form ([`PrepassForm::Packed`]): each
//! prepass entry's `_packed` twin writes the normals, velocity and depth
//! into one Rgba32Uint texel (common.wgsl `prepass_pack`), and
//! [`PrepassUnpack`] writes them back into `gbuffer` and `lindepth` with
//! two fullscreen passes, so every pass that reads the prepass reads the
//! same targets on every tier. The outline ids do not fit beside them:
//! `outline-id` is cleared, so a pass that reads them sees no object ids
//! on such a device.
//!
//! The form is the renderer's, fixed when it is made, so the prepass
//! pipelines' caches keep their keys: each builder asks the form for its
//! fragment entry and targets.

use crate::Caps;
use crate::deferred::Deferred;
use std::borrow::Cow;

/// The packed prepass target: two halves of normal, two of velocity, the
/// depth and the reflectivity's bits (common.wgsl `prepass_pack`).
pub(crate) const PACKED_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Uint;

/// How the device draws the depth prepass. See the module documentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum PrepassForm {
    /// The three targets at once.
    #[default]
    Targets,
    /// One packed target, unpacked after the pass.
    Packed,
}

const fn target(format: wgpu::TextureFormat) -> Option<wgpu::ColorTargetState> {
    Some(wgpu::ColorTargetState {
        format,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    })
}

const TARGETS: [Option<wgpu::ColorTargetState>; 3] = [
    target(super::GBUFFER_FORMAT),
    target(super::LINDEPTH_FORMAT),
    target(super::OUTLINE_FORMAT),
];
const PACKED: [Option<wgpu::ColorTargetState>; 1] = [target(PACKED_FORMAT)];

impl PrepassForm {
    /// The form a device with `caps` draws.
    pub(crate) fn of(caps: Caps) -> PrepassForm {
        if caps.contains(Caps::MRT) {
            PrepassForm::Targets
        } else {
            PrepassForm::Packed
        }
    }

    /// The fragment entry point of a prepass pipeline whose three-target
    /// entry is `entry`: its `_packed` twin in the packed form.
    pub(crate) fn entry(self, entry: &str) -> Cow<'_, str> {
        match self {
            PrepassForm::Targets => Cow::Borrowed(entry),
            PrepassForm::Packed => Cow::Owned(format!("{entry}_packed")),
        }
    }

    /// A prepass pipeline's colour targets.
    pub(crate) fn targets(self) -> &'static [Option<wgpu::ColorTargetState>] {
        match self {
            PrepassForm::Targets => &TARGETS,
            PrepassForm::Packed => &PACKED,
        }
    }
}

const UNPACK_WGSL: &str = concat!(
    wgsl_library!("fullscreen.wgsl"),
    wgsl!("prepass_unpack.wgsl")
);

/// The fullscreen passes that write a packed prepass back into its
/// targets (prepass_unpack.wgsl).
pub(crate) struct PrepassUnpack {
    layout: wgpu::BindGroupLayout,
    gbuffer: Deferred<wgpu::RenderPipeline>,
    lindepth: Deferred<wgpu::RenderPipeline>,
}

impl PrepassUnpack {
    /// The module's source, for `tests/shaders.rs`.
    pub(crate) fn source() -> String {
        UNPACK_WGSL.to_string()
    }

    pub(crate) fn new(device: &wgpu::Device) -> PrepassUnpack {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prepass unpack"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("prepass unpack"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::shader(device, "prepass unpack", Self::source());
        let pipeline = |label, entry, format| {
            super::deferred_fullscreen(
                device,
                label,
                &module,
                entry,
                &pipeline_layout,
                format,
                None,
            )
        };
        PrepassUnpack {
            gbuffer: pipeline(
                "prepass unpack normals",
                "fs_gbuffer",
                super::GBUFFER_FORMAT,
            ),
            lindepth: pipeline(
                "prepass unpack depth",
                "fs_lindepth",
                super::LINDEPTH_FORMAT,
            ),
            layout,
        }
    }

    /// Writes `packed` into `lindepth`, and into `gbuffer` when one is
    /// given (a view's resolve reads its depth alone).
    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut crate::Encoder,
        packed: &wgpu::TextureView,
        gbuffer: Option<&wgpu::TextureView>,
        lindepth: &wgpu::TextureView,
    ) {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prepass unpack"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(packed),
            }],
        });
        let clear = wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT);
        if let Some(gbuffer) = gbuffer {
            super::fullscreen_pass(
                encoder,
                "prepass unpack normals",
                gbuffer,
                clear,
                &self.gbuffer,
                &group,
            );
        }
        super::fullscreen_pass(
            encoder,
            "prepass unpack depth",
            lindepth,
            clear,
            &self.lindepth,
            &group,
        );
    }
}

/// The clear value of a packed prepass target: what `PrepassUnpack` turns
/// into the targets' own clears (no normal, no velocity, the far depth, no
/// reflectivity).
pub(crate) fn packed_clear() -> wgpu::Color {
    wgpu::Color {
        r: 0.0,
        g: 0.0,
        b: f64::from(super::FAR_DEPTH.to_bits()),
        a: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_form_follows_mrt() {
        assert_eq!(PrepassForm::of(Caps::FULL), PrepassForm::Targets);
        let mini = Caps::FULL & crate::Tier::MiniGame.ceiling();
        assert_eq!(PrepassForm::of(mini), PrepassForm::Packed);
        assert_eq!(PrepassForm::Packed.entry("fs_prepass"), "fs_prepass_packed");
        assert_eq!(PrepassForm::Packed.targets().len(), 1);
        assert_eq!(PrepassForm::Targets.targets().len(), 3);
    }
}
