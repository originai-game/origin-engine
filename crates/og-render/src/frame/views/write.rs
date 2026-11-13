//! The views' uniforms: each view's camera as the frame uniform its passes read.

use super::*;

/// A secondary camera's billboard and line uniforms (a view's, a probe
/// face's): billboards face it; lines keep their width as a share of the
/// image (CSS pixels of a frame of its height).
pub(crate) fn camera_gpu(
    frame: &CameraFrame,
    position: Vec3,
    size: (u32, u32),
    css_height: f32,
) -> (
    crate::particles_gpu::ParticleViewGpu,
    crate::lines::LineUniform,
) {
    let view_t = frame.view.transpose();
    let (w, h) = (size.0 as f32, size.1 as f32);
    let particles = crate::particles_gpu::ParticleViewGpu {
        view_proj: frame.view_proj.to_cols_array_2d(),
        cam_pos: position.extend(frame.orthographic as u32 as f32).to_array(),
        cam_forward: frame.forward.extend(0.0).to_array(),
        cam_right: view_t.x_axis.truncate().extend(0.0).to_array(),
        cam_up: view_t.y_axis.truncate().extend(0.0).to_array(),
        screen: [w, h, 1.0 / w, 1.0 / h],
    };
    let lines = crate::lines::LineUniform::new(
        frame.view_proj,
        frame.proj,
        size.0,
        size.1,
        css_height,
        false,
    );
    (particles, lines)
}

impl Views {
    /// Writes the views' frame, effect and light-cluster uniforms (the
    /// frame's with each view's camera) and makes their resolve pipelines.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        samples: u32,
        kept: &[usize],
        scene: &crate::RenderScene,
        plans: &[ViewPlan],
        uniform: &crate::frame::FrameUniform,
        fx: &crate::fx::FxUniform,
        lights: bool,
        viewport_format: wgpu::TextureFormat,
        light_map: Option<(f32, u32)>,
        fields: usize,
        soft: bool,
        glow: Option<f32>,
    ) {
        if glow.is_some() && self.glow_add.is_none() {
            self.glow_add = Some(glow_add(device));
        }
        // Held apart while the slots are borrowed, put back below.
        let add = self.glow_add.take();
        for p in plans {
            // A depth texture, or soft particles to fade against it.
            let depth_drawn = soft || matches!(p.dest, Dest::Texture { depth: Some(_), .. });
            let slot = self.slot(
                device,
                p.index,
                p.size,
                samples,
                depth_drawn,
                std::mem::size_of::<crate::frame::FrameUniform>() as u64,
                std::mem::size_of::<crate::fx::FxUniform>() as u64,
            );
            let frustum = crate::Frustum::from_view_proj(p.frame.view_proj);
            slot.clusters.build_kept(
                &scene.lighting.lights,
                kept,
                &crate::clusters::ClusterView {
                    camera: &p.frame,
                    frustum: &frustum,
                    position: p.position,
                    near: p.near,
                    far: p.far,
                    width: p.size.0,
                    height: p.size.1,
                },
            );
            if lights {
                queue.write_buffer(
                    &slot.light_buffers[1],
                    0,
                    bytemuck::cast_slice(&slot.clusters.table),
                );
                queue.write_buffer(
                    &slot.light_buffers[2],
                    0,
                    bytemuck::cast_slice(&slot.clusters.ref_words()),
                );
            }
            let vp = p.frame.view_proj.to_cols_array_2d();
            let mut u = *uniform;
            u.view_proj = vp;
            u.inv_view_proj = p.frame.view_proj.inverse().to_cols_array_2d();
            u.view_pos = p
                .position
                .extend(p.frame.orthographic as u32 as f32)
                .to_array();
            u.view_forward = p.frame.forward.extend(u.view_forward[3]).to_array();
            (u.view_right, u.view_up) = p.frame.axes();
            u.cluster = slot.clusters.params;
            // The main camera's screen-space reflections lie elsewhere.
            u.ssr = [0.0; 4];
            queue.write_buffer(&slot.frame_buffer, 0, bytemuck::bytes_of(&u));
            let mut f = *fx;
            f.ao = [0.0; 4];
            f.view_proj_unjittered = vp;
            f.prev_view_proj = vp;
            let (w, h) = (p.size.0 as f32, p.size.1 as f32);
            f.screen = [w, h, 1.0 / w, 1.0 / h];
            crate::fx::unclustered(&mut f);
            queue.write_buffer(&slot.fx_buffer, 0, bytemuck::bytes_of(&f));
            let strips = light_map.map_or(1, |(_, s)| s.max(1));
            let ambient = scene
                .sprite
                .lighting_2d
                .map_or(glam::Vec3::ONE, |l| l.ambient);
            queue.write_buffer(
                &slot.sprite_fx_buffer,
                0,
                bytemuck::bytes_of(&[f.screen, [strips as f32, ambient.x, ambient.y, ambient.z]]),
            );
            // The 2D light map at the frame's share of the view's size,
            // its strips side by side (light2d.rs).
            slot.light_map = light_map.map(|(scale, strips)| {
                let size = crate::light2d::strips_size(
                    (
                        ((p.size.0 as f32 * scale) as u32).max(1),
                        ((p.size.1 as f32 * scale) as u32).max(1),
                    ),
                    strips,
                    device.limits().max_texture_dimension_2d,
                );
                match slot.light_map.take() {
                    Some((s, v)) if s == size => (s, v),
                    _ => {
                        slot.light_dir = None;
                        (size, hdr_texture(device, "view light map", size))
                    }
                }
            });
            slot.light_dir = match &slot.light_map {
                Some((size, _)) if scene.has_texture_normals() => Some(
                    slot.light_dir
                        .take()
                        .unwrap_or_else(|| hdr_texture(device, "view light directions", *size)),
                ),
                _ => None,
            };
            // Field targets follow the attachments' size (`slot` remade
            // them with a new key).
            if slot.fields.len() != fields || slot.fields_size != p.size {
                slot.fields = (0..fields)
                    .map(|_| hdr_texture(device, "view metaball field", p.size))
                    .collect();
                slot.fields_size = p.size;
            }
            slot.glow = match glow {
                None => None,
                Some(intensity) => {
                    let size = ((p.size.0 / 2).max(1), (p.size.1 / 2).max(1));
                    let levels = crate::post::bloom_levels(p.size.0, p.size.1);
                    let g = match slot.glow.take() {
                        Some(g) if g.size == size && g.levels.len() == levels as usize => g,
                        _ => view_glow(
                            device,
                            queue,
                            add.as_ref().expect("made above"),
                            size,
                            levels,
                        ),
                    };
                    // The upsample's radius, one texel, as the frame's.
                    let u = [intensity.max(0.0), 1.0];
                    for uniform in &g.uniforms {
                        queue.write_buffer(uniform, 0, bytemuck::cast_slice(&u));
                    }
                    Some(g)
                }
            };
            let (particles, lines) =
                camera_gpu(&p.frame, p.position, p.size, scene.frame.css_height);
            queue.write_buffer(&slot.particle_buffer, 0, bytemuck::bytes_of(&particles));
            queue.write_buffer(&slot.line_buffer, 0, bytemuck::bytes_of(&lines));
        }
        self.glow_add = add;
        for p in plans {
            match p.dest {
                Dest::Texture { format, depth, .. } => {
                    let f = format.wgpu();
                    self.pipeline(device, f, resolve_entry(f, false));
                    if depth.is_some() {
                        self.pipeline(device, wgpu::TextureFormat::Rgba16Float, "fs_depth");
                    }
                }
                Dest::Viewport(_) => {
                    let f = viewport_format;
                    self.pipeline(device, f, "fs_color");
                }
            }
        }
    }
}
