//! The 3D world's simulations and fluids made with the renderer: the
//! visual surfaces, garments, bodies and fluids where the device can
//! simulate them, the water, the vegetation's wind field and trample map
//! (where compute runs), the metaballs and the 3D fluids.

use super::*;

pub(super) struct WorldSimsInputs<'a> {
    pub device: &'a wgpu::Device,
    pub flags: wgpu::DownlevelFlags,
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub off: pipeline_features::Features,
    pub samples: u32,
}

pub(super) struct WorldSimsOutputs {
    pub surfaces: Option<surface::SurfacePipelines>,
    pub garments: Option<garment_gpu::Pipelines>,
    pub visual: Option<visual_sim::VisualPipelines>,
    pub visual_fluids: Option<visual_fluid::VisualFluids>,
    pub water: water::WaterRenderer,
    pub vegetation_compute: bool,
    pub wind_field: wind_field::WindFieldSim,
    pub trample: trample::Trample,
    pub metaballs: metaballs::MetaballRenderer,
    pub fluid3d: fluid3d::Fluid3dRenderer,
}

impl Renderer {
    /// Makes the world's simulations, water and fluids.
    #[allow(clippy::too_many_lines)]
    pub(super) fn world_sims(
        WorldSimsInputs {
            device,
            flags,
            frame_layout,
            off,
            samples,
        }: WorldSimsInputs<'_>,
    ) -> WorldSimsOutputs {
        let surfaces = ribbons_gpu::GpuTrails::supported(flags)
            .then(|| surface::SurfacePipelines::new(device));
        let garments = surfaces
            .as_ref()
            .map(|_| crate::garment_gpu::Pipelines::new(device));
        let visual = surfaces
            .as_ref()
            .map(|_| crate::visual_sim::VisualPipelines::new(device));
        let visual_fluids = surfaces
            .as_ref()
            .map(|_| crate::visual_fluid::VisualFluids::new(device));
        let water = water::WaterRenderer::new(
            device,
            frame_layout,
            off,
            samples,
            flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
        );
        // The vegetation's wind field (G2), simulated where compute runs.
        let vegetation_compute = WORLD_3D && flags.contains(wgpu::DownlevelFlags::COMPUTE_SHADERS);
        let wind_field = crate::wind_field::WindFieldSim::new(device, vegetation_compute);
        // The grass trample map (G3), pressed where compute runs.
        let trample = crate::trample::Trample::new(device, vegetation_compute);
        let metaballs = metaballs::MetaballRenderer::new(
            device,
            frame_layout,
            &off.leave_out(FRAME_WGSL.to_string()),
            samples,
        );
        let fluid3d = fluid3d::Fluid3dRenderer::new(
            device,
            frame_layout,
            &off.leave_out(fluid_shared_source()),
            samples,
        );
        WorldSimsOutputs {
            surfaces,
            garments,
            visual,
            visual_fluids,
            water,
            vegetation_compute,
            wind_field,
            trample,
            metaballs,
            fluid3d,
        }
    }
}
