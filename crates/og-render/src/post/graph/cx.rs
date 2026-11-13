//! What the post families bind and encode with: the realized graph, the
//! post pipelines and the views every family reads the same way (the lit
//! image at each stage, the exposure, the prepass or its placeholders).

use super::*;

pub(in crate::post) fn tex(v: &wgpu::TextureView) -> wgpu::BindingResource<'_> {
    wgpu::BindingResource::TextureView(v)
}

/// A pass's bind groups, one per TAA history slot.
pub(in crate::post) type Pair = [wgpu::BindGroup; 2];

/// What a family's bind groups are made from, for one realization of the
/// graph.
pub(in crate::post) struct BindCx<'a> {
    pub device: &'a wgpu::Device,
    pub graph: &'a FrameGraph,
    pub passes: &'a PostPasses,
    pub persistent: &'a Persistent,
    pub probes: &'a wgpu::Buffer,
    /// The frame's passes.
    pub frame: &'a [FramePass],
    pub cfg: &'a GraphConfig,
    /// The grading table.
    pub lut: &'a wgpu::TextureView,
    /// The stylise chain's palette, canvas and ramp.
    pub stylise: (
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
    ),
    pub names: LitNames,
}

impl<'a> BindCx<'a> {
    /// A bind group of `res` at bindings 0, 1, 2 and so on.
    pub fn group(
        &self,
        label: &str,
        layout: &wgpu::BindGroupLayout,
        res: &[wgpu::BindingResource],
    ) -> wgpu::BindGroup {
        let entries: Vec<_> = res
            .iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: r.clone(),
            })
            .collect();
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &entries,
        })
    }

    /// A group per history slot.
    pub fn pair(&self, f: impl Fn(usize) -> wgpu::BindGroup) -> Pair {
        [f(0), f(1)]
    }

    pub fn sampler(&self) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::Sampler(&self.passes.sampler)
    }

    pub fn has(&self, p: FramePass) -> bool {
        self.frame.contains(&p)
    }

    pub fn view(&self, name: &str) -> &'a wgpu::TextureView {
        self.graph.view(name)
    }

    /// The target `name` in a frame with `pass`, else black.
    pub fn view_of(&self, pass: FramePass, name: &str) -> &'a wgpu::TextureView {
        if self.has(pass) {
            self.view(name)
        } else {
            &self.passes.black
        }
    }

    /// The TAA history, in a frame with TAA.
    pub fn history(&self) -> Option<&'a [wgpu::TextureView; 2]> {
        self.persistent
            .history
            .as_ref()
            .filter(|_| self.has(FramePass::Taa))
    }

    /// The lit image in frame slot `slot` at a stage ([`LitNames`]).
    pub fn stage(&self, name: Option<&'static str>, slot: usize) -> &'a wgpu::TextureView {
        match name {
            Some(n) => self.view(n),
            None => &self.history().expect("TAA history")[slot],
        }
    }

    /// The lit image as the last effect left it.
    pub fn source(&self, slot: usize) -> &'a wgpu::TextureView {
        self.stage(self.names.last, slot)
    }

    /// The metered luminance's last level (black without metering).
    pub fn metered(&self) -> &'a wgpu::TextureView {
        self.view_of(FramePass::Meter, &luminance_name(meter_levels() - 1))
    }

    /// The exposure the passes of frame slot `slot` read: the adapted one
    /// this slot writes, or the metered texel.
    pub fn lum(&self, slot: usize) -> &'a wgpu::TextureView {
        if self.has(FramePass::Adapt) {
            &self.passes.exposure.views[slot]
        } else {
            self.metered()
        }
    }

    /// The prepass's view depth (far without a prepass).
    pub fn depth(&self) -> &'a wgpu::TextureView {
        self.graph
            .try_view("lindepth")
            .unwrap_or(&self.passes.far_depth)
    }

    /// The prepass's normals and velocity (black without a prepass).
    pub fn gbuffer(&self) -> &'a wgpu::TextureView {
        self.graph.try_view("gbuffer").unwrap_or(&self.passes.black)
    }
}

/// What a family encodes one of its passes with.
pub(in crate::post) struct PostCx<'a> {
    pub graph: &'a FrameGraph,
    pub passes: &'a PostPasses,
    pub persistent: &'a Persistent,
    /// This frame's slot of the two-frame histories.
    pub slot: usize,
    /// Where the last pass of the frame draws.
    pub output: &'a wgpu::TextureView,
    pub gtao: bool,
}

/// The clear every post target starts from.
pub(in crate::post) const CLEAR: wgpu::LoadOp<wgpu::Color> =
    wgpu::LoadOp::Clear(wgpu::Color::BLACK);
