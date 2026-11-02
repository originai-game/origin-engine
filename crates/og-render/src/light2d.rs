//! The 2D light map: ambient plus every [`Light2d`], shadowed by
//! [`Occluder2d`]s with soft penumbrae (`shaders/light2d.wgsl`). Lit sprite
//! layers multiply their colour with it in the sprite shader.
//!
//! Light layers: each different light mask of the frame's lit receivers
//! ([`crate::RenderScene::sprite_light_masks`], the sprite meshes' and
//! tile layers' `light_mask`; at most [`MAX_LIGHT_MASKS_2D`]) has a strip
//! of its own, side by side in one
//! target, lit by the lights whose `layers` share a bit with it; a
//! receiver samples its strip ([`LightSlots`]). Mask 1 (layer 0, every
//! receiver without a mask) is always strip 0, so a frame without masks
//! draws one strip, as before there were any.

use crate::HDR_FORMAT;
use crate::deferred::{AsModule, Deferred};
use crate::effects::{
    Light2d, Light2dKind, MAX_LIGHT_MASKS_2D, MAX_LIGHTS_2D, MAX_OCCLUDERS_2D, Occluder2d,
};
use glam::{Mat4, Vec3};

pub(crate) const LIGHT2D_WGSL: &str = wgsl!("light2d.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Lights2dUniform {
    view_proj: [[f32; 4]; 4],
    info: [f32; 4],
    occluders: [[f32; 4]; 2 * MAX_OCCLUDERS_2D],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightInstance {
    pos_radius: [f32; 4],
    color_source: [f32; 4],
    /// x: casts shadows, y: kind (0 point, 1 spot, 2 global), z, w: a
    /// spot's cosines of its outer and inner half-angles.
    flags: [f32; 4],
    /// xy: the axis, z: height.
    direction: [f32; 4],
}

const LIGHT_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<LightInstance>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
};

/// The frame's receiver masks, by strip: mask 1 first, then each other
/// mask in the order receivers name it, at most [`MAX_LIGHT_MASKS_2D`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LightSlots(Vec<u32>);

impl LightSlots {
    pub(crate) fn new(masks: impl IntoIterator<Item = u32>) -> LightSlots {
        let mut slots = vec![1u32];
        for m in masks {
            if slots.len() < MAX_LIGHT_MASKS_2D && !slots.contains(&m) {
                slots.push(m);
            }
        }
        LightSlots(slots)
    }

    /// The masks of `scene`'s receivers in its lit layers: the sprites',
    /// then the sprite meshes', then the tile layers'.
    pub(crate) fn of(scene: &crate::RenderScene) -> LightSlots {
        let Some(l) = scene.lighting_2d else {
            return LightSlots::new([]);
        };
        let lit = |layer: i32| (l.min_layer..=l.max_layer).contains(&layer);
        let sprites = scene
            .sprite_light_masks
            .iter()
            .zip(&scene.sprites)
            .filter(|(_, s)| lit(s.layer))
            .map(|(m, _)| *m);
        let meshes = scene
            .sprite_meshes
            .iter()
            .filter(|m| lit(m.layer))
            .map(|m| m.light_mask);
        let tiles = scene
            .tile_layers
            .iter()
            .filter(|t| t.lit && lit(t.layer))
            .map(|t| t.light_mask);
        LightSlots::new(sprites.chain(meshes).chain(tiles))
    }

    /// A receiver's light: 0 outside `range` (no lighting, or a layer it
    /// does not light), else 1 + the strip of `mask` (the sprite shaders'
    /// `lit`).
    pub(crate) fn lit(&self, range: Option<(i32, i32)>, layer: i32, mask: u32) -> f32 {
        match range {
            Some((lo, hi)) if (lo..=hi).contains(&layer) => 1.0 + self.slot(mask) as f32,
            _ => 0.0,
        }
    }

    /// The strips the light map holds.
    pub(crate) fn count(&self) -> u32 {
        self.0.len() as u32
    }

    /// Receiver mask `mask`'s strip (0, mask 1's, past the limit).
    pub(crate) fn slot(&self, mask: u32) -> u32 {
        self.0.iter().position(|&m| m == mask).unwrap_or(0) as u32
    }
}

pub(crate) struct Light2dRenderer {
    pipeline: Deferred<wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    instances: wgpu::Buffer,
    group: wgpu::BindGroup,
    count: u32,
    /// Each strip's instances (`instances` holds them strip after strip).
    ranges: Vec<std::ops::Range<u32>>,
    /// The frame's uniform as written, which each view's copy takes with
    /// its own camera.
    written: Lights2dUniform,
    /// A uniform and group per view (`views.rs`), by its place in the
    /// frame's plans.
    views: Vec<(wgpu::Buffer, wgpu::BindGroup)>,
}

impl Light2dRenderer {
    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.pipeline])
    }

    pub fn new(device: &wgpu::Device) -> Light2dRenderer {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lights 2d"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d"),
            size: std::mem::size_of::<Lights2dUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d"),
            size: (std::mem::size_of::<LightInstance>() * MAX_LIGHTS_2D * MAX_LIGHT_MASKS_2D)
                as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights 2d"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        // Made when a frame first draws 2D lights (deferred.rs).
        let module = crate::deferred::module(device, "lights 2d", LIGHT2D_WGSL.into());
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lights 2d"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = {
            let (device, pl) = (device.clone(), pl.clone());
            Deferred::new(move || {
                let module = module.module();
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("lights 2d"),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[Some(LIGHT_LAYOUT)],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some("fs"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Light2dRenderer {
            pipeline,
            layout,
            uniform,
            instances,
            group,
            count: 0,
            ranges: Vec::new(),
            written: bytemuck::Zeroable::zeroed(),
            views: Vec::new(),
        }
    }

    /// Writes the frame's lights and occluders (beyond the limits they are
    /// dropped) for the strips of `slots`; returns (lights, occluders)
    /// drawn.
    pub fn write(
        &mut self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        lights: &[Light2d],
        occluders: &[Occluder2d],
        slots: &LightSlots,
    ) -> (u32, u32) {
        let mut u: Lights2dUniform = bytemuck::Zeroable::zeroed();
        u.view_proj = view_proj.to_cols_array_2d();
        let occ = occluders.len().min(MAX_OCCLUDERS_2D);
        u.info = [occ as f32, 0.0, 0.0, 0.0];
        for (i, o) in occluders.iter().take(occ).enumerate() {
            let (s, c) = o.rotation.sin_cos();
            u.occluders[i * 2] = [o.center.x, o.center.y, o.half_size.x, o.half_size.y];
            u.occluders[i * 2 + 1] = [c, s, 0.0, 0.0];
        }
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        self.written = u;
        let lights = &lights[..lights.len().min(MAX_LIGHTS_2D)];
        let mut instances: Vec<LightInstance> = Vec::new();
        self.ranges.clear();
        for &mask in &slots.0 {
            let start = instances.len() as u32;
            instances.extend(lights.iter().filter(|l| l.layers & mask != 0).map(instance));
            self.ranges.push(start..instances.len() as u32);
        }
        if !instances.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
        }
        self.count = instances.len() as u32;
        (lights.len() as u32, occ as u32)
    }

    /// Writes view `k`'s uniform: the frame's lights and occluders seen
    /// through `view_proj` (a view's camera, `views.rs`).
    pub fn write_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        k: usize,
        view_proj: Mat4,
    ) {
        while self.views.len() <= k {
            let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("lights 2d (view)"),
                size: std::mem::size_of::<Lights2dUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lights 2d (view)"),
                layout: &self.layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }],
            });
            self.views.push((uniform, group));
        }
        let mut u = self.written;
        u.view_proj = view_proj.to_cols_array_2d();
        queue.write_buffer(&self.views[k].0, 0, bytemuck::bytes_of(&u));
    }

    /// Clears `target` (`size`: its strips side by side) to `ambient` and
    /// adds the lights as view `k` sees them
    /// ([`Light2dRenderer::write_view`]).
    pub fn encode_view(
        &self,
        encoder: &mut crate::Encoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        ambient: Vec3,
        k: usize,
    ) {
        self.encode_with(encoder, target, size, ambient, &self.views[k].1);
    }

    /// Clears `target` (`size`: its strips side by side) to `ambient` and
    /// adds the lights.
    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        ambient: Vec3,
    ) {
        self.encode_with(encoder, target, size, ambient, &self.group);
    }

    fn encode_with(
        &self,
        encoder: &mut crate::Encoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        ambient: Vec3,
        group: &wgpu::BindGroup,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lights 2d"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: ambient.x as f64,
                        g: ambient.y as f64,
                        b: ambient.z as f64,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        let strips = self.ranges.len().max(1) as u32;
        let width = size.0 / strips;
        for (s, r) in self.ranges.iter().enumerate() {
            if r.is_empty() {
                continue;
            }
            if strips > 1 {
                pass.set_viewport(
                    (s as u32 * width) as f32,
                    0.0,
                    width as f32,
                    size.1 as f32,
                    0.0,
                    1.0,
                );
            }
            pass.draw(0..6, r.clone());
        }
    }
}

/// The light map's size for `strips` strips of `size` side by side,
/// narrower strips when they would pass `max` texels.
pub(crate) fn strips_size(size: (u32, u32), strips: u32, max: u32) -> (u32, u32) {
    let strips = strips.max(1);
    let width = size.0.min(max / strips).max(1);
    (width * strips, size.1.min(max).max(1))
}

/// One light's instance.
fn instance(l: &Light2d) -> LightInstance {
    let c: Vec3 = l.color * l.intensity;
    let kind = match l.kind {
        Light2dKind::Point => 0.0,
        Light2dKind::Spot => 1.0,
        Light2dKind::Global => 2.0,
    };
    let outer = l
        .outer_angle
        .max(l.inner_angle)
        .clamp(0.0, std::f32::consts::PI);
    let inner = l.inner_angle.clamp(0.0, outer);
    let d = l.direction.normalize_or(glam::Vec2::X);
    LightInstance {
        pos_radius: [l.position.x, l.position.y, l.radius.max(1e-3), l.falloff],
        color_source: c.extend(l.source_radius.max(0.0)).to_array(),
        flags: [
            (l.cast_shadows && l.kind != Light2dKind::Global) as u32 as f32,
            kind,
            outer.cos(),
            inner.cos(),
        ],
        direction: [d.x, d.y, l.height.max(0.0), 0.0],
    }
}
