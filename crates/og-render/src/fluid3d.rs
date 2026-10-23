//! 3D particle fluids (owed row 29): [`Fluid3d`], og-physics'
//! `og.Fluid3D` particles drawn as one screen-space surface (see
//! `shaders/fluid3d.wgsl`). The CPU uploads the particles each frame as
//! instances (their positions come from the CPU solver anyway), so every
//! tier, WebGL2 included, draws the same passes with no storage buffer or
//! compute:
//!
//! 1. sphere imposters write the nearest view depth (`fluid.depth`, with
//!    `fluid.zbuf` for the depth test), cut by the depth prepass;
//! 2. the same spheres add up their chords through each pixel weighted by
//!    the fluid's optical density, and their colour (`fluid.thick`);
//! 3. two passes of a separable bilateral filter smooth the depth
//!    (`fluid.blur`, then `fluid.smooth`) over the drawn radius times the
//!    smoothing, as seen at that depth;
//! 4. the composite, in the main pass after the scene is copied (as for
//!    water), rebuilds normals from the smoothed depth, refracts and
//!    absorbs the scene behind by the thickness, scatters the sun, the
//!    image-based light and the clustered local lights, reflects the
//!    environment and writes depth.
//!
//! All the frame's fluids share the surface: they merge where they meet,
//! each particle carrying its own colour and density; the look (how much
//! it reflects and refracts, how much it is smoothed) is the first fluid's.
//! Marching cubes was not built: the screen-space surface costs four
//! passes over the pixels the fluid covers whatever the particle count,
//! where a polygonised surface needs a density grid and a mesh rebuilt
//! every frame on the CPU for WebGL2.

use crate::deferred::{AsModule, Deferred};
use glam::{Mat4, Vec3};
use std::sync::Arc;

/// A 3D particle fluid to draw. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Fluid3d {
    /// The particles' world positions.
    pub particles: Arc<[Vec3]>,
    /// Each particle's linear colour (`None`, or a different length:
    /// `color` for all).
    pub colors: Option<Arc<[Vec3]>>,
    /// Linear colour of the body.
    pub color: Vec3,
    /// How much of the light passing through one metre it takes (0: clear,
    /// towards 1: opaque).
    pub opacity: f32,
    /// The radius each particle is drawn with (m).
    pub radius: f32,
    /// The volume each particle stands for (m³): with `opacity` it gives
    /// the optical depth of a column of fluid.
    pub volume: f32,
    /// How much it reflects (scales the Fresnel term).
    pub reflectivity: f32,
    /// How far refraction shifts what is seen through it.
    pub refraction: f32,
    /// How much the surface is smoothed (times the drawn radius; 0: the
    /// spheres).
    pub smoothing: f32,
}

impl Default for Fluid3d {
    fn default() -> Self {
        Fluid3d {
            particles: Arc::from(Vec::new()),
            colors: None,
            color: Vec3::new(0.2, 0.55, 0.9),
            opacity: 0.6,
            radius: 0.12,
            volume: 0.001,
            reflectivity: 1.0,
            refraction: 1.0,
            smoothing: 1.0,
        }
    }
}

/// `FluidUniform` in fluid3d.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FluidGpu {
    right: [f32; 4],
    up: [f32; 4],
    screen: [f32; 4],
    look: [f32; 4],
}

/// `BlurUniform` in fluid3d_blur.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurGpu {
    dir: [f32; 4],
    params: [f32; 4],
    screen: [f32; 4],
}

/// One particle as an instance: position and drawn radius; colour and
/// optical density per metre of chord.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    center_radius: [f32; 4],
    color_k: [f32; 4],
}

/// The largest blur radius, in pixels.
const MAX_BLUR_PX: f32 = 24.0;

/// The targets of [`crate::post::FramePass::Fluid3d`] and what it reads.
pub(crate) struct Fluid3dViews<'a> {
    pub scene_depth: &'a wgpu::TextureView,
    pub scene_color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
    pub zbuf: &'a wgpu::TextureView,
    pub thick: &'a wgpu::TextureView,
    pub blur: &'a wgpu::TextureView,
    pub smooth: &'a wgpu::TextureView,
}

/// The frame's targets for the fluid (`None` without them).
pub(crate) fn views<'a>(
    graph: &'a crate::post::FrameGraph,
    scene_depth: &'a wgpu::TextureView,
) -> Option<Fluid3dViews<'a>> {
    Some(Fluid3dViews {
        scene_depth,
        scene_color: graph.try_view("scene-copy")?,
        depth: graph.try_view("fluid.depth")?,
        zbuf: graph.try_view("fluid.zbuf")?,
        thick: graph.try_view("fluid.thick")?,
        blur: graph.try_view("fluid.blur")?,
        smooth: graph.try_view("fluid.smooth")?,
    })
}

/// The camera as the fluid needs it.
pub(crate) struct Fluid3dCamera {
    pub view: Mat4,
    pub proj: Mat4,
    pub orthographic: bool,
}

/// A frame's fluid surface, ready to record.
pub(crate) struct Fluid3dFrame {
    instances: u32,
    spheres: wgpu::BindGroup,
    blur: [wgpu::BindGroup; 2],
    composite: wgpu::BindGroup,
}

pub(crate) struct Fluid3dRenderer {
    sphere_layout: wgpu::BindGroupLayout,
    blur_layout: wgpu::BindGroupLayout,
    composite_layout: wgpu::BindGroupLayout,
    depth: Deferred<wgpu::RenderPipeline>,
    thick: Deferred<wgpu::RenderPipeline>,
    blur: Deferred<wgpu::RenderPipeline>,
    composite: Deferred<wgpu::RenderPipeline>,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    blur_uniforms: [wgpu::Buffer; 2],
    instances: wgpu::Buffer,
    capacity: u64,
    scratch: Vec<Instance>,
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, filterable: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn instance_buffer(device: &wgpu::Device, count: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fluid 3d instances"),
        size: count * std::mem::size_of::<Instance>() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    entries: &[(u32, wgpu::BindingResource)],
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = entries
        .iter()
        .map(|(binding, resource)| wgpu::BindGroupEntry {
            binding: *binding,
            resource: resource.clone(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fluid 3d"),
        layout,
        entries: &entries,
    })
}

impl Fluid3dRenderer {
    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.depth, &self.thick, &self.blur, &self.composite])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        shared_wgsl: &str,
        samples: u32,
    ) -> Fluid3dRenderer {
        let sphere_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid 3d spheres"),
            entries: &[uniform_entry(0), texture_entry(5, false)],
        });
        let composite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid 3d composite"),
            entries: &[
                uniform_entry(0),
                texture_entry(1, false),
                texture_entry(2, false),
                texture_entry(3, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture_entry(5, false),
            ],
        });
        let blur_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fluid 3d blur"),
            entries: &[uniform_entry(0), texture_entry(1, false)],
        });
        // Made when a fluid is first drawn (deferred.rs).
        let module = crate::deferred::module(device, "fluid 3d", fluid_source(shared_wgsl));
        let blur_module = crate::deferred::module(device, "fluid 3d blur", FLUID_BLUR_WGSL.into());
        let sphere_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluid 3d spheres"),
            bind_group_layouts: &[Some(frame_layout), Some(&sphere_layout)],
            immediate_size: 0,
        });
        let composite_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluid 3d composite"),
            bind_group_layouts: &[Some(frame_layout), Some(&composite_layout)],
            immediate_size: 0,
        });
        let blur_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fluid 3d blur"),
            bind_group_layouts: &[Some(&blur_layout)],
            immediate_size: 0,
        });
        let sphere = |label: &'static str, fs: &'static str, format, blend, depth: bool| {
            let (device, sphere_pl, module) = (device.clone(), sphere_pl.clone(), module.clone());
            Deferred::new(move || {
                let module = module.module();
                let attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
                let instance_layout = Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attributes,
                });
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&sphere_pl),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some("vs_sphere"),
                        compilation_options: Default::default(),
                        buffers: std::slice::from_ref(&instance_layout),
                    },
                    primitive: Default::default(),
                    depth_stencil: depth.then(|| wgpu::DepthStencilState {
                        format: crate::DEPTH_FORMAT,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some(fs),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let depth = sphere(
            "fluid 3d depth",
            "fs_depth",
            crate::post::LINDEPTH_FORMAT,
            None,
            true,
        );
        let add = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let thick = sphere(
            "fluid 3d thickness",
            "fs_thick",
            crate::HDR_FORMAT,
            Some(wgpu::BlendState {
                color: add,
                alpha: add,
            }),
            false,
        );
        let blur = crate::post::deferred_fullscreen(
            device,
            "fluid 3d blur",
            &blur_module,
            "fs",
            &blur_pl,
            crate::post::LINDEPTH_FORMAT,
            None,
        );
        let composite =
            crate::deferred_hdr(device, &composite_pl, module, move |layout, module| {
                crate::HdrPipeline {
                    label: "fluid 3d composite",
                    layout,
                    module,
                    vs: "vs_full",
                    fs: "fs_composite",
                    buffers: &[],
                    cull: None,
                    depth_write: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    blend: None,
                    samples,
                }
            });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fluid 3d"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let buffer = |label, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let blur_size = std::mem::size_of::<BlurGpu>();
        Fluid3dRenderer {
            sphere_layout,
            blur_layout,
            composite_layout,
            depth,
            thick,
            blur,
            composite,
            sampler,
            uniform: buffer("fluid 3d", std::mem::size_of::<FluidGpu>()),
            blur_uniforms: [
                buffer("fluid 3d blur x", blur_size),
                buffer("fluid 3d blur y", blur_size),
            ],
            instances: instance_buffer(device, 1024),
            capacity: 1024,
            scratch: Vec::new(),
        }
    }

    /// Uploads the frame's particles and uniforms and makes the bind
    /// groups. `None` when there is nothing to draw.
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fluids: &[Fluid3d],
        camera: &Fluid3dCamera,
        screen: (u32, u32),
        views: &Fluid3dViews,
    ) -> Option<Fluid3dFrame> {
        self.scratch.clear();
        for f in fluids {
            let r = f.radius;
            if !(r.is_finite() && r > 0.0) {
                continue;
            }
            let k = -(1.0 - f.opacity.clamp(0.0, 0.999)).ln();
            let share = f.volume.max(0.0) / (4.0 / 3.0 * std::f32::consts::PI * r * r * r);
            let density = if share.is_finite() { k * share } else { 0.0 };
            let colors = f.colors.as_ref().filter(|c| c.len() == f.particles.len());
            for (i, p) in f.particles.iter().enumerate() {
                if !p.is_finite() {
                    continue;
                }
                let c = colors.map_or(f.color, |c| c[i]);
                self.scratch.push(Instance {
                    center_radius: p.extend(r).to_array(),
                    color_k: c.max(Vec3::ZERO).extend(density).to_array(),
                });
            }
        }
        let first = fluids.iter().find(|f| !f.particles.is_empty())?;
        if self.scratch.is_empty() {
            return None;
        }
        let count = self.scratch.len() as u64;
        if count > self.capacity {
            self.capacity = count.next_power_of_two();
            self.instances = instance_buffer(device, self.capacity);
        }
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&self.scratch));
        let (w, h) = (screen.0 as f32, screen.1 as f32);
        let screen_v = [w, h, 1.0 / w.max(1.0), 1.0 / h.max(1.0)];
        let right = camera.view.row(0).truncate();
        let up = camera.view.row(1).truncate();
        // Pixels per metre at view depth 1 (per metre when orthographic).
        let scale = camera.proj.y_axis.y.abs() * h * 0.5;
        let ortho = f32::from(camera.orthographic);
        let gpu = FluidGpu {
            right: right.extend(scale).to_array(),
            up: up.extend(first.reflectivity.max(0.0)).to_array(),
            screen: screen_v,
            look: [first.refraction.max(0.0), ortho, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&gpu));
        let radius = first.radius.max(1e-4) * first.smoothing.max(0.0);
        for (buffer, dir) in self.blur_uniforms.iter().zip([[1.0, 0.0], [0.0, 1.0]]) {
            let b = BlurGpu {
                dir: [dir[0], dir[1], ortho, MAX_BLUR_PX],
                params: [scale, radius, first.radius.max(1e-4) * 2.0, 0.0],
                screen: screen_v,
            };
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&b));
        }
        let tex = wgpu::BindingResource::TextureView;
        let spheres = group(
            device,
            &self.sphere_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (5, tex(views.scene_depth)),
            ],
        );
        let blur = [(0, views.depth), (1, views.blur)].map(|(i, src)| {
            group(
                device,
                &self.blur_layout,
                &[
                    (0, self.blur_uniforms[i].as_entire_binding()),
                    (1, tex(src)),
                ],
            )
        });
        let composite = group(
            device,
            &self.composite_layout,
            &[
                (0, self.uniform.as_entire_binding()),
                (1, tex(views.smooth)),
                (2, tex(views.thick)),
                (3, tex(views.scene_color)),
                (4, wgpu::BindingResource::Sampler(&self.sampler)),
                (5, tex(views.scene_depth)),
            ],
        );
        Some(Fluid3dFrame {
            instances: count as u32,
            spheres,
            blur,
            composite,
        })
    }

    /// Records the depth, thickness and smoothing passes. Returns the draw
    /// calls.
    pub(crate) fn encode(
        &self,
        encoder: &mut crate::Encoder,
        frame: &Fluid3dFrame,
        frame_group: &wgpu::BindGroup,
        views: &Fluid3dViews,
    ) -> u32 {
        let far = wgpu::Color {
            r: crate::post::FAR_DEPTH as f64,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        };
        let attachment = |view, clear| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluid 3d depth"),
                color_attachments: &[attachment(views.depth, far)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: views.zbuf,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.depth);
            pass.set_bind_group(0, frame_group, &[]);
            pass.set_bind_group(1, &frame.spheres, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..frame.instances);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluid 3d thickness"),
                color_attachments: &[attachment(views.thick, wgpu::Color::TRANSPARENT)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.thick);
            pass.set_bind_group(0, frame_group, &[]);
            pass.set_bind_group(1, &frame.spheres, &[]);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..frame.instances);
        }
        for (target, group) in [(views.blur, &frame.blur[0]), (views.smooth, &frame.blur[1])] {
            crate::post::fullscreen_pass(
                encoder,
                "fluid 3d smoothing",
                target,
                wgpu::LoadOp::Clear(far),
                &self.blur,
                group,
            );
        }
        4
    }

    /// Draws the surface into the main pass (group 0: the frame). Returns
    /// the draw calls.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass, frame: &Fluid3dFrame) -> u32 {
        pass.set_pipeline(&self.composite);
        pass.set_bind_group(1, &frame.composite, &[]);
        pass.draw(0..3, 0..1);
        1
    }
}

/// The 3D fluid's module after `shared` (the frame group, the light
/// tables and the lighting), as the renderer compiles it.
pub(crate) fn fluid_source(shared: &str) -> String {
    format!("{shared}{}", wgsl3d!("fluid3d.wgsl"))
}

/// The 3D fluid's blur module.
pub(crate) const FLUID_BLUR_WGSL: &str = wgsl3d!("fluid3d_blur.wgsl");
