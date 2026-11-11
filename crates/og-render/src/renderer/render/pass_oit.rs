//! The main camera's order-independent transparency (oit.rs): which of
//! its blended surfaces join the OIT pass, their pipelines, and the pass
//! and resolve around them.

use super::*;
use crate::renderer::batches::{Made, variant_job, variants};
use oit::{Oit, Source};

/// A part of the main camera's transparent surfaces, in the order the
/// main pass draws them with OIT: the transmissive meshes (opaque
/// pipelines over the scene copy, depth written) before the OIT pass, the
/// blended surfaces in it, the billboards it cannot take after its
/// resolve, back to front.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BlendedPart {
    Transmissive,
    Oit(Oit),
    Rest,
}

impl Renderer {
    /// An OIT pipeline made by [`Renderer::ensure_oit_pipelines`].
    pub(super) fn oit_pipeline(&self, source: Source, mode: Oit) -> Option<&wgpu::RenderPipeline> {
        self.oit
            .pipelines
            .get(&(source, mode, self.samples))
            .map(|p| p.get(self.frame_features))
    }

    /// The OIT pipeline source of a blended batch, as `lit_pipeline_of`
    /// picks its sorted pipeline.
    pub(super) fn oit_lit_source(b: &Batch, skinned: bool, physical: bool) -> Source {
        use oit::LitModule;
        let module = match (physical, b.shader) {
            (true, None) => LitModule::Physical,
            (true, Some(sh)) => LitModule::PhysicalShader(sh),
            (false, _) if b.array => LitModule::Array,
            (false, Some(sh)) => LitModule::Shader(sh),
            (false, None) => LitModule::Standard,
        };
        Source::Lit(module, b.side, skinned, b.effects)
    }

    /// The OIT source of a billboard run (or its drop shadow): tone-mapped
    /// alpha sprites without a normal map, and text. Additive and
    /// untonemapped runs (which write the HDR alpha) and normal-mapped or
    /// array sprites stay sorted, drawn after the resolve.
    pub(super) fn oit_billboard(
        &self,
        run: &sprites::SpriteRun,
        scene: &RenderScene,
    ) -> Option<Source> {
        if !run.tone_mapped {
            return None;
        }
        match run.source {
            RunSource::Sprite(SpriteBlend::Alpha, t) if self.normal_group(scene, t).is_none() => {
                Some(Source::Sprite)
            }
            RunSource::Font(f) if self.cache.fonts[f.index()].layout.color => Some(Source::Sprite),
            RunSource::Font(_) => Some(Source::Msdf),
            _ => None,
        }
    }

    /// Makes the OIT pipelines of `mode` the frame's blended surfaces may
    /// need: its blended batches (and their lightmapped variants), the
    /// blended skinned ones, `billboards`' sources, alpha particles and
    /// strips. Made before culling, so for every blended batch.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn ensure_oit_pipelines(
        &mut self,
        mode: Oit,
        batches: &[Batch],
        skinned: &[Batch],
        billboards: &[Source],
        particles: bool,
        strips: (bool, bool),
    ) {
        let samples = self.samples;
        self.oit
            .resolver
            .ensure(&self.device, samples, mode == Oit::List);
        // The weighted mode's front layer is marked by a pass of its own.
        let modes: &[Oit] = if mode == Oit::Front {
            &[Oit::Front, Oit::Mark]
        } else {
            &[mode]
        };
        let mut wanted: Vec<Source> = Vec::new();
        for b in batches.iter().filter(|b| b.transparent && b.blend) {
            let physical = b.physical && self.physical.is_some();
            wanted.push(Self::oit_lit_source(b, false, physical));
            if let Some(f) = Self::lightmap_family(&self.lightmaps, b, physical) {
                wanted.push(Source::Lightmap(f, b.side, b.effects));
            }
        }
        for b in skinned.iter().filter(|b| b.transparent && b.blend) {
            let physical = b.physical && self.physical.is_some();
            wanted.push(Self::oit_lit_source(b, true, physical));
        }
        wanted.extend_from_slice(billboards);
        if particles {
            wanted.push(Source::Particles);
        }
        if strips.0 {
            wanted.push(Source::Ribbon);
        }
        if strips.1 && self.gpu_trails.is_some() {
            wanted.push(Source::Strip);
        }
        for (source, &mode) in wanted
            .iter()
            .flat_map(|&s| modes.iter().map(move |m| (s, m)))
        {
            if self.oit.pipelines.contains_key(&(source, mode, samples)) {
                continue;
            }
            let frame = &self.frame_layout;
            let device = self.device.clone();
            // Each made by a job owning what it reads, on the compile
            // workers natively; the built-in lit shader's in the variant for
            // the frame's features and the full one (pipeline_features.rs).
            let none = crate::pipeline_features::Features::NONE;
            let (declared, job) = match source {
                Source::Lit(module, side, skinned, effects) => {
                    use oit::LitModule;
                    let physical =
                        matches!(module, LitModule::Physical | LitModule::PhysicalShader(_));
                    let (name, groups, shader, declared): (
                        &'static str,
                        [&wgpu::BindGroupLayout; 2],
                        &wgpu::ShaderModule,
                        _,
                    ) = match module {
                        LitModule::Standard => (
                            "pbr (oit)",
                            [&self.material_layout, &self.fx_layout],
                            self.pbr_shader.get(),
                            self.pbr_shader.declared,
                        ),
                        LitModule::Shader(sh) => (
                            "pbr (oit)",
                            [&self.material_layout, &self.fx_layout],
                            &self.cache.shaders[sh],
                            none,
                        ),
                        LitModule::Array => {
                            let a = self.array_maps.as_ref().expect("an array batch");
                            (
                                "pbr array maps (oit)",
                                [&a.layout, &self.fx_layout],
                                &a.module,
                                none,
                            )
                        }
                        LitModule::Physical | LitModule::PhysicalShader(_) => {
                            let p = self.physical.as_ref().expect("a physical batch");
                            let shader = match module {
                                LitModule::PhysicalShader(sh) => &self.cache.physical_shaders[&sh],
                                _ => &p.module,
                            };
                            (
                                "physical (oit)",
                                [&p.material_layout, &p.fx_layout],
                                shader,
                                none,
                            )
                        }
                    };
                    let (name, rest): (&'static str, Vec<&wgpu::BindGroupLayout>) = if skinned {
                        let name = match name {
                            "pbr (oit)" => "pbr skinned (oit)",
                            "pbr array maps (oit)" => "pbr array maps skinned (oit)",
                            _ => "physical skinned (oit)",
                        };
                        (name, vec![groups[0], groups[1], &self.palette_layout])
                    } else {
                        (name, groups.to_vec())
                    };
                    let layout = self.oit.layout(&device, name, mode, frame, &rest).clone();
                    let shader = shader.clone();
                    const PLAIN: &[Option<wgpu::VertexBufferLayout<'static>>] =
                        &[Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
                    const SKIN: &[Option<wgpu::VertexBufferLayout<'static>>] = &[
                        Some(VERTEX_LAYOUT),
                        Some(skin::SKINNED_INSTANCE_LAYOUT),
                        Some(skin::SKIN_VERTEX_LAYOUT),
                    ];
                    let job = variant_job(move |off| {
                        let mut constants = vec![("EFFECTS", effects as u32 as f64)];
                        constants.extend_from_slice(off);
                        Made::Pipeline(oit::pipeline(
                            &device,
                            &constants,
                            HdrPipeline {
                                alpha_to_coverage: false,
                                label: name,
                                layout: &layout,
                                module: &shader,
                                vs: match (physical, skinned) {
                                    (false, false) => "vs",
                                    (false, true) => "vs_skinned",
                                    (true, false) => "vs_physical",
                                    (true, true) => "vs_physical_skinned",
                                },
                                fs: "fs",
                                buffers: if skinned { SKIN } else { PLAIN },
                                cull: cull_mode(side),
                                depth_write: false,
                                depth_compare: wgpu::CompareFunction::Less,
                                blend: None,
                                samples,
                            },
                            mode,
                        ))
                    });
                    (declared, job)
                }
                Source::Lightmap(family, side, effects) => {
                    use lightmap::Family;
                    let lightmap_fx = self.fx_lean_layout.as_ref().unwrap_or(&self.fx_layout);
                    let (name, groups, shader, declared): (
                        &'static str,
                        [&wgpu::BindGroupLayout; 2],
                        &wgpu::ShaderModule,
                        _,
                    ) = match family {
                        Family::Standard | Family::Compact => (
                            "lightmapped (oit)",
                            [&self.material_layout, lightmap_fx],
                            self.pbr_shader.get(),
                            self.pbr_shader.declared,
                        ),
                        Family::Shader(sh) => (
                            "lightmapped (oit)",
                            [&self.material_layout, lightmap_fx],
                            &self.cache.shaders[sh],
                            none,
                        ),
                        Family::Array => {
                            let a = self.array_maps.as_ref().expect("an array batch");
                            (
                                "lightmapped array maps (oit)",
                                [&a.layout, lightmap_fx],
                                &a.module,
                                none,
                            )
                        }
                        Family::Physical | Family::PhysicalShader(_) => {
                            let p = self.physical.as_ref().expect("a physical batch");
                            let shader = match family {
                                Family::PhysicalShader(sh) => &self.cache.physical_shaders[&sh],
                                _ => &p.module,
                            };
                            (
                                "lightmapped physical (oit)",
                                [&p.material_layout, &p.fx_layout],
                                shader,
                                none,
                            )
                        }
                    };
                    let pages = self.lightmaps.pages_layout();
                    let layout = self
                        .oit
                        .layout(&device, name, mode, frame, &[groups[0], groups[1], pages])
                        .clone();
                    let shader = shader.clone();
                    let job = variant_job(move |off| {
                        Made::Pipeline(lightmap::Lightmaps::oit_pipeline(
                            &device,
                            family,
                            &layout,
                            &shader,
                            (side, effects),
                            samples,
                            mode,
                            off,
                        ))
                    });
                    (declared, job)
                }
                Source::Sprite | Source::Msdf => {
                    let layout = self
                        .oit
                        .layout(
                            &device,
                            "sprite (oit)",
                            mode,
                            frame,
                            &[&self.sprite_layout, &self.sprite_fx_layout],
                        )
                        .clone();
                    let shader = self.sprite_shader.get().clone();
                    let job = variant_job(move |_| {
                        Made::Pipeline(oit::pipeline(
                            &device,
                            &[],
                            HdrPipeline {
                                alpha_to_coverage: false,
                                label: "billboard",
                                layout: &layout,
                                module: &shader,
                                vs: "vs",
                                fs: if source == Source::Msdf {
                                    "fs_msdf"
                                } else {
                                    "fs"
                                },
                                buffers: SPRITE_BUFFERS,
                                cull: None,
                                depth_write: false,
                                depth_compare: wgpu::CompareFunction::LessEqual,
                                blend: None,
                                samples,
                            },
                            mode,
                        ))
                    });
                    (none, job)
                }
                Source::Ribbon | Source::Strip => {
                    let strip = source == Source::Strip;
                    let mut rest = vec![&self.sprite_layout];
                    if strip {
                        rest.push(&self.gpu_trails.as_ref().expect("checked above").draw_layout);
                    } else {
                        rest.push(&self.particles.gpu.effects.empty_layout);
                    }
                    rest.push(&self.particles.gpu.effects.layout);
                    let name = if strip { "strip (oit)" } else { "ribbon (oit)" };
                    let layout = self.oit.layout(&device, name, mode, frame, &rest).clone();
                    let shader = self.ribbon_shader.get().clone();
                    let job = variant_job(move |_| {
                        Made::Pipeline(oit::pipeline(
                            &device,
                            &[],
                            HdrPipeline {
                                alpha_to_coverage: false,
                                label: name,
                                layout: &layout,
                                module: &shader,
                                vs: if strip { "vs_strip" } else { "vs" },
                                fs: "fs",
                                buffers: if strip { &[] } else { RIBBON_BUFFERS },
                                cull: None,
                                depth_write: false,
                                depth_compare: wgpu::CompareFunction::LessEqual,
                                blend: None,
                                samples,
                            },
                            mode,
                        ))
                    });
                    (none, job)
                }
                Source::Particles => {
                    let layout = self
                        .oit
                        .layout(
                            &device,
                            "particles (oit)",
                            mode,
                            frame,
                            &[
                                &self.sprite_layout,
                                &self.particles.gpu.draw_layout,
                                &self.particles.gpu.effects.layout,
                            ],
                        )
                        .clone();
                    let shader = self.particles.gpu.draw_module.get().clone();
                    const PARTICLES: &[Option<wgpu::VertexBufferLayout<'static>>] =
                        &[Some(particles_gpu::PARTICLE_LAYOUT)];
                    let job = variant_job(move |_| {
                        Made::Pipeline(oit::pipeline(
                            &device,
                            &[],
                            HdrPipeline {
                                alpha_to_coverage: false,
                                label: "particles",
                                layout: &layout,
                                module: &shader,
                                vs: "vs",
                                fs: "fs",
                                buffers: PARTICLES,
                                cull: None,
                                depth_write: false,
                                depth_compare: wgpu::CompareFunction::LessEqual,
                                blend: None,
                                samples,
                            },
                            mode,
                        ))
                    });
                    (none, job)
                }
            };
            let pipeline = variants(declared, self.frame_features, job);
            self.oit.pipelines.insert((source, mode, samples), pipeline);
        }
    }

    /// The billboard runs (drop shadows, then billboards) that join the
    /// OIT pass, with their OIT pipelines; group 2 is left as `sprite_fx`.
    /// Returns the draws.
    pub(super) fn frame_draw_oit_billboards(
        &self,
        pass: &mut wgpu::RenderPass,
        runs: &[sprites::SpriteRun],
        sprite_fx: &wgpu::BindGroup,
        mode: Oit,
        scene: &RenderScene,
    ) -> u32 {
        let mut draws = 0;
        let mut current = None;
        for run in runs {
            let Some(source) = self.oit_billboard(run, scene) else {
                continue;
            };
            if draws == 0 {
                pass.set_bind_group(2, sprite_fx, &[]);
            }
            if current != Some(source) {
                let Some(p) = self.oit_pipeline(source, mode) else {
                    continue;
                };
                pass.set_pipeline(p);
                current = Some(source);
            }
            let group = match run.source {
                RunSource::Sprite(_, None) => &self.white_sprite_group,
                RunSource::Sprite(_, Some(t)) => self.cache.sprite_groups[t.index()]
                    .as_ref()
                    .expect("made before the pass"),
                RunSource::Font(f) => &self.cache.fonts[f.index()].bind_group,
                _ => unreachable!("oit_billboard takes sprites and text"),
            };
            pass.set_bind_group(1, group, &[]);
            let start = run.first as u64 * SPRITE_STRIDE;
            pass.set_vertex_buffer(
                0,
                self.sprite_buffer
                    .slice(start..start + run.count as u64 * SPRITE_STRIDE),
            );
            pass.draw(0..6, 0..run.count);
            draws += 1;
        }
        draws
    }

    /// The resolve of the OIT pass over the HDR target (per sample under
    /// MSAA, into `hdr-msaa`): its own pass, which reads the depth the
    /// linked lists test their nodes against.
    pub(super) fn encode_oit_resolve(&self, encoder: &mut crate::Encoder, mode: Oit) -> u32 {
        let list = mode == Oit::List;
        let Some((layout, pipeline)) = self.oit.resolver.get(self.samples, list) else {
            return 0;
        };
        let mut resources = vec![
            wgpu::BindingResource::TextureView(self.graph.view("oit.accum")),
            wgpu::BindingResource::TextureView(self.graph.view("oit.reveal")),
            wgpu::BindingResource::TextureView(self.graph.view("oit.front")),
        ];
        if list {
            let arena = self.oit.arena.as_ref().expect("a list frame's arena");
            resources.push(wgpu::BindingResource::TextureView(self.graph.view("depth")));
            resources.push(arena.buffer.as_entire_binding());
        }
        let group = bind_group(&self.device, "oit resolve", layout, &resources);
        let view = if self.samples > 1 {
            self.graph.view("hdr-msaa")
        } else {
            self.graph.view("hdr")
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("oit resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        1
    }
}
