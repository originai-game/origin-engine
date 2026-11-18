//! The effect passes of fxpost.wgsl: one layout (a uniform, two images,
//! the view depth and the gbuffer) and one pipeline per entry point. A
//! family names what each of its passes binds ([`ao`], [`outline`],
//! [`screen`]) and which entry draws it into which target.

use super::*;

impl BindCx<'_> {
    /// An effect group: images `a` and `b`, the depth `d`, sized by
    /// `uniform`.
    fn fx_in(
        &self,
        uniform: &wgpu::Buffer,
        a: &wgpu::TextureView,
        d: &wgpu::TextureView,
        b: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.group(
            "fx post",
            &self.passes.fx_layout,
            &[
                uniform.as_entire_binding(),
                self.sampler(),
                tex(a),
                tex(d),
                tex(self.gbuffer()),
                tex(b),
                wgpu::BindingResource::Sampler(&self.passes.nearest_sampler),
                self.probes.as_entire_binding(),
            ],
        )
    }

    /// An effect group with `d` in the depth's slot.
    pub fn fx_depth(
        &self,
        a: &wgpu::TextureView,
        d: &wgpu::TextureView,
        b: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        self.fx_in(&self.passes.fx_buffer, a, d, b)
    }

    /// An effect group over the prepass's depth.
    pub fn fx(&self, a: &wgpu::TextureView, b: &wgpu::TextureView) -> wgpu::BindGroup {
        self.fx_depth(a, self.depth(), b)
    }

    /// An effect group after TAA: the frame it resolved (the output's
    /// size under TAAU), with its own uniform's screen.
    pub fn fx_after(&self, a: &wgpu::TextureView, b: &wgpu::TextureView) -> wgpu::BindGroup {
        self.fx_in(&self.passes.fx_after_buffer, a, self.depth(), b)
    }
}

/// An effect pass: its entry point and the target it draws into.
pub(super) type Draw = (&'static str, &'static str);

/// The effect passes' bind groups.
pub(super) struct Fx {
    groups: HashMap<FramePass, Pair>,
    zoom: Option<[Pair; 2]>,
}

impl Fx {
    pub fn bind(cx: &BindCx) -> Fx {
        let groups = cx
            .frame
            .iter()
            .filter_map(|&p| {
                let groups = ao::groups(cx, p)
                    .or_else(|| outline::groups(cx, p))
                    .or_else(|| screen::groups(cx, p))?;
                Some((p, groups))
            })
            .collect();
        let zoom = (cx.has(FramePass::ScreenBlur) && cx.cfg.zoom_blur).then(|| {
            [
                cx.pair(|_| cx.fx_after(cx.view("zoom.0"), &cx.passes.black)),
                cx.pair(|slot| cx.fx_after(cx.view("zoom.1"), cx.stage(cx.names.after_mb, slot))),
            ]
        });
        Fx { groups, zoom }
    }

    /// Records `pass` when it is an effect pass; its draws.
    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        cx: &PostCx,
        pass: FramePass,
    ) -> Option<u32> {
        // One logical/timed pass includes all three stages, so the
        // screen-blur GPU cost cannot hide auxiliary work.
        if pass == FramePass::ScreenBlur
            && let Some(zoom) = &self.zoom
        {
            for (entry, target, group) in [
                ("fs_zoom_fine", "zoom.0", &self.groups[&pass][cx.slot]),
                ("fs_zoom_middle", "zoom.1", &zoom[0][cx.slot]),
                ("fs_zoom_coarse", "hdr.blur", &zoom[1][cx.slot]),
            ] {
                fullscreen_pass(
                    encoder,
                    entry,
                    cx.graph.view(target),
                    CLEAR,
                    &cx.passes.fx[entry],
                    group,
                );
            }
            return Some(3);
        }
        let (entry, target) = ao::draw(cx, pass)
            .or_else(|| outline::draw(pass))
            .or_else(|| screen::draw(pass))?;
        fullscreen_pass(
            encoder,
            entry,
            cx.graph.view(target),
            CLEAR,
            &cx.passes.fx[entry],
            &self.groups[&pass][cx.slot],
        );
        Some(1)
    }
}
