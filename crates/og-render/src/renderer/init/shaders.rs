//! The renderer's shader modules (parsed when the first pipeline that uses
//! them is made, without the features it is made without) and the pipeline
//! layouts the lit, sky, sprite, strip and shadow pipelines share, with the
//! lightmaps and the effect materials whose groups they bind.

use super::*;

pub(super) struct ShadersAndLayoutsInputs<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub info: &'a RenderInfo,
    pub planar: bool,
    pub frame_layout: &'a wgpu::BindGroupLayout,
    pub material_layout: &'a wgpu::BindGroupLayout,
    pub fx_layout: &'a wgpu::BindGroupLayout,
    pub palette_layout: &'a wgpu::BindGroupLayout,
    pub sky_layout: &'a wgpu::BindGroupLayout,
    pub sprite_layout: &'a wgpu::BindGroupLayout,
    pub sprite_fx_layout: &'a wgpu::BindGroupLayout,
    pub shadow_bind_layout: &'a wgpu::BindGroupLayout,
}

pub(super) struct ShadersAndLayoutsOutputs {
    pub off: pipeline_features::Features,
    pub pbr_shader: deferred::Module,
    pub sky_shader: deferred::Module,
    pub sprite_shader: deferred::Module,
    pub ribbon_shader: deferred::Module,
    pub shadow_shader: deferred::Module,
    pub pbr_layout: wgpu::PipelineLayout,
    pub lightmaps: lightmap::Lightmaps,
    pub pbr_skinned_layout: wgpu::PipelineLayout,
    pub sky_pl: wgpu::PipelineLayout,
    pub sprite_pl: wgpu::PipelineLayout,
    pub effects: crate::effect::EffectRenderer,
    pub ribbon_pl: wgpu::PipelineLayout,
    pub shadow_layouts: [wgpu::PipelineLayout; 3],
}

impl Renderer {
    /// Makes the shader modules and the shared pipeline layouts.
    #[allow(clippy::too_many_lines)]
    pub(super) fn shaders_and_layouts(
        ShadersAndLayoutsInputs {
            device,
            queue,
            info,
            planar,
            frame_layout,
            material_layout,
            fx_layout,
            palette_layout,
            sky_layout,
            sprite_layout,
            sprite_fx_layout,
            shadow_bind_layout,
        }: ShadersAndLayoutsInputs<'_>,
    ) -> ShadersAndLayoutsOutputs {
        // Pipelines.
        // Parsed when the first pipeline that uses them is made
        // (deferred.rs): a frame compiles only what it draws.
        // The features the renderer is made without are in none of the
        // modules that carry their code (pipeline_features.rs).
        let off = info.without.features();
        let pbr_shader = deferred::module(
            device,
            "pbr",
            off.leave_out(lit_source_with(&hooks::default_hooks(), planar)),
        );
        let sky_shader = deferred::module(device, "sky", off.leave_out(sky_source()));
        let sprite_shader = deferred::module(device, "sprite", off.leave_out(sprite_source()));
        let ribbon_shader = deferred::module(device, "ribbon", off.leave_out(ribbon_source()));
        let shadow_shader =
            deferred::module(device, "shadow", shadow_source(&hooks::default_hooks()));
        let pipeline_layout = |label: &str, layouts: &[&wgpu::BindGroupLayout]| {
            let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &layouts,
                immediate_size: 0,
            })
        };
        let pbr_layout = pipeline_layout("pbr", &[frame_layout, material_layout, fx_layout]);
        let lightmaps = lightmap::Lightmaps::new(device, queue, frame_layout);
        let pbr_skinned_layout = pipeline_layout(
            "pbr skinned",
            &[frame_layout, material_layout, fx_layout, palette_layout],
        );
        let sky_pl = pipeline_layout("sky", &[frame_layout, sky_layout]);
        let sprite_pl = pipeline_layout("sprite", &[frame_layout, sprite_layout, sprite_fx_layout]);
        // Effect materials (effect.rs): group 3 of the particle and strip draws.
        let effects = crate::effect::EffectRenderer::new(device, queue);
        let ribbon_pl = pipeline_layout(
            "ribbon",
            &[
                frame_layout,
                sprite_layout,
                &effects.empty_layout,
                &effects.layout,
            ],
        );
        let shadow_layouts = [
            pipeline_layout("shadow", &[shadow_bind_layout]),
            pipeline_layout("shadow dissolve", &[shadow_bind_layout, material_layout]),
            pipeline_layout(
                "shadow skinned",
                &[shadow_bind_layout, material_layout, palette_layout],
            ),
        ];
        ShadersAndLayoutsOutputs {
            off,
            pbr_shader,
            sky_shader,
            sprite_shader,
            ribbon_shader,
            shadow_shader,
            pbr_layout,
            lightmaps,
            pbr_skinned_layout,
            sky_pl,
            sprite_pl,
            effects,
            ribbon_pl,
            shadow_layouts,
        }
    }
}
