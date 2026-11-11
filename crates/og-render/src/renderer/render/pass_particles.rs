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

    /// The 2D particles simulated in compute: room for their ranges,
    /// what each draws with, and the compute's parameters.
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &particles_gpu::ParticleFrame,
        sprites: &sprites::GpuSprites,
    ) {
        let particle_sprites = u64::from(sprites.total) * SPRITE_STRIDE;
        if particle_sprites > self.sprite_buffer.size() {
            self.sprite_buffer =
                buffers::particle_sprite_buffer(device, particle_sprites.next_power_of_two());
        }
        self.sprite_sources = sprites.ranges.iter().map(|r| r.source).collect();
        self.gpu.prepare_sprites(device, queue, frame, sprites);
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
    type Plan<'f> = ParticlesPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.particles
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
