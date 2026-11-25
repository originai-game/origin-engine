use super::*;

impl ScatterRenderer {
    pub(super) fn slots(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        wind: &wgpu::Buffer,
        [wind_field, trample]: [&wgpu::TextureView; 2],
        n: u64,
    ) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("scatter"),
                size: SLOT * n,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wind.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(wind_field),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(trample),
                },
            ],
        });
        (buffer, group)
    }

    /// The main camera's scatter group over the prepass's view depth
    /// `view` (renderer.rs `card_depth`), for [`Self::draw`]'s cards.
    pub(crate) fn card_depth_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> Option<wgpu::BindGroup> {
        let (layout, _) = self.card_depth.as_ref()?;
        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scatter (prepass depth)"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.uniform,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<ScatterGpu>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.wind_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.wind_field),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.trample),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        }))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn pipeline(
        &mut self,
        device: &wgpu::Device,
        shader: Option<usize>,
        side: Side,
        pass: ScatterPass,
        impostor: bool,
        foliage: bool,
        opaque_shadow: bool,
    ) {
        let foliage = foliage && !impostor;
        let opaque_shadow = opaque_shadow && !impostor && pass == ScatterPass::Shadow;
        let kind = pass as u8
            + if impostor { IMPOSTOR_KIND } else { 0 }
            + if foliage { FOLIAGE_KIND } else { 0 }
            + if opaque_shadow { OPAQUE_SHADOW_KIND } else { 0 };
        // The shadow pass reads no material shader (copies cast as cut by the material's alpha).
        let shader = shader.filter(|_| pass != ScatterPass::Shadow);
        let key = if impostor {
            (None, Side::Double, kind)
        } else {
            (shader, side, kind)
        };
        if self.pipelines.contains_key(&key) || self.shadow_pipelines.contains_key(&key) {
            return;
        }
        if pass == ScatterPass::Shadow {
            let set = if impostor {
                self.card_shadows(device, false)
            } else {
                Self::mesh_shadow_pipeline(
                    device,
                    &self.shadow_layout,
                    &self.shadow_module,
                    side,
                    foliage,
                    opaque_shadow,
                )
            };
            self.shadow_pipelines.insert(key, set);
            return;
        }
        let (module, declared) = match shader {
            Some(sh) => self.hooked[&sh].clone(),
            None => (self.lit_module.clone(), self.declared),
        };
        let (vs, fs, buffers, cull) = match (impostor, foliage) {
            (true, _) => ("vs_impostor", "fs_impostor", IMPOSTOR_BUFFERS, None),
            (false, false) => ("vs", "fs", MESH_BUFFERS, crate::cull_mode(side)),
            (false, true) => ("vs_foliage", "fs", FOLIAGE_BUFFERS, crate::cull_mode(side)),
        };
        let d = LitDesc {
            label: match (impostor, pass) {
                (true, _) => "impostor",
                (false, ScatterPass::Prepass) => "scatter prepass",
                (false, _) => "scatter",
            },
            module,
            declared,
            vs,
            fs,
            buffers,
            cull,
        };
        // Over the prepass's own depth (single-sampled), the lit pass's
        // twin ([`EQUAL_KIND`]): a copy tests it for equality; a card, drawn
        // at its sphere's front, for LessEqual, and takes the surface the
        // prepass found (impostor.wgsl `fs_impostor_kept`).
        let twin = match (pass, impostor, &self.card_depth) {
            _ if pass != ScatterPass::Lit || self.samples != 1 => None,
            (_, false, _) => Some((
                LitDesc {
                    fs: "fs_equal",
                    ..d.clone()
                },
                &self.lit_fx_layout,
                wgpu::CompareFunction::Equal,
            )),
            (_, true, Some((_, layout))) => Some((
                LitDesc {
                    vs: "vs_impostor_front",
                    fs: "fs_impostor_kept",
                    ..d.clone()
                },
                layout,
                wgpu::CompareFunction::GreaterEqual,
            )),
            (_, true, None) => None,
        };
        if let Some((d, layout, over)) = twin {
            let equal = self.lit_variants(device, d, pass, layout.clone(), Some(over));
            self.pipelines
                .insert((key.0, key.1, key.2 + EQUAL_KIND), equal);
        }
        let p = self.lit_variants(device, d, pass, self.lit_fx_layout.clone(), None);
        self.pipelines.insert(key, p);
    }

    /// The lit or prepass `pass`'s pipeline of `d`, as the variant for the
    /// frame's features and the full one (pipeline_features.rs): the
    /// prepass's fragment entry is `d.fs` with `_prepass` (`fs_prepass` for
    /// `fs`); with `over`, the lit pass over the prepass's depth, tested
    /// by it ([`EQUAL_KIND`]).
    pub(super) fn lit_variants(
        &self,
        device: &wgpu::Device,
        d: LitDesc,
        pass: ScatterPass,
        layout: wgpu::PipelineLayout,
        over: Option<wgpu::CompareFunction>,
    ) -> LitVariants {
        let (device, samples) = (device.clone(), self.samples);
        let declared = d.declared;
        Variants::new(declared, move |features: Features| {
            let (device, layout, d) = (device.clone(), layout.clone(), d.clone());
            let constants = features.constants(declared);
            Deferred::anywhere(move || {
                lit_pipeline(&device, &layout, &d, (pass, over), samples, &constants)
            })
        })
    }

    /// A scattered mesh's shadow pipeline (shadow_target.rs).
    fn mesh_shadow_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        module: &wgpu::ShaderModule,
        side: Side,
        foliage: bool,
        opaque: bool,
    ) -> wgpu::RenderPipeline {
        let (buffers, vs) = match foliage {
            true => (FOLIAGE_BUFFERS, "vs_shadow_foliage"),
            false => (MESH_BUFFERS, "vs_shadow"),
        };
        let fs = if opaque {
            "fs_shadow_opaque"
        } else {
            "fs_shadow"
        };
        crate::traffic::create_render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some(if opaque {
                    "scatter shadow (opaque)"
                } else {
                    "scatter shadow"
                }),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some(vs),
                    compilation_options: Default::default(),
                    buffers,
                },
                primitive: wgpu::PrimitiveState {
                    cull_mode: crate::ShadowTarget::cull(side),
                    ..Default::default()
                },
                depth_stencil: Some(crate::ShadowTarget::depth()),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some(fs),
                    compilation_options: Default::default(),
                    targets: &[Some(crate::ShadowTarget::color())],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            },
        )
    }

    /// An impostor's (`impostor_shadow.wgsl`) or a crowd card's
    /// (`crowd_card_shadow.wgsl`) shadow pipeline: the card has no vertex
    /// buffer but the instances' (slot 0), both faces cast (shadow_target.rs).
    pub(super) fn card_shadows(&self, device: &wgpu::Device, card: bool) -> wgpu::RenderPipeline {
        let (label, entry, buffers) = match card {
            true => ("crowd card", "crowd_card", CARD_BUFFERS),
            false => ("impostor", "impostor", IMPOSTOR_BUFFERS),
        };
        card_shadow(
            device,
            (&self.shadow_layout, &self.shadow_module),
            label,
            entry,
            buffers,
        )
    }
}

fn card_shadow(
    device: &wgpu::Device,
    (layout, module): (&wgpu::PipelineLayout, &wgpu::ShaderModule),
    label: &'static str,
    entry: &str,
    buffers: &[Option<wgpu::VertexBufferLayout<'static>>],
) -> wgpu::RenderPipeline {
    let (vs, fs) = (format!("vs_{entry}_shadow"), format!("fs_{entry}_shadow"));
    crate::traffic::create_render_pipeline(
        device,
        &wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some(&vs),
                compilation_options: Default::default(),
                buffers,
            },
            primitive: Default::default(),
            depth_stencil: Some(crate::ShadowTarget::depth()),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some(&fs),
                compilation_options: Default::default(),
                targets: &[Some(crate::ShadowTarget::color())],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        },
    )
}

/// The scattered meshes', impostors' and crowd cards' vertex buffers.
const MESH_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
/// A foliage mesh's: its foliage stream third ([`FOLIAGE_KIND`]).
const FOLIAGE_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[
    Some(crate::VERTEX_LAYOUT),
    Some(crate::INSTANCE_LAYOUT),
    Some(crate::FOLIAGE_LAYOUT),
];
const IMPOSTOR_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(crate::INSTANCE_LAYOUT)];
pub(super) const CARD_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(CARD_LAYOUT)];

/// A lit or prepass pipeline of the scatter's lit module (or a material
/// shader's): its entries, vertex buffers and culled face.
#[derive(Clone)]
pub(super) struct LitDesc {
    pub label: &'static str,
    pub module: wgpu::ShaderModule,
    /// The features the module declares (pipeline_features.rs).
    pub declared: Features,
    pub vs: &'static str,
    pub fs: &'static str,
    pub buffers: &'static [Option<wgpu::VertexBufferLayout<'static>>],
    pub cull: Option<wgpu::Face>,
}

/// `d`'s `pass` pipeline with `constants` (the lit pass's into the HDR
/// target at `samples`, the prepass's into the G-buffer); `over`: the lit
/// pass's over the prepass's depth, which it tests with that function and
/// leaves as it is ([`EQUAL_KIND`]).
fn lit_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    d: &LitDesc,
    (pass, over): (ScatterPass, Option<wgpu::CompareFunction>),
    samples: u32,
    constants: &[(&str, f64)],
) -> wgpu::RenderPipeline {
    if pass == ScatterPass::Lit {
        return crate::hdr_pipeline_with(
            device,
            constants,
            crate::HdrPipeline {
                // opaque copies write alpha 1 (every sample); alpha-to-coverage materials their coverage;
                // over the prepass's depth, alpha 1 where the prepass kept the copy
                alpha_to_coverage: over.is_none(),
                label: d.label,
                layout,
                module: &d.module,
                vs: d.vs,
                fs: d.fs,
                buffers: d.buffers,
                cull: d.cull,
                depth_write: over.is_none(),
                depth_compare: over.unwrap_or(crate::LIT_DEPTH_COMPARE),
                blend: None,
                samples,
            },
        );
    }
    let target = |format| {
        Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })
    };
    let options = || wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    };
    let fs = format!("{}_prepass", d.fs);
    crate::traffic::create_render_pipeline(
        device,
        &wgpu::RenderPipelineDescriptor {
            label: Some(d.label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &d.module,
                entry_point: Some(d.vs),
                compilation_options: options(),
                buffers: d.buffers,
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: d.cull,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &d.module,
                entry_point: Some(&fs),
                compilation_options: options(),
                targets: &[
                    target(crate::post::GBUFFER_FORMAT),
                    target(crate::post::LINDEPTH_FORMAT),
                    target(crate::post::OUTLINE_FORMAT),
                    target(crate::post::MOTION_DEPTH_FORMAT),
                ],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        },
    )
}
