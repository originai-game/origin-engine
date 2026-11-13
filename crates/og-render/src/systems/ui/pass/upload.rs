//! A frame's UI tables and layer targets on the device.

use super::*;

impl UiRenderer {
    /// Writes the frame's tables and uniforms and makes its groups: the
    /// output's (`uniform`), each layer's, their targets and composite
    /// groups; `backdrop`: the glass pyramid, the copy of the target and
    /// the target the overlay draws into when it reads what is under it
    /// (the glass source), when the frame has them, with `decode`;
    /// `panel_view` finds a world panel's target by its texture index.
    #[allow(clippy::too_many_arguments)]
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        overlay_layout: &wgpu::BindGroupLayout,
        scene: &RenderScene,
        frame: &OverlayFrame,
        uniform: [f32; 8],
        backdrop: Option<(
            &wgpu::TextureView,
            &wgpu::TextureView,
            &wgpu::TextureView,
            bool,
        )>,
        texture_view: impl Fn(crate::TextureHandle) -> Option<wgpu::TextureView>,
        panel_view: impl Fn(usize) -> Option<wgpu::TextureView>,
        depth: &wgpu::TextureView,
        camera: super::super::mass::MassCameraGpu,
        mips: &crate::mips::MipGenerator,
    ) {
        let Some(overlay) = &scene.ui.overlay else {
            return;
        };
        write_storage(
            device,
            queue,
            &mut self.instances,
            "ui instances",
            bytemuck::cast_slice(&frame.ui_instances),
        );
        write_storage(
            device,
            queue,
            &mut self.gradients,
            "ui gradients",
            bytemuck::cast_slice(&frame.tables.gradients),
        );
        write_storage(
            device,
            queue,
            &mut self.clips,
            "ui clips",
            bytemuck::cast_slice(&frame.tables.clips),
        );
        write_storage(
            device,
            queue,
            &mut self.points,
            "ui points",
            bytemuck::cast_slice(&frame.tables.points),
        );
        let need = 1 + frame.layers.len();
        while self.uniforms.len() < need {
            self.uniforms.push(uniform_buffer(device, "ui frame", 32));
        }
        while self.layer_params.len() < frame.layers.len() {
            self.layer_params.push(uniform_buffer(
                device,
                "ui layer",
                std::mem::size_of::<LayerParams>() as u64,
            ));
        }
        let group0 = |buffer: &wgpu::Buffer| {
            crate::bind_group(
                device,
                "ui frame",
                overlay_layout,
                &[
                    buffer.as_entire_binding(),
                    self.clips.as_entire_binding(),
                    self.gradients.as_entire_binding(),
                    self.points.as_entire_binding(),
                    self.instances.as_entire_binding(),
                    wgpu::BindingResource::TextureView(depth),
                ],
            )
        };
        queue.write_buffer(&self.uniforms[0], 0, bytemuck::cast_slice(&uniform));
        let mut groups = UiFrameGroups {
            main: Some(group0(&self.uniforms[0])),
            ..Default::default()
        };
        if !frame.mass_instances.is_empty() {
            let n = frame
                .mass_templates
                .len()
                .min(super::super::UI_MASS_BATCHES);
            let mut camera = camera;
            camera.tick[1] = (n * super::super::mass::TEMPLATE_TEXELS) as u32;
            queue.write_buffer(&self.mass_camera, 0, bytemuck::bytes_of(&camera));
            let bytes: &[u8] = bytemuck::cast_slice(&frame.mass_instances);
            if bytes.len() as u64 > self.mass_instances.size() {
                self.mass_instances =
                    vertex_buffer(device, (bytes.len() as u64).next_power_of_two());
            }
            queue.write_buffer(&self.mass_instances, 0, bytes);
            // The table: rows of TABLE_WIDTH entries, at least one.
            let width = super::super::mass::TABLE_WIDTH;
            let mut texels: Vec<[f32; 4]> = Vec::with_capacity(
                n * super::super::mass::TEMPLATE_TEXELS + frame.mass_table.len(),
            );
            for t in &frame.mass_templates[..n] {
                texels.extend(t.texels());
            }
            texels.extend_from_slice(&frame.mass_table);
            let rows = (texels.len() as u32).div_ceil(width).max(1);
            if self.mass_table.as_ref().is_none_or(|t| t.2 < rows) {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("ui mass table"),
                    size: wgpu::Extent3d {
                        width,
                        height: rows.next_power_of_two(),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Float,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                self.mass_table = Some((texture, view, rows.next_power_of_two()));
            }
            let (texture, view, _) = self.mass_table.as_ref().expect("made above");
            texels.resize((rows * width) as usize, [0.0; 4]);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&texels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 16),
                    rows_per_image: Some(rows),
                },
                wgpu::Extent3d {
                    width,
                    height: rows,
                    depth_or_array_layers: 1,
                },
            );
            groups.mass = Some(crate::bind_group(
                device,
                "ui mass",
                &self.mass_layout,
                &[
                    self.mass_camera.as_entire_binding(),
                    wgpu::BindingResource::TextureView(view),
                ],
            ));
        }
        // Targets: one per layer drawn, reused by size and mips.
        let mut used = vec![false; self.targets.len()];
        let mut target_of = Vec::new();
        let wants_mips = |plan: &crate::overlay::LayerPlan| {
            layer_mips(overlay.ui.layers[plan.index as usize].material)
        };
        for plan in &frame.layers {
            // A world panel draws into its own render target; a backdrop
            // draws nothing of its own.
            if plan.target.is_some() || frame.backdrops.contains(&plan.index) {
                target_of.push(usize::MAX);
                continue;
            }
            let with_mips = wants_mips(plan);
            let found = self.targets.iter().enumerate().position(|(i, t)| {
                !used[i] && t.size == plan.size && t.chain.is_some() == with_mips
            });
            let i = match found {
                Some(i) => i,
                None => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("ui layer"),
                        size: wgpu::Extent3d {
                            width: plan.size.0,
                            height: plan.size.1,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: if with_mips {
                            crate::mips::mip_count(plan.size.0, plan.size.1)
                        } else {
                            1
                        },
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: LAYER_FORMAT,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    });
                    let sampled = texture.create_view(&Default::default());
                    let (view, chain) = if with_mips {
                        let chain = mips.chain(device, &texture);
                        (chain.level(0).clone(), Some(chain))
                    } else {
                        (sampled.clone(), None)
                    };
                    self.targets.push(LayerTarget {
                        size: plan.size,
                        texture: texture.clone(),
                        view,
                        sampled,
                        chain,
                    });
                    used.push(false);
                    self.targets.len() - 1
                }
            };
            used[i] = true;
            target_of.push(i);
        }
        // Targets no frame used are let go when there are many.
        if self.targets.len() > 16 {
            let mut k = 0;
            self.targets.retain(|_| {
                let keep = used.get(k).copied().unwrap_or(false);
                k += 1;
                keep
            });
            // Indices moved: find them again.
            target_of.clear();
            let mut taken = vec![false; self.targets.len()];
            for plan in &frame.layers {
                if plan.target.is_some() || frame.backdrops.contains(&plan.index) {
                    target_of.push(usize::MAX);
                    continue;
                }
                let i = self
                    .targets
                    .iter()
                    .enumerate()
                    .position(|(i, t)| {
                        !taken[i] && t.size == plan.size && t.chain.is_some() == wants_mips(plan)
                    })
                    .expect("kept above");
                taken[i] = true;
                target_of.push(i);
            }
        }
        self.filters.begin();
        // A layer's target as it is sampled: its view, top left and scale.
        let plan_of = |k: u32| frame.layers.iter().position(|p| p.index == k);
        let sampled = |n: usize| -> Option<(wgpu::TextureView, [f32; 2], f32)> {
            let plan = &frame.layers[n];
            let view = match plan.target {
                Some(t) => panel_view(t)?,
                None => self.targets.get(target_of[n])?.sampled.clone(),
            };
            Some((view, plan.origin, plan.scale))
        };
        let layer_input = |k: u32| plan_of(k).and_then(sampled);
        let inputs = super::super::filter::Inputs {
            white: &self.white,
            texture_view: &texture_view,
            layer: &layer_input,
        };
        for (n, plan) in frame.layers.iter().enumerate() {
            let layer = &overlay.ui.layers[plan.index as usize];
            let u = [
                plan.size.0 as f32,
                plan.size.1 as f32,
                plan.scale,
                0.0,
                plan.origin[0],
                plan.origin[1],
                uniform[6],
                0.0,
            ];
            queue.write_buffer(&self.uniforms[n + 1], 0, bytemuck::cast_slice(&u));
            let builtin = match layer.material {
                UiMaterial::Builtin(b) => builtin_code(b),
                _ => 0.0,
            };
            let mut params = LayerParams {
                rect: plan.rect,
                params: layer.params,
                info: [builtin, 0.0, 0.0, 0.0],
                mask_uv: [0.0, 0.0, 1.0, 1.0],
                drape: [[0.0; 4]; 6],
            };
            if let Some(g) = &layer.ground {
                let d = &g.drape;
                let w = [d.scale.x, d.scale.y, d.reach];
                params.info[3] = if g.plane { 2.0 } else { 1.0 };
                for i in 0..3 {
                    params.drape[i] = [d.a[i][0], d.a[i][1], d.a[i][2], w[i]];
                    params.drape[i + 3] = [d.b[i][0], d.b[i][1], d.b[i][2], 0.0];
                }
            }
            let mut mask_view = None;
            match &layer.mask {
                Some(UiMask::Gradient(_)) => {
                    params.info[1] = 1.0;
                    params.info[2] =
                        frame.layer_masks.get(&plan.index).copied().unwrap_or(0) as f32;
                }
                Some(UiMask::Image { texture, uv }) => {
                    if let Some(v) = texture_view(*texture) {
                        params.info[1] = 2.0;
                        params.mask_uv = *uv;
                        mask_view = Some(v);
                    }
                }
                None => {}
            }
            queue.write_buffer(&self.layer_params[n], 0, bytemuck::bytes_of(&params));
            // A world panel composites nowhere: its group samples white.
            let mut target = self
                .targets
                .get(target_of[n])
                .map_or(&self.white, |t| &t.sampled);
            // A filter's result instead of what the layer drew; a
            // backdrop's over what is under it where it composites, and
            // nothing when that cannot be read.
            let backdrop_of = frame.backdrops.contains(&plan.index);
            let parent = match plan.parent {
                _ if !backdrop_of => None,
                0 => backdrop
                    .map(|(_, _, source, decode)| (source.clone(), [0.0; 2], plan.scale, decode)),
                j => plan_of(j - 1)
                    .and_then(sampled)
                    .map(|(v, o, s)| (v, o, s, false)),
            };
            let source = match (&parent, self.targets.get(target_of[n])) {
                (Some((view, origin, scale, decode)), _) => Some(
                    super::super::filter::Source::Capture(super::super::filter::Capture {
                        view,
                        origin: *origin,
                        scale: *scale,
                        decode: *decode,
                    }),
                ),
                (None, Some(t)) if !backdrop_of => {
                    Some(super::super::filter::Source::Drawn(&t.sampled))
                }
                _ => None,
            };
            if let Some(source) = source
                && let Some(p) = self.filters.prepare(
                    device,
                    queue,
                    layer.filter.as_ref(),
                    source,
                    plan.size,
                    plan.origin,
                    plan.scale,
                    &inputs,
                )
            {
                groups.filters.insert(plan.index, p);
            }
            if let Some(p) = groups.filters.get(&plan.index) {
                target = &p.result;
            } else if backdrop_of {
                continue;
            }
            let composite = crate::bind_group(
                device,
                "ui layer composite",
                &self.layer_layout,
                &[
                    wgpu::BindingResource::TextureView(target),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.layer_params[n].as_entire_binding(),
                    wgpu::BindingResource::TextureView(mask_view.as_ref().unwrap_or(&self.white)),
                ],
            );
            groups.layers.insert(
                plan.index,
                (group0(&self.uniforms[n + 1]), target_of[n], composite),
            );
        }
        // What a layer's blends read: premultiplied, linear.
        let copies = frame.layers.iter().filter(|p| {
            frame
                .layer_runs(p.index)
                .iter()
                .any(|r| r.source.needs_copy())
        });
        let size = copies.fold((0, 0), |(w, h), p| (w.max(p.size.0), h.max(p.size.1)));
        if size.0 > 0 {
            if self
                .layer_behind
                .as_ref()
                .is_none_or(|(_, s)| s.0 < size.0 || s.1 < size.1)
            {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("ui layer behind"),
                    size: wgpu::Extent3d {
                        width: size.0,
                        height: size.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: LAYER_FORMAT,
                    usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                self.layer_behind = Some((texture, size));
            }
            queue.write_buffer(
                &self.layer_backdrop_params,
                0,
                bytemuck::cast_slice(&[0.0f32, 1.0, 0.0, 0.0]),
            );
            let (texture, _) = self.layer_behind.as_ref().expect("made above");
            let view = texture.create_view(&Default::default());
            groups.layer_backdrop = Some(crate::bind_group(
                device,
                "ui layer backdrop",
                &self.backdrop_layout,
                &[
                    wgpu::BindingResource::TextureView(&self.white),
                    wgpu::BindingResource::TextureView(&view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.layer_backdrop_params.as_entire_binding(),
                ],
            ));
        }
        if let Some((pyramid, behind, _, decode)) = backdrop {
            queue.write_buffer(
                &self.backdrop_params,
                0,
                bytemuck::cast_slice(&[
                    f32::from(u8::from(decode)),
                    0.0,
                    0.0,
                    crate::glass::LEVELS as f32,
                ]),
            );
            groups.backdrop = Some(crate::bind_group(
                device,
                "ui backdrop",
                &self.backdrop_layout,
                &[
                    wgpu::BindingResource::TextureView(pyramid),
                    wgpu::BindingResource::TextureView(behind),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.backdrop_params.as_entire_binding(),
                ],
            ));
        }
        self.frame = groups;
    }
}

/// A Porter-Duff pipe's index in [`OPERATORS`].
pub(super) fn operator(pipe: ShapePipe) -> usize {
    OPERATORS.iter().position(|o| *o == pipe).unwrap_or(0)
}

/// A built-in material's code in ui_builtin.wgsl.
pub(super) fn builtin_code(b: UiBuiltin) -> f32 {
    match b {
        UiBuiltin::None => 0.0,
        UiBuiltin::Dissolve => 1.0,
        UiBuiltin::Shine => 2.0,
        UiBuiltin::Hologram => 3.0,
        UiBuiltin::Glitch => 4.0,
        UiBuiltin::Ripple => 5.0,
        UiBuiltin::Pixelate => 6.0,
        UiBuiltin::ColorAdjust => 7.0,
        UiBuiltin::Outline => 8.0,
        UiBuiltin::Blur => 9.0,
        UiBuiltin::ColorMatrix => 10.0,
        UiBuiltin::DropShadow => 11.0,
    }
}
