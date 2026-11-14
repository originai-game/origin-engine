use super::*;

/// The pipelines in [`TerrainPasses`] over `layouts`, from these modules;
/// `holes`: with the hole test (terrain.wgsl `TERRAIN_HOLES`), whose
/// discard a terrain without holes leaves out (early depth and Apple's
/// hidden-surface removal keep working for it).
pub(super) fn terrain_passes(
    device: &wgpu::Device,
    layouts: TerrainPassLayouts,
    samples: u32,
    lit_source: String,
    shadow_source: String,
    corners_source: String,
    holes: bool,
) -> TerrainPasses {
    // Made when a terrain is first drawn (deferred.rs).
    let lit_module = crate::deferred::unbounded_module(device, "terrain", lit_source);
    let shadow_module = crate::deferred::unbounded_module(device, "terrain shadow", shadow_source);
    let corners_module = crate::deferred::module(device, "terrain corners", corners_source);
    let pl = |label, layouts: &[&wgpu::BindGroupLayout]| {
        let layouts: Vec<_> = layouts.iter().map(|l| Some(*l)).collect();
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        })
    };
    let lit_layout = pl(
        "terrain",
        &[layouts.frame, layouts.terrain, layouts.corners, layouts.fx],
    );
    let prepass_layout = pl(
        "terrain prepass",
        &[layouts.frame, layouts.terrain, layouts.corners],
    );
    let shadow_layout = pl(
        "terrain shadow",
        &[layouts.shadow, layouts.terrain, layouts.corners],
    );
    let corners_layout = pl("terrain corners", &[layouts.corners_write, layouts.terrain]);
    let stamp_lit_layout = pl(
        "terrain stamp",
        &[layouts.frame, layouts.terrain, layouts.stamp, layouts.fx],
    );
    let stamp_prepass_layout = pl(
        "terrain stamp prepass",
        &[layouts.frame, layouts.terrain, layouts.stamp],
    );
    let stamp_shadow_layout = pl(
        "terrain stamp shadow",
        &[layouts.shadow, layouts.terrain, layouts.stamp],
    );
    let constants: &'static [(&'static str, f64)] = if holes {
        &[("TERRAIN_HOLES", 1.0)]
    } else {
        &[("TERRAIN_HOLES", 0.0)]
    };
    // The lit module's constants for a variant with `features`.
    let declared = lit_module.declared;
    let lit_constants = move |features: Features| {
        let mut c = constants.to_vec();
        c.extend(features.constants(declared));
        c
    };
    const STAMP_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
        &[Some(crate::VERTEX_LAYOUT)];
    // A mirrored terrain's or stamp's triangles wind the other way on
    // screen: its pipelines take clockwise as the front (`Faces::front`).
    let front = |mirrored: bool| {
        crate::Faces {
            side: crate::Side::Front,
            mirrored,
        }
        .front()
    };
    let lit = |label: &'static str,
               layout: &wgpu::PipelineLayout,
               vs: &'static str,
               fs: &'static str,
               buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>],
               mirrored: bool| {
        let (device, layout, module) = (device.clone(), layout.clone(), lit_module.clone());
        Variants::new(declared, move |features| {
            let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
            let constants = lit_constants(features);
            crate::deferred::Sampled::anywhere(samples, move |samples| {
                crate::hdr_pipeline_facing(
                    &device,
                    &constants,
                    front(mirrored),
                    crate::HdrPipeline {
                        alpha_to_coverage: false,
                        label,
                        layout: &layout,
                        module: module.module(),
                        vs,
                        fs,
                        buffers,
                        cull: Some(wgpu::Face::Back),
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples,
                    },
                )
            })
        })
    };
    let options = move || wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    };
    let target = |format| {
        Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })
    };
    let depth = |write| {
        Some(wgpu::DepthStencilState {
            format: crate::DEPTH_FORMAT,
            depth_write_enabled: Some(write),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        })
    };
    let prepass = |label: &'static str,
                   layout: &wgpu::PipelineLayout,
                   vs: &'static str,
                   fs: &'static str,
                   buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>],
                   mirrored: bool| {
        let (device, layout, module) = (device.clone(), layout.clone(), lit_module.clone());
        Variants::new(declared, move |features| {
            let (device, layout, module) = (device.clone(), layout.clone(), module.clone());
            let constants = lit_constants(features);
            Deferred::anywhere(move || {
                let module = module.module();
                let options = || wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    ..Default::default()
                };
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some(vs),
                        compilation_options: options(),
                        buffers,
                    },
                    primitive: wgpu::PrimitiveState {
                        front_face: front(mirrored),
                        cull_mode: Some(wgpu::Face::Back),
                        ..Default::default()
                    },
                    depth_stencil: depth(true),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some(fs),
                        compilation_options: options(),
                        targets: &[
                            target(crate::post::GBUFFER_FORMAT),
                            target(crate::post::LINDEPTH_FORMAT),
                            target(crate::post::OUTLINE_FORMAT),
                        ],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        })
    };
    // An open surface: both faces cast (the bias in the lit pass keeps
    // its own lit side from shadowing itself); a stamp's back faces too.
    let shadow = |label: &'static str,
                  layout: &wgpu::PipelineLayout,
                  vs: &'static str,
                  fs: &'static str,
                  buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>]| {
        let (device, layout, module) = (device.clone(), layout.clone(), shadow_module.clone());
        // Both faces cast into a local layer's two channels too
        // (shadow_target.rs).
        let fragment_constants: &'static [(&'static str, f64)] = if holes {
            &[("TERRAIN_HOLES", 1.0), ("SHADOW_SIDE", 2.0)]
        } else {
            &[("TERRAIN_HOLES", 0.0), ("SHADOW_SIDE", 2.0)]
        };
        Deferred::anywhere(move || {
            let module = module.module();
            crate::ShadowSet::new(|target| {
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some(vs),
                        compilation_options: options(),
                        buffers,
                    },
                    primitive: wgpu::PrimitiveState {
                        cull_mode: target.cull(crate::Side::Double),
                        ..Default::default()
                    },
                    depth_stencil: Some(target.depth()),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some(fs),
                        compilation_options: wgpu::PipelineCompilationOptions {
                            constants: fragment_constants,
                            ..Default::default()
                        },
                        targets: &[Some(target.color())],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        })
    };
    TerrainPasses {
        lit: [false, true].map(|m| Rc::new(lit("terrain", &lit_layout, "vs", "fs", &[], m))),
        prepass: [false, true].map(|m| {
            Rc::new(prepass(
                "terrain prepass",
                &prepass_layout,
                "vs",
                "fs_prepass",
                &[],
                m,
            ))
        }),
        shadow: Rc::new(shadow(
            "terrain shadow",
            &shadow_layout,
            "vs_shadow",
            "fs_shadow",
            &[],
        )),
        stamp_lit: [false, true].map(|m| {
            Rc::new(lit(
                "terrain stamp",
                &stamp_lit_layout,
                "vs_stamp",
                "fs_stamp",
                STAMP_BUFFERS,
                m,
            ))
        }),
        stamp_prepass: [false, true].map(|m| {
            Rc::new(prepass(
                "terrain stamp prepass",
                &stamp_prepass_layout,
                "vs_stamp",
                "fs_stamp_prepass",
                STAMP_BUFFERS,
                m,
            ))
        }),
        stamp_shadow: Rc::new(shadow(
            "terrain stamp shadow",
            &stamp_shadow_layout,
            "vs_stamp_shadow",
            "fs_stamp_shadow",
            STAMP_BUFFERS,
        )),
        corners: Rc::new({
            let device = device.clone();
            Deferred::anywhere(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("terrain corners"),
                    layout: Some(&corners_layout),
                    module: corners_module.module(),
                    entry_point: Some("write_corners"),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        }),
    }
}
