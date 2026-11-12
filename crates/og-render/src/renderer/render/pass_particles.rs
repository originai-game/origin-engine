//! The particles' compute pass (particles_gpu.rs): the emitters simulated
//! on the GPU step against the scene's depth and normals, and the
//! sprite-mode systems expand into sprite instances the 2D runs draw.

use super::*;

/// What the particles keep between frames.
pub(crate) struct Particles {
    pub gpu: particles_gpu::ParticleRenderer,
    /// The instances of the sprite-mode particles simulated in compute
    /// (`particle_sprites.wgsl` writes them; `RunSource::Particles` runs
    /// draw them), and what each range of this frame draws with.
    pub sprite_buffer: wgpu::Buffer,
    pub sprite_sources: Vec<RunSource>,
    /// Where particles simulate ([`RenderDesc::particles`]).
    pub simulation: ParticleSimulation,
}

impl Particles {
    pub(crate) fn new(
        device: &wgpu::Device,
        effects: crate::effect::EffectRenderer,
        frame_layout: &wgpu::BindGroupLayout,
        sprite_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        simulation: ParticleSimulation,
    ) -> Particles {
        Particles {
            gpu: particles_gpu::ParticleRenderer::new(
                device,
                effects,
                frame_layout,
                sprite_layout,
                frame_wgsl,
            ),
            sprite_buffer: buffers::particle_sprite_buffer(device, SPRITE_STRIDE * 64),
            sprite_sources: Vec::new(),
            simulation,
        }
    }
}

pub(super) struct ParticlesPass;

/// A frame's particles simulated in compute.
pub(super) struct ParticlesPlan<'f> {
    pub frame: &'f particles_gpu::ParticleFrame,
    pub sprites: &'f sprites::GpuSprites,
}

impl Pass for ParticlesPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Particles,
        name: "particles",
        needs: Caps::COMPUTE,
    };
    type State = Particles;
    type Frame = ();
    type PlanIn<'a> = ();
    /// The frame's particles and the 2D ones' sprite ranges.
    type UploadIn<'a> = ParticlesPlan<'a>;
    type BindIn<'f> = ParticlesPlan<'f>;
    type Plan<'f> = ParticlesPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.particles
    }

    /// The 2D particles simulated in compute: room for their ranges,
    /// what each draws with, and the compute's parameters.
    fn upload(r: &mut Renderer, _: &mut (), plan: ParticlesPlan<'_>) {
        let (st, device) = (&mut r.particles, &r.device);
        let particle_sprites = u64::from(plan.sprites.total) * SPRITE_STRIDE;
        if particle_sprites > st.sprite_buffer.size() {
            st.sprite_buffer =
                buffers::particle_sprite_buffer(device, particle_sprites.next_power_of_two());
        }
        st.sprite_sources = plan.sprites.ranges.iter().map(|r| r.source).collect();
        st.gpu
            .prepare_sprites(device, &r.queue, plan.frame, plan.sprites);
    }

    /// The pass runs when an emitter simulates on the GPU.
    fn bind<'f>(
        _: &'f Renderer,
        _: &'f mut (),
        plan: ParticlesPlan<'f>,
    ) -> Option<ParticlesPlan<'f>> {
        plan.frame.any_gpu.then_some(plan)
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &ParticlesPlan<'f>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let r = cx.ctx.r;
        st.gpu.simulate(
            &r.device,
            cx.encoder,
            plan.frame,
            cx.scene_depth,
            r.graph.try_view("gbuffer").unwrap_or(&r.post.white),
        );
        st.gpu.expand_sprites(
            &r.device,
            cx.encoder,
            plan.frame,
            plan.sprites,
            &st.sprite_buffer,
        );
    }
}
