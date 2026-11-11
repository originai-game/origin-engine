//! The volumetric fog's froxel pass (volumetric.rs): the fog's in-scattered
//! light, in compute, into the frustum's grid the main pass applies.

use super::*;

pub(super) struct VolumetricPass;

impl Pass for VolumetricPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Volumetric,
        name: "volumetric",
        needs: Caps::COMPUTE,
    };
    type State = crate::volumetric::Volumetric;
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.volumetric
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        st.encode_froxels(
            &r.device,
            cx.encoder,
            &r.frame_buffer,
            &r.cascade_map.array_view,
            &r.local_map.array_view,
            [
                &r.light_buffers[0],
                &r.light_buffers[1],
                &r.light_buffers[2],
            ],
        );
    }
}

impl VolumetricPass {
    /// The frame's fog (both methods: the main pass applies the raymarched
    /// one itself), and the froxel pass's plan when it runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn plan(
        st: &mut <Self as Pass>::State,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame_layout: &wgpu::BindGroupLayout,
        fog: Option<&crate::volumetric::VolumetricFog>,
        size: (u32, u32),
        scene: &RenderScene,
    ) -> Option<()> {
        let fog = fog?;
        st.prepare(
            device,
            queue,
            frame_layout,
            fog,
            size,
            scene.tick,
            &scene.quality,
        );
        (st.method == VolumetricMethod::Froxel).then_some(())
    }
}
