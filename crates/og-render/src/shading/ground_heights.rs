//! The grounded ground's height map: what stands on the ground of a
//! grounded environment ([`crate::GroundProjection`]), seen from below,
//! from which the sky pass reads how much of the map's light reaches each
//! point of the ground (sky_ground.wgsl `ground_open`).
//!
//! It is a map of its own (no layer of the cascade array, no memory until
//! a frame grounds an environment whose ground takes shadows), drawn by the
//! cascades' caster pipelines: an orthographic view from under the ground
//! looking up, its depth reversed. A cascade's pipelines keep the casters' far sides, which
//! seen from below are the sides facing up, and the depth test keeps the
//! nearest, which with the depth reversed is the highest: each texel holds
//! the top of what stands over it (a floor under a prop is lower and loses
//! to it). Its texels are drawn again only when what it was drawn from
//! changes, as the weather's shelter map.

use glam::{Mat4, Vec3, Vec4};

use crate::ShadowMap;
use crate::deferred::{self, Deferred, Sampled};

/// The largest half side of the map's square, in metres: it covers the
/// ground out to this far from the dome's ground point (less when the dome
/// is smaller).
pub(crate) const GROUND_SPAN: f32 = 32.0;
/// How far below the ground the map's view starts, in metres.
pub(crate) const GROUND_BELOW: f32 = 1.0;
/// How far above the ground the map reaches, in metres (higher tops are
/// read as this high).
pub(crate) const GROUND_ABOVE: f32 = 63.0;

/// The maps' side in texels: 6 cm a texel over the widest ground.
pub(crate) const GROUND_SIZE: u32 = 512;

/// Whether a frame of `scene` reads the ground's maps: a grounded sky
/// background (`GroundProjection::gpu` draws its dome) whose ground takes
/// shadows.
pub(crate) fn wanted(scene: &crate::RenderScene) -> bool {
    let env = &scene.lighting.environment;
    matches!(env.background, crate::Background::Sky)
        && env.ground.is_some_and(|g| g.shadows > 0.0)
        && crate::GroundProjection::gpu(env.ground)[0][3] > 0.0
}

/// The sky pass's catcher: its pipeline for the grounded ground
/// (sky_ground.wgsl `fs_sky_ground`), the openness bake (ground_open.wgsl)
/// and their layouts, all made when first used; and the maps, made by the
/// first frame that grounds an environment whose ground takes shadows.
pub(crate) struct GroundCatcher {
    /// The sky pipeline's group 2: the openness.
    pub(crate) layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: Sampled<wgpu::RenderPipeline>,
    bake_layout: wgpu::BindGroupLayout,
    bake: Deferred<wgpu::RenderPipeline>,
    pub(crate) maps: Option<GroundMaps>,
}

/// The ground's maps: the heights (drawn by the casters), its uniform,
/// the openness baked from them, and the groups that read them.
pub(crate) struct GroundMaps {
    pub(crate) heights: ShadowMap,
    uniform: wgpu::Buffer,
    open: wgpu::TextureView,
    bake_group: wgpu::BindGroup,
    /// The sky pipeline's group 2.
    pub(crate) group: wgpu::BindGroup,
    /// What the heights were last drawn from (`ground_key`; `None`: not
    /// drawn yet).
    pub(crate) key: Option<u64>,
}

impl GroundCatcher {
    pub(crate) fn new(
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 2],
        samples: u32,
    ) -> GroundCatcher {
        use crate::gpu::layout::texture_entry;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky ground"),
            entries: &[texture_entry(0, true, wgpu::TextureViewDimension::D2)],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky ground"),
            bind_group_layouts: &[Some(layouts[0]), Some(layouts[1]), Some(&layout)],
            immediate_size: 0,
        });
        let module = deferred::module(device, "sky ground", crate::sky_ground_source());
        let open_source = open_source();
        let pipeline = crate::deferred_hdr(device, &pl, module, samples, |layout, module| {
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "sky ground",
                layout,
                module,
                vs: "vs_sky",
                fs: "fs_sky_ground",
                buffers: &[],
                cull: None,
                depth_write: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                blend: None,
                samples: 1,
            }
        });
        let bake_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ground open"),
            entries: &[
                texture_entry(0, false, wgpu::TextureViewDimension::D2),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bake = {
            let (device, layout) = (device.clone(), bake_layout.clone());
            Deferred::anywhere(move || {
                let module = crate::shader(&device, "ground open", open_source.clone());
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ground open"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_render_pipeline(
                    &device,
                    &wgpu::RenderPipelineDescriptor {
                        label: Some("ground open"),
                        layout: Some(&pl),
                        vertex: wgpu::VertexState {
                            module: &module,
                            entry_point: Some("vs_ground_open"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: Default::default(),
                        fragment: Some(wgpu::FragmentState {
                            module: &module,
                            entry_point: Some("fs_ground_open"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format: OPEN_FORMAT,
                                blend: None,
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        multiview_mask: None,
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        GroundCatcher {
            layout,
            pipeline,
            bake_layout,
            bake,
            maps: None,
        }
    }

    /// Makes the maps the first time a frame of `scene` wants them
    /// ([`wanted`]); never otherwise, so a sky without a grounded ground
    /// takes no memory for them.
    pub(crate) fn fit(&mut self, device: &wgpu::Device, scene: &crate::RenderScene) {
        if self.maps.is_some() || !wanted(scene) {
            return;
        }
        let heights = ShadowMap::new(
            device,
            "ground heights",
            GROUND_SIZE,
            1,
            wgpu::TextureFormat::R32Float,
            // Its one layer is bound as a 2D texture (the bake reads it).
            wgpu::TextureViewDimension::D2,
        );
        let uniform = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("ground open"),
                size: 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let open = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("ground open"),
                size: wgpu::Extent3d {
                    width: GROUND_SIZE,
                    height: GROUND_SIZE,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OPEN_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
                texture_binding_view_dimension: None,
            })
            .create_view(&Default::default());
        let bake_group = crate::bind_group(
            device,
            "ground open",
            &self.bake_layout,
            &[
                wgpu::BindingResource::TextureView(&heights.layers[0]),
                uniform.as_entire_binding(),
            ],
        );
        let group = crate::bind_group(
            device,
            "sky ground",
            &self.layout,
            &[wgpu::BindingResource::TextureView(&open)],
        );
        self.maps = Some(GroundMaps {
            heights,
            uniform,
            open,
            bake_group,
            group,
            key: None,
        });
    }

    /// The bake's uniform for `g` (written with the frame's).
    pub(crate) fn write(&self, queue: &wgpu::Queue, g: &crate::GroundProjection) {
        if let Some(m) = &self.maps {
            crate::traffic::write_buffer(
                queue,
                &m.uniform,
                0,
                bytemuck::cast_slice(&[ground_half(g), 0.0, 0.0, 0.0]),
            );
        }
    }

    /// Bakes the openness from the heights just drawn.
    pub(crate) fn bake(&self, encoder: &mut crate::Encoder) {
        let Some(m) = &self.maps else { return };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ground open"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &m.open,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.bake.wait());
        pass.set_bind_group(0, &m.bake_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// The openness bake's module (ground_open.wgsl), with the heights' range.
pub(crate) fn open_source() -> String {
    format!(
        "const GROUND_BELOW: f32 = {GROUND_BELOW:?};\nconst GROUND_ABOVE: f32 = {GROUND_ABOVE:?};\n{}",
        wgsl!("ground_open.wgsl")
    )
}

/// The openness map's format: a share, 0 to 1.
const OPEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

/// The map's half side for `g` (metres).
pub(crate) fn ground_half(g: &crate::GroundProjection) -> f32 {
    g.radius.min(GROUND_SPAN)
}

/// The map's view projection for `g` (frame coordinates): from
/// `GROUND_BELOW` under its ground point looking up (+y), +x to the
/// right and +z up the map, depth 1 at the eye and 0 at `GROUND_ABOVE`
/// over the ground. A depth `z` read back is the height
/// `ground - GROUND_BELOW + (1 - z) * (GROUND_BELOW + GROUND_ABOVE)`.
pub(crate) fn ground_view(g: &crate::GroundProjection) -> Mat4 {
    let eye = g.center - Vec3::Y * GROUND_BELOW;
    let view = glam::camera::rh::view::look_at_mat4(eye, eye + Vec3::Y, Vec3::Z);
    let h = ground_half(g);
    let proj = glam::camera::rh::proj::directx::orthographic(
        -h,
        h,
        -h,
        h,
        0.0,
        GROUND_BELOW + GROUND_ABOVE,
    );
    let reverse = Mat4::from_cols(
        Vec4::X,
        Vec4::Y,
        Vec4::new(0.0, 0.0, -1.0, 0.0),
        Vec4::W + Vec4::Z,
    );
    reverse * proj * view
}

/// What the map was drawn from: the dome's ground and the casters standing
/// in its volume (their bounding spheres) and the terrains. The map is
/// drawn again only when this changes.
pub(crate) fn ground_key(
    g: &crate::GroundProjection,
    spheres: impl Iterator<Item = (Vec3, f32)>,
    terrains: usize,
) -> u64 {
    let bits = |v: f32| u64::from(v.to_bits());
    let ground = [g.center.x, g.center.y, g.center.z, ground_half(g)].map(bits);
    let spheres = spheres.flat_map(|(c, r)| [c.x, c.y, c.z, r].map(bits));
    og_core::hash::hash_words(ground.into_iter().chain([terrains as u64]).chain(spheres))
}
