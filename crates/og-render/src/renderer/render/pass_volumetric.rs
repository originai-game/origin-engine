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
    /// Whether the froxel pass runs.
    type Frame = bool;
    /// The frame's medium and its scene.
    type PlanIn<'a> = (Option<&'a crate::volumetric::Medium>, &'a RenderScene);
    type UploadIn<'a> = ();
    type BindIn<'f> = ();
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.volumetric
    }

    /// The frame's medium (both methods: the main pass applies the
    /// raymarched one itself), and whether the froxel pass runs.
    fn plan(r: &mut Renderer, (medium, scene): Self::PlanIn<'_>) -> bool {
        let Some(medium) = medium else {
            return false;
        };
        r.volumetric.prepare(
            &r.device,
            &r.queue,
            &r.frame_layout,
            medium,
            (&scene.atmosphere.fog_volumes, scene.frame.camera.position),
            (r.width, r.height),
            scene.frame.tick,
            &scene.frame.quality,
        );
        r.volumetric.method == VolumetricMethod::Froxel
    }

    fn bind<'f>(_: &'f Renderer, froxels: &'f mut bool, _: ()) -> Option<()> {
        froxels.then_some(())
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        st.encode_froxels(
            &r.device,
            cx.encoder,
            &r.lights.frame_buffer,
            &r.shadows.cascade_map.array_view,
            &r.shadows.local_map.array_view,
            [
                &r.lights.light_buffers[0],
                &r.lights.light_buffers[1],
                &r.lights.light_buffers[2],
            ],
            &r.lights.caustic_grid,
        );
    }
}
