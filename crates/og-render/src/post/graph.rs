//! Render graph realization, bindings and post encoding.

use super::*;

/// The post passes' bind groups for one realization of the graph. Groups
/// in pairs are per TAA history slot.
struct Bindings {
    taa: Option<[wgpu::BindGroup; 2]>,
    smaa_resolve: Option<[wgpu::BindGroup; 2]>,
    taa_sharpen: Option<[wgpu::BindGroup; 2]>,
    /// The passes after the tonemap, each reading its LDR target.
    chain: HashMap<FramePass, wgpu::BindGroup>,
    meter: Option<[wgpu::BindGroup; 2]>,
    reduce: Vec<wgpu::BindGroup>,
    adapt: Option<[wgpu::BindGroup; 2]>,
    prefilter: Option<[wgpu::BindGroup; 2]>,
    down: Vec<wgpu::BindGroup>,
    up: Vec<wgpu::BindGroup>,
    post: [wgpu::BindGroup; 2],
    fx: HashMap<FramePass, [wgpu::BindGroup; 2]>,
    /// The lens flare's passes (`flare.rs`).
    flare: HashMap<FramePass, [wgpu::BindGroup; 2]>,
}

/// The frame's render graph: the compiled plan for the current pass list,
/// the transient textures realized from it (kept in a pool across frames
/// and reused by description) and the bind groups that read them.
#[derive(Default)]
pub(crate) struct FrameGraph {
    passes: Vec<FramePass>,
    cfg: Option<GraphConfig>,
    /// Identifies the grading LUT the bindings were made with.
    lut_key: Option<(u64, usize)>,
    /// Identifies the palette and canvas the stylise bindings were made with.
    stylise_key: u64,
    order: Vec<FramePass>,
    plan: Option<GraphPlan>,
    pool: Vec<(TargetDesc, wgpu::TextureView)>,
    views: HashMap<String, wgpu::TextureView>,
    bindings: Option<Bindings>,
    /// Bumped whenever the targets are realized anew (views change).
    pub generation: u64,
}

impl FrameGraph {
    /// GPU bytes of the targets realized for the current plan.
    pub fn target_bytes(&self) -> u64 {
        self.pool
            .iter()
            .map(|(_, v)| crate::renderer::view_bytes(v))
            .sum()
    }

    /// Forgets everything size-dependent (on resize).
    pub fn invalidate(&mut self) {
        let generation = self.generation + 1;
        *self = FrameGraph::default();
        self.generation = generation;
    }

    /// The plan the last [`FrameGraph::prepare`] compiled.
    pub fn plan(&self) -> Option<&GraphPlan> {
        self.plan.as_ref()
    }

    /// The passes in execution order.
    pub fn order(&self) -> &[FramePass] {
        &self.order
    }

    /// Physical transient textures of the current plan.
    pub fn transient_textures(&self) -> usize {
        self.plan.as_ref().map_or(0, |p| p.physical().len())
    }

    pub fn view(&self, name: &str) -> &wgpu::TextureView {
        self.views
            .get(name)
            .unwrap_or_else(|| panic!("render graph target '{name}' is not realized"))
    }

    /// What custom post pass `i` reads and writes (`post_custom.rs`): the
    /// lit image before it (the TAA history of `slot` when TAA wrote it
    /// last) and its own target.
    pub fn custom_post_views<'a>(
        &'a self,
        persistent: &'a Persistent,
        i: u32,
        slot: usize,
    ) -> (&'a wgpu::TextureView, &'a wgpu::TextureView) {
        let read = match i {
            0 => match lit_names(&self.passes).before_custom {
                Some(n) => self.view(n),
                None => &persistent.history.as_ref().expect("TAA history")[slot],
            },
            i => self.view(CUSTOM_POST_NAMES[i as usize - 1]),
        };
        (read, self.view(CUSTOM_POST_NAMES[i as usize]))
    }

    /// The view of `name` when the current graph has it.
    pub fn try_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.views.get(name)
    }

    /// Compiles the graph for `passes` and realizes its transient targets,
    /// unless the previous frame had the same passes, configuration and LUT.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        passes: Vec<FramePass>,
        cfg: &GraphConfig,
        post: &PostPasses,
        persistent: &Persistent,
        lut: (Option<(u64, usize)>, &wgpu::TextureView),
        stylise: (&wgpu::TextureView, &wgpu::TextureView, u64),
    ) -> Result<(), GraphError> {
        if self.plan.is_some()
            && passes == self.passes
            && self.cfg.as_ref() == Some(cfg)
            && self.lut_key == lut.0
            && self.stylise_key == stylise.2
        {
            return Ok(());
        }
        let plan = build_graph(&passes, cfg).compile()?;
        self.order = plan.order().iter().map(|&i| passes[i]).collect();
        // Physical slots to pool textures: the first unused entry with the
        // same description, else a new texture.
        let mut taken = vec![false; self.pool.len()];
        let mut slot_views = Vec::with_capacity(plan.physical().len());
        for desc in plan.physical() {
            let found = (0..self.pool.len()).find(|&i| !taken[i] && self.pool[i].0 == *desc);
            let index = match found {
                Some(i) => i,
                None => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("graph transient"),
                        size: wgpu::Extent3d {
                            width: desc.width,
                            height: desc.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: desc.samples,
                        dimension: wgpu::TextureDimension::D2,
                        format: desc.format,
                        usage: desc.usage,
                        view_formats: &[],
                    });
                    self.pool
                        .push((*desc, texture.create_view(&Default::default())));
                    taken.push(false);
                    self.pool.len() - 1
                }
            };
            taken[index] = true;
            slot_views.push(self.pool[index].1.clone());
        }
        self.views.clear();
        for (name, slot) in plan.transients() {
            self.views
                .insert(name.to_string(), slot_views[slot].clone());
        }
        self.bindings = Some(self.bind(
            device,
            post,
            persistent,
            &passes,
            cfg,
            lut.1,
            (stylise.0, stylise.1),
        ));
        self.generation += 1;
        self.passes = passes;
        self.cfg = Some(*cfg);
        self.lut_key = lut.0;
        self.stylise_key = stylise.2;
        self.plan = Some(plan);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn bind(
        &self,
        device: &wgpu::Device,
        passes: &PostPasses,
        persistent: &Persistent,
        frame: &[FramePass],
        cfg: &GraphConfig,
        lut: &wgpu::TextureView,
        stylise: (&wgpu::TextureView, &wgpu::TextureView),
    ) -> Bindings {
        fn tex(v: &wgpu::TextureView) -> wgpu::BindingResource<'_> {
            wgpu::BindingResource::TextureView(v)
        }
        let sampler = wgpu::BindingResource::Sampler(&passes.sampler);
        let group = |label: &str, layout: &wgpu::BindGroupLayout, res: &[wgpu::BindingResource]| {
            let entries: Vec<_> = res
                .iter()
                .enumerate()
                .map(|(i, r)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: r.clone(),
                })
                .collect();
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &entries,
            })
        };
        let has = |p: FramePass| frame.contains(&p);
        let names = lit_names(frame);
        let taa = has(FramePass::Taa);
        let history = persistent.history.as_ref().filter(|_| taa);
        // The lit image in frame slot `i` at each stage.
        let stage = |name: Option<&'static str>, slot: usize| -> &wgpu::TextureView {
            match name {
                Some(n) => self.view(n),
                None => &history.expect("TAA history")[slot],
            }
        };
        let source = |slot: usize| stage(names.last, slot);
        // The prepass's velocity and view depth TAA reprojects moving
        // objects with (placeholders without a prepass: `TaaUniform::still.y` 0).
        let taa_velocity = self.try_view("gbuffer").unwrap_or(&passes.black);
        let taa_depth = self.try_view("lindepth").unwrap_or(&passes.far_depth);
        // The OIT revealage TAA trusts its history less under (taa.wgsl);
        // white for the passes that do not weigh it.
        let reveal = self
            .try_view("oit.reveal")
            .filter(|_| taa)
            .unwrap_or(&passes.no_reactive);
        let temporal_group = |current: &wgpu::TextureView,
                              previous: &wgpu::TextureView,
                              reactive: &wgpu::TextureView| {
            group(
                "temporal resolve",
                &passes.taa_layout,
                &[
                    tex(current),
                    tex(previous),
                    tex(self.view("depth")),
                    sampler.clone(),
                    passes.taa_buffer.as_entire_binding(),
                    wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                    tex(taa_velocity),
                    tex(taa_depth),
                    tex(reactive),
                ],
            )
        };
        let taa_groups = history.map(|h| {
            [0usize, 1].map(|slot| temporal_group(self.view(names.main), &h[1 - slot], reveal))
        });
        let smaa_resolve = persistent.smaa_history.as_ref().map(|h| {
            [0usize, 1].map(|slot| temporal_group(&h[slot], &h[1 - slot], &passes.no_reactive))
        });
        // The sharpening reads what TAA wrote this frame (as both images).
        let taa_sharpen = history.filter(|_| has(FramePass::TaaSharpen)).map(|h| {
            [0usize, 1].map(|slot| {
                group(
                    "taa sharpen",
                    &passes.taa_layout,
                    &[
                        tex(&h[slot]),
                        tex(&h[slot]),
                        tex(self.view("depth")),
                        sampler.clone(),
                        passes.taa_buffer.as_entire_binding(),
                        wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                        tex(taa_velocity),
                        tex(taa_depth),
                        tex(&passes.no_reactive),
                    ],
                )
            })
        });
        let mut chain = HashMap::new();
        // The paint and lines passes read the prepass (a canvas, hatching
        // or halftone on surfaces, ink on depth edges) when the frame has
        // one; the other passes bind placeholders.
        let prepass_of = |p: FramePass| {
            let lines = matches!(p, FramePass::Paint | FramePass::Lines);
            (
                self.try_view("lindepth")
                    .filter(|_| lines)
                    .unwrap_or(&passes.far_depth),
                self.try_view("gbuffer")
                    .filter(|_| lines)
                    .unwrap_or(&passes.black),
                self.try_view("object-space")
                    .filter(|_| lines)
                    .unwrap_or(&passes.no_object),
            )
        };
        for (i, p) in chain_passes(frame).into_iter().enumerate() {
            let source = self.view(&ldr_name(i));
            let (depth, normals, object) = prepass_of(p);
            let g = if p == FramePass::Fxaa {
                group(
                    "fxaa",
                    &passes.source_layout,
                    &[tex(source), sampler.clone()],
                )
            } else if matches!(p, FramePass::SmaaBlend | FramePass::SmaaResolve) {
                let smaa = passes.smaa.as_ref().expect("SMAA renderer");
                chain.insert(
                    FramePass::SmaaEdges,
                    group(
                        "smaa edges",
                        &passes.source_layout,
                        &[tex(source), sampler.clone()],
                    ),
                );
                chain.insert(
                    FramePass::SmaaWeights,
                    group(
                        "smaa weights",
                        &smaa.weights_layout,
                        &[
                            tex(self.view("smaa.edges")),
                            sampler.clone(),
                            tex(&smaa.area),
                            tex(&smaa.search),
                            passes.taa_buffer.as_entire_binding(),
                        ],
                    ),
                );
                let blend = group(
                    "smaa blend",
                    &smaa.blend_layout,
                    &[tex(source), sampler.clone(), tex(self.view("smaa.weights"))],
                );
                if p == FramePass::SmaaResolve {
                    chain.insert(FramePass::SmaaBlend, blend);
                    continue;
                }
                blend
            } else {
                group(
                    &p.name(),
                    &passes.stylise_layout,
                    &[
                        tex(source),
                        sampler.clone(),
                        passes.stylise_buffer.as_entire_binding(),
                        tex(stylise.0),
                        tex(stylise.1),
                        wgpu::BindingResource::Sampler(&passes.repeat_sampler),
                        tex(&passes.blue_noise),
                        tex(depth),
                        tex(normals),
                        tex(object),
                    ],
                )
            };
            chain.insert(p, g);
        }
        let levels = meter_levels();
        let auto = has(FramePass::Meter);
        let metered = if auto {
            self.view(&luminance_name(levels - 1))
        } else {
            &passes.black
        };
        // The exposure the passes of frame slot `i` read: the adapted one
        // this slot writes, or the metered texel.
        let adapting = has(FramePass::Adapt);
        let lum = |slot: usize| -> &wgpu::TextureView {
            if adapting {
                &passes.exposure.views[slot]
            } else {
                metered
            }
        };
        let adapt = adapting.then(|| {
            [0usize, 1].map(|slot| {
                group(
                    "exposure adapt",
                    &passes.adapt_layout,
                    &[
                        tex(metered),
                        tex(&passes.exposure.views[1 - slot]),
                        passes.adapt_buffer.as_entire_binding(),
                    ],
                )
            })
        });
        let meter = auto.then(|| {
            [0, 1].map(|slot| {
                group(
                    "meter",
                    &passes.source_layout,
                    &[tex(source(slot)), sampler.clone()],
                )
            })
        });
        let reduce = if auto {
            (0..levels - 1)
                .map(|i| {
                    group(
                        "reduce",
                        &passes.source_layout,
                        &[tex(self.view(&luminance_name(i))), sampler.clone()],
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let god = if has(FramePass::GodRaysBlur) {
            self.view("god.blur")
        } else {
            &passes.black
        };
        let flare_view = if has(FramePass::FlareBlurY) {
            self.view("flare")
        } else {
            &passes.black
        };
        // Only the tonemap reads the flare: the bloom passes bind black
        // there, so no texture they render into can be one they hold.
        let post_group_with = |label: &str,
                               src: &wgpu::TextureView,
                               bloom: &wgpu::TextureView,
                               slot: usize,
                               flare: &wgpu::TextureView| {
            group(
                label,
                &passes.post_layout,
                &[
                    tex(src),
                    sampler.clone(),
                    passes.post_buffer.as_entire_binding(),
                    tex(lum(slot)),
                    tex(bloom),
                    tex(lut),
                    tex(god),
                    tex(flare),
                ],
            )
        };
        let post_group =
            |label: &str, src: &wgpu::TextureView, bloom: &wgpu::TextureView, slot: usize| {
                post_group_with(label, src, bloom, slot, &passes.black)
            };
        let bloom = has(FramePass::BloomPrefilter);
        let bloom_count = frame
            .iter()
            .filter(|p| matches!(p, FramePass::BloomDown(_)))
            .count() as u32
            + 1;
        let glow = if has(FramePass::Glow) {
            self.view("glow")
        } else {
            &passes.black
        };
        let prefilter = bloom
            .then(|| [0, 1].map(|slot| post_group("bloom prefilter", source(slot), glow, slot)));
        let (down, up) = if bloom {
            (
                (0..bloom_count - 1)
                    .map(|i| post_group("bloom down", self.view(&bloom_name(i)), &passes.black, 0))
                    .collect(),
                (1..bloom_count)
                    .map(|i| post_group("bloom up", self.view(&bloom_name(i)), &passes.black, 0))
                    .collect(),
            )
        } else {
            (Vec::new(), Vec::new())
        };
        let bloom_top = if bloom {
            self.view("bloom.0")
        } else {
            &passes.black
        };
        let post =
            [0, 1].map(|slot| post_group_with("post", source(slot), bloom_top, slot, flare_view));

        // The effect passes: (a, depth, gbuffer, b) per slot.
        let depth = self.try_view("lindepth").unwrap_or(&passes.far_depth);
        let gbuffer = self.try_view("gbuffer").unwrap_or(&passes.black);
        let fx_group_in = |uniform: &wgpu::Buffer,
                           a: &wgpu::TextureView,
                           d: &wgpu::TextureView,
                           b: &wgpu::TextureView| {
            group(
                "fx post",
                &passes.fx_layout,
                &[
                    uniform.as_entire_binding(),
                    sampler.clone(),
                    tex(a),
                    tex(d),
                    tex(gbuffer),
                    tex(b),
                    wgpu::BindingResource::Sampler(&passes.nearest_sampler),
                ],
            )
        };
        let fx_group_depth =
            |a: &wgpu::TextureView, d: &wgpu::TextureView, b: &wgpu::TextureView| {
                fx_group_in(&passes.fx_buffer, a, d, b)
            };
        let fx_group = |a: &wgpu::TextureView, b: &wgpu::TextureView| fx_group_depth(a, depth, b);
        // The passes after TAA: the frame it resolved (the output's size
        // under TAAU), with their own uniform's screen.
        let fx_after = |a: &wgpu::TextureView, b: &wgpu::TextureView| {
            fx_group_in(&passes.fx_after_buffer, a, depth, b)
        };
        let black = &passes.black;
        let mut fx = HashMap::new();
        for &p in frame {
            let pair = |f: &dyn Fn(usize) -> wgpu::BindGroup| [f(0), f(1)];
            let groups = match p {
                // The AO reads the depth with the water's surfaces in it when there is one.
                // The GI reads the lit image the previous frame left: the
                // TAA history it wrote, or the main pass's copy.
                FramePass::Ao => {
                    let d = self.try_view("ao-depth").unwrap_or(depth);
                    let previous = |slot: usize| match (cfg.gi, history, &persistent.gi) {
                        (false, ..) => black,
                        (true, Some(h), _) => &h[1 - slot],
                        (true, None, Some((_, g))) => &g[1 - slot],
                        (true, None, None) => black,
                    };
                    pair(&|slot| fx_group_depth(previous(slot), d, black))
                }
                FramePass::AoBlur => pair(&|_| fx_group(black, self.view("ao.raw"))),
                FramePass::AoUpsample => pair(&|_| fx_group(black, self.view("ao.blur"))),
                // The AO it passes through (none without the AO passes).
                FramePass::Contact => {
                    pair(&|_| fx_group(black, self.try_view("ao.pre").unwrap_or(&passes.white)))
                }
                FramePass::Ssr => pair(&|_| fx_group(self.view("hdr"), black)),
                FramePass::DepthBands => pair(&|_| fx_group(self.view(names.base), black)),
                FramePass::Outline => {
                    pair(&|_| fx_group(self.view(names.lit), self.view("outline-id")))
                }
                FramePass::DofCoc => pair(&|slot| fx_after(stage(names.aa, slot), black)),
                FramePass::DofBlur => pair(&|_| fx_after(self.view("dof.coc"), black)),
                FramePass::DofComposite => {
                    pair(&|slot| fx_after(stage(names.aa, slot), self.view("dof.blur")))
                }
                FramePass::MotionBlur => {
                    pair(&|slot| fx_after(stage(names.after_dof, slot), black))
                }
                FramePass::ScreenBlur => pair(&|slot| fx_after(stage(names.after_mb, slot), black)),
                FramePass::GodRaysMask => pair(&|slot| fx_after(source(slot), black)),
                FramePass::GodRaysBlur => pair(&|_| fx_after(black, self.view("god.mask"))),
                _ => continue,
            };
            fx.insert(p, groups);
        }
        // The lens flare's passes: (source, view depth, exposure,
        // visibility) per slot, placeholders where a pass reads none.
        let mut flare = HashMap::new();
        let flare_group = |src: &wgpu::TextureView, vis: &wgpu::TextureView, slot: usize| {
            group(
                "lens flare",
                &passes.flare.layout,
                &[
                    passes.flare.buffer.as_entire_binding(),
                    sampler.clone(),
                    tex(src),
                    tex(depth),
                    tex(lum(slot)),
                    tex(vis),
                ],
            )
        };
        for (p, src) in [
            (FramePass::FlareOcclusion, None),
            (FramePass::Flare, Some("bloom.0")),
            (FramePass::FlareBlurX, Some("flare.raw")),
            (FramePass::FlareBlurY, Some("flare.x")),
        ] {
            if has(p) {
                let src = src.map_or(black, |n| self.view(n));
                // The occlusion pass writes the visibility the others read.
                let vis = match p {
                    FramePass::FlareOcclusion => black,
                    _ => self.try_view("flare.vis").unwrap_or(black),
                };
                flare.insert(p, [0, 1].map(|slot| flare_group(src, vis, slot)));
            }
        }
        Bindings {
            flare,
            taa: taa_groups,
            smaa_resolve,
            taa_sharpen,
            chain,
            meter,
            reduce,
            adapt,
            prefilter,
            down,
            up,
            post,
            fx,
        }
    }

    /// Records one post or effect pass; returns its draws.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_post(
        &self,
        encoder: &mut crate::Encoder,
        passes: &PostPasses,
        persistent: &Persistent,
        pass: FramePass,
        slot: usize,
        output: &wgpu::TextureView,
        gtao: bool,
    ) -> u32 {
        let b = self.bindings.as_ref().expect("prepared");
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        // Screen-space GI in the AO passes' place (ssgi.rs).
        let gi = self.cfg.is_some_and(|c| c.gi);
        // GTAO with its bent normals: four channels blurred as the GI's.
        let bent = self.cfg.is_some_and(|c| c.bent);
        let half = self.views.contains_key("ao.blur");
        // With contact shadows the AO passes end in `ao.pre` (plan.rs).
        let ao = super::plan::ao_target_name(self.views.contains_key("ao.pre"));
        let effect = match pass {
            FramePass::Ao if gi => Some(("fs_ssgi", "ao.raw")),
            FramePass::Ao if bent => Some(("fs_gtao_bent", "ao.raw")),
            FramePass::Ao => Some((if gtao { "fs_gtao" } else { "fs_ssao" }, "ao.raw")),
            FramePass::AoBlur if gi || bent => {
                Some(("fs_gi_blur", if half { "ao.blur" } else { ao }))
            }
            // Half resolution: the blur's depth is at twice its texel, which
            // gathers do not read.
            FramePass::AoBlur if half => Some(("fs_ao_blur", "ao.blur")),
            FramePass::AoBlur => Some((AO_BLUR_GATHER, ao)),
            FramePass::AoUpsample if gi || bent => Some(("fs_gi_upsample", ao)),
            FramePass::AoUpsample => Some(("fs_ao_upsample", ao)),
            FramePass::Contact => Some(("fs_contact", "ao")),
            FramePass::Outline => Some(("fs_outline", "hdr.outline")),
            FramePass::DofCoc => Some(("fs_dof_coc", "dof.coc")),
            FramePass::DofBlur => Some(("fs_dof_blur", "dof.blur")),
            FramePass::DofComposite => Some(("fs_dof_composite", "hdr.dof")),
            FramePass::MotionBlur => Some(("fs_motion_blur", "hdr.mb")),
            FramePass::ScreenBlur => Some(("fs_screen_blur", "hdr.blur")),
            FramePass::GodRaysMask => Some(("fs_godrays_mask", "god.mask")),
            FramePass::GodRaysBlur => Some(("fs_godrays_blur", "god.blur")),
            FramePass::DepthBands => Some(("fs_depth_bands", "hdr.bands")),
            FramePass::Ssr => Some(("fs_ssr", "hdr.ssr")),
            _ => None,
        };
        if let Some((entry, target)) = effect {
            fullscreen_pass(
                encoder,
                entry,
                self.view(target),
                clear,
                &passes.fx[entry],
                &b.fx[&pass][slot],
            );
            return 1;
        }
        let flare = match pass {
            FramePass::FlareOcclusion => Some((&passes.flare.occlusion, "flare.vis")),
            FramePass::Flare => Some((&passes.flare.flare, "flare.raw")),
            FramePass::FlareBlurX => Some((&passes.flare.blur_x, "flare.x")),
            FramePass::FlareBlurY => Some((&passes.flare.blur_y, "flare")),
            _ => None,
        };
        if let Some((pipeline, target)) = flare {
            fullscreen_pass(
                encoder,
                &pass.name(),
                self.view(target),
                clear,
                pipeline,
                &b.flare[&pass][slot],
            );
            return 1;
        }
        match pass {
            FramePass::Taa => {
                if let (Some(pipeline), Some(history), Some(groups)) =
                    (&passes.taa, &persistent.history, &b.taa)
                {
                    fullscreen_pass(
                        encoder,
                        "taa",
                        &history[slot],
                        clear,
                        pipeline,
                        &groups[slot],
                    );
                    return 1;
                }
                0
            }
            FramePass::Meter => {
                let groups = b.meter.as_ref().expect("metering bound");
                fullscreen_pass(
                    encoder,
                    "meter",
                    self.view("luminance.0"),
                    clear,
                    &passes.meter,
                    &groups[slot],
                );
                1
            }
            FramePass::Reduce(i) => {
                fullscreen_pass(
                    encoder,
                    "meter reduce",
                    self.view(&luminance_name(i + 1)),
                    clear,
                    &passes.reduce,
                    &b.reduce[i as usize],
                );
                1
            }
            FramePass::Adapt => {
                let groups = b.adapt.as_ref().expect("adaptation bound");
                fullscreen_pass(
                    encoder,
                    "exposure adapt",
                    &passes.exposure.views[slot],
                    clear,
                    &passes.adapt,
                    &groups[slot],
                );
                1
            }
            FramePass::BloomPrefilter => {
                let groups = b.prefilter.as_ref().expect("bloom bound");
                fullscreen_pass(
                    encoder,
                    "bloom prefilter",
                    self.view("bloom.0"),
                    clear,
                    &passes.prefilter,
                    &groups[slot],
                );
                1
            }
            FramePass::BloomDown(i) => {
                fullscreen_pass(
                    encoder,
                    "bloom down",
                    self.view(&bloom_name(i + 1)),
                    clear,
                    &passes.down,
                    &b.down[i as usize],
                );
                1
            }
            FramePass::BloomUp(i) => {
                fullscreen_pass(
                    encoder,
                    "bloom up",
                    self.view(&bloom_name(i)),
                    wgpu::LoadOp::Load,
                    &passes.up,
                    &b.up[i as usize],
                );
                1
            }
            FramePass::Tonemap => {
                let chain = chain_passes(&self.passes);
                match (&passes.tonemap_ldr, chain.is_empty()) {
                    (Some(tonemap_ldr), false) => fullscreen_pass(
                        encoder,
                        "tonemap",
                        self.view(&ldr_name(0)),
                        clear,
                        tonemap_ldr,
                        &b.post[slot],
                    ),
                    _ => fullscreen_pass(
                        encoder,
                        "tonemap",
                        output,
                        clear,
                        &passes.tonemap,
                        &b.post[slot],
                    ),
                }
                1
            }
            FramePass::SmaaEdges | FramePass::SmaaWeights => {
                let smaa = passes.smaa.as_ref().expect("SMAA renderer");
                let (target, pipeline) = if pass == FramePass::SmaaEdges {
                    ("smaa.edges", &smaa.edges)
                } else {
                    ("smaa.weights", &smaa.weights)
                };
                fullscreen_pass(
                    encoder,
                    &pass.name(),
                    self.view(target),
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    pipeline,
                    &b.chain[&pass],
                );
                1
            }
            FramePass::SmaaBlend if persistent.smaa_history.is_some() => {
                let temporal = passes
                    .smaa
                    .as_ref()
                    .and_then(|s| s.temporal.as_ref())
                    .expect("T2x renderer");
                fullscreen_pass(
                    encoder,
                    "smaa history",
                    &persistent.smaa_history.as_ref().unwrap()[slot],
                    clear,
                    &temporal.blend,
                    &b.chain[&pass],
                );
                1
            }
            FramePass::Paint
            | FramePass::Lines
            | FramePass::Lens
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::SmaaBlend
            | FramePass::SmaaResolve
            | FramePass::Upscale => {
                let chain = chain_passes(&self.passes);
                let i = chain.iter().position(|&p| p == pass).expect("in the chain");
                let last = i + 1 == chain.len();
                let group = if pass == FramePass::SmaaResolve {
                    b.smaa_resolve.as_ref().map(|g| &g[slot])
                } else {
                    b.chain.get(&pass)
                };
                match (passes.chain.get(&pass), group) {
                    (Some((to_ldr, to_output)), Some(group)) => {
                        let (target, pipeline) = if last {
                            (output, to_output)
                        } else {
                            (self.view(&ldr_name(i + 1)), to_ldr)
                        };
                        fullscreen_pass(encoder, &pass.name(), target, clear, pipeline, group);
                        1
                    }
                    _ => 0,
                }
            }
            FramePass::TaaSharpen => match (&passes.taa_sharpen_pipeline, &b.taa_sharpen) {
                (Some(pipeline), Some(groups)) => {
                    fullscreen_pass(
                        encoder,
                        "taa sharpen",
                        self.view("hdr.sharp"),
                        clear,
                        pipeline,
                        &groups[slot],
                    );
                    1
                }
                _ => 0,
            },
            FramePass::Ao
            | FramePass::AoBlur
            | FramePass::AoUpsample
            | FramePass::Contact
            | FramePass::Outline
            | FramePass::DofCoc
            | FramePass::DofBlur
            | FramePass::DofComposite
            | FramePass::MotionBlur
            | FramePass::ScreenBlur
            | FramePass::CustomPost(_)
            | FramePass::GodRaysMask
            | FramePass::GodRaysBlur
            | FramePass::Cascade(_)
            | FramePass::Shelter
            | FramePass::LocalFace(_)
            | FramePass::Prepass
            | FramePass::Particles
            | FramePass::Light2d
            | FramePass::Metaballs
            | FramePass::Fluid3d
            | FramePass::Probes
            | FramePass::LiveSky
            | FramePass::GiProbes
            | FramePass::Reflection(_)
            | FramePass::View(_)
            | FramePass::Volumetric
            | FramePass::Main
            | FramePass::DepthBands
            | FramePass::Ssr
            | FramePass::Glow
            | FramePass::FlareOcclusion
            | FramePass::Flare
            | FramePass::FlareBlurX
            | FramePass::FlareBlurY => 0,
        }
    }
}

/// The graph target of planar reflection slot `k` (0: `reflection`, 1: `reflection.1`) with `suffix`.
pub(crate) fn reflection_target(k: u8, suffix: &str) -> &'static str {
    match (k, suffix) {
        (0, "") => "reflection",
        (0, "-depth") => "reflection-depth",
        (0, "-msaa") => "reflection-msaa",
        (_, "") => "reflection.1",
        (_, "-depth") => "reflection.1-depth",
        _ => "reflection.1-msaa",
    }
}
