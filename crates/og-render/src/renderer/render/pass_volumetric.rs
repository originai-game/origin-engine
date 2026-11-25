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
    /// The frame's medium, scene, previous view-projection, and perspective flag.
    type PlanIn<'a> = (
        Option<&'a crate::volumetric::Medium>,
        &'a RenderScene,
        Mat4,
        bool,
    );
    type UploadIn<'a> = ();
    type BindIn<'f> = ();
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.volumetric
    }

    /// The frame's medium (both methods: the main pass applies the
    /// raymarched one itself), and whether the froxel pass runs.
    fn plan(
        r: &mut Renderer,
        (medium, scene, prev_view_proj, perspective): Self::PlanIn<'_>,
    ) -> bool {
        let Some(medium) = medium else {
            return false;
        };
        let has_spots = r
            .lights
            .clusters
            .kept
            .iter()
            .any(|&i| matches!(scene.lighting.lights[i].kind, LightKind::Spot { .. }));
        r.volumetric.prepare(
            &r.device,
            &r.queue,
            &r.frame_layout,
            medium,
            (&scene.atmosphere.fog_volumes, scene.frame.camera.position),
            scene.atmosphere.fog.map_or(1.0, |f| f.sky),
            (r.width, r.height),
            r.samples,
            scene.frame.tick,
            &scene.frame.quality,
            prev_view_proj,
            perspective && !scene.frame.camera_cut,
            has_spots,
        );
        r.volumetric.method == VolumetricMethod::Froxel
    }

    fn bind<'f>(_: &'f Renderer, froxels: &'f mut bool, _: ()) -> Option<()> {
        froxels.then_some(())
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let ibl = r.lights.ibl.views();
        st.encode_froxels(
            &r.device,
            cx.encoder,
            &r.lights.frame_buffer,
            &r.shadows.cascade_map.array_view,
            &r.shadows.local_map.array_view,
            &r.lights.light_tables,
            &crate::volumetric::FroxelAmbient {
                irradiance: &ibl.irradiance,
                prefiltered: &ibl.prefiltered,
                sampler: &ibl.sampler,
                probes: &r.lights.lightmaps.probe_buffer,
            },
            (&r.lights.caustic_grid, &r.sky.cloud_shadow.view),
        );
    }
}
