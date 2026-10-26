//! Water (the E5 round). [`Water`]: a planar 3D surface drawn after the
//! opaque scene and the sky (see `shaders/water.wgsl`): a grid on the
//! surface's local xz plane, displaced by an optional height field (the
//! wave simulation of og-physics' `og.WaterSurface`), animated detail
//! normals, colour by the depth of water below (from the depth prepass,
//! which a frame with water always runs), foam where it meets geometry and
//! on steep crests, refraction of a copy of the scene drawn so far, and
//! reflection: a march through the depth prepass on the full tier, the
//! sky's prefiltered cubemap where that misses and everywhere on WebGL2.
//!
//! Phase 2 adds, each off by default: a **planar reflection** (the scene
//! drawn again from the camera mirrored in the surface, clipped at the
//! surface by an oblique near plane, at a fraction of the output's
//! resolution: one surface a frame), the directional light's **shadows**
//! on the surface, **caustics** on what lies below it (the lit and terrain
//! shaders read them from the frame), and the **underwater view**: with
//! the camera below the surface the frame's fog becomes the water's
//! (`underwater_color`, `underwater_density`) and the surface, seen from
//! below, shows what is above inside Snell's window and reflects the
//! water's colour outside it.
//!
//! [`Water2d`]: a side-on water body (og-physics' `og.WaterLine`), drawn
//! through the sprite order as a strip: the surface line's columns over a
//! body that darkens with depth and a bright lip along the top.

use glam::{Mat4, Vec2, Vec3, Vec4};
use std::sync::Arc;

/// A height field over a square of `size` metres centred on the surface's
/// origin: `resolution` x `resolution` heights, row-major from -z to +z
/// and -x to +x (og-physics' `waterHeights`).
#[derive(Clone, Debug, PartialEq)]
pub struct WaterHeights {
    pub resolution: u32,
    pub size: f32,
    pub heights: Arc<[f32]>,
}

/// A planar 3D water surface. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Water {
    /// World from the surface's frame: the water lies on local y = 0,
    /// centred on the origin.
    pub transform: Mat4,
    /// The extent along local x and z, in metres.
    pub size: Vec2,
    /// Grid cells per side (the height field's detail; 1 for a flat sheet).
    /// The renderer draws more where a cell would be longer than 256 m.
    pub segments: u32,
    /// Linear colour of shallow water (tints what is seen through it).
    pub shallow: Vec3,
    /// Linear colour of deep water.
    pub deep: Vec3,
    /// The depth (m) over which water turns from shallow to deep.
    pub depth_fade: f32,
    /// How opaque deep water gets (0: clear, 1: the deep colour).
    pub opacity: f32,
    /// Linear foam colour.
    pub foam: Vec3,
    /// The width (m of water depth) of the foam band where the water meets
    /// geometry (0: no foam).
    pub foam_width: f32,
    /// Detail waves per metre.
    pub wave_scale: f32,
    /// How fast the detail waves travel.
    pub wave_speed: f32,
    /// How strongly they bend the normal (0: a mirror).
    pub wave_strength: f32,
    /// How far refraction shifts what is seen through the water.
    pub refraction: f32,
    /// How much it reflects (scales the Fresnel term).
    pub reflectivity: f32,
    /// The wave simulation's heights (`None`: flat).
    pub heights: Option<WaterHeights>,
    /// Reflect the scene by drawing it again mirrored in the surface, at
    /// this fraction of the output's resolution (0: off; the screen-space
    /// and cubemap reflections). One surface a frame: the first that asks.
    pub planar_reflection: f32,
    /// How much the directional light's shadows darken the surface's
    /// highlight, foam and colour (0: none, 1: fully).
    pub shadows: f32,
    /// The reflectance seen straight down (Schlick's F0): 0.02 for clean
    /// water; three.js Water.js uses 0.3.
    pub f0: f32,
    /// A floor under the reflected colour, `bias + (1 - bias) · reflected`
    /// (three.js Water.js: 0.1).
    pub reflection_bias: f32,
    /// With the camera below the surface: the water's fog colour and its
    /// density per metre (0: no underwater view).
    pub underwater_color: Vec3,
    pub underwater_density: f32,
    /// Caustics on what lies below the surface (0: none) and their size
    /// (cells per metre). One surface a frame: the first that has them.
    pub caustics: f32,
    pub caustics_scale: f32,
    /// Draw the water over this mesh instead of the `size` grid: its
    /// vertices in the surface's frame (`transform`, scale included), its
    /// normals the surface's. Streams, channels and pools of any outline get
    /// the same depth colour, refraction, foam and reflections. No height
    /// field; the planar reflection still mirrors in the frame's xz plane.
    pub mesh: Option<crate::scene::MeshHandle>,
    /// Ripples from a tangent-space normal map (three.js' water normals:
    /// red +x, green +z, blue up) instead of the travelling waves.
    pub normal_map: Option<crate::TextureHandle>,
    /// The map's repeats per metre (lake) or per uv unit (flow).
    pub normal_scale: f32,
    /// Zero: three.js Water's four drifting layers over world xz. Else
    /// three.js Water2's two layers sliding along it in uv (length: uv per
    /// second), cross-faded, with Water2's look: refraction shifts by
    /// N.xz × 0.05 × `refraction` of the screen, the reflection is tinted by
    /// `shallow` as well, and there is no sun highlight.
    pub flow: Vec2,
    /// three.js Water's sun glint on the mirrored image (spec × sunColor
    /// over the light's intensity); 0: the half-vector highlight. With it
    /// the body is Water.js's too: `deep` is its waterColor, scattered by
    /// N·V, plus its sun diffuse (sunColor = light × glint / 2), shadowed.
    pub glint: f32,
}

impl Default for Water {
    fn default() -> Self {
        Water {
            transform: Mat4::IDENTITY,
            size: Vec2::splat(16.0),
            segments: 64,
            shallow: Vec3::new(0.55, 0.85, 0.82),
            deep: Vec3::new(0.02, 0.09, 0.14),
            depth_fade: 2.5,
            opacity: 0.92,
            foam: Vec3::new(0.95, 0.97, 1.0),
            foam_width: 0.35,
            wave_scale: 0.6,
            wave_speed: 1.0,
            wave_strength: 0.35,
            refraction: 1.0,
            reflectivity: 1.0,
            heights: None,
            planar_reflection: 0.0,
            shadows: 0.0,
            f0: 0.02,
            reflection_bias: 0.0,
            underwater_color: Vec3::new(0.03, 0.16, 0.2),
            underwater_density: 0.09,
            caustics: 0.0,
            caustics_scale: 0.6,
            mesh: None,
            normal_map: None,
            normal_scale: 1.0,
            flow: Vec2::ZERO,
            glint: 0.0,
        }
    }
}

/// A side-on 2D water body. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Water2d {
    /// The surface's left end at rest, in world units (the body hangs
    /// below it by `depth`).
    pub origin: Vec2,
    pub width: f32,
    pub depth: f32,
    /// Offsets of the surface from rest, evenly spaced across `width`
    /// (at least two).
    pub surface: Vec<f32>,
    /// Linear rgba at the top and at the bottom of the body.
    pub top: Vec4,
    pub bottom: Vec4,
    /// Linear rgba of the lip along the surface and its thickness.
    pub lip: Vec4,
    pub lip_width: f32,
    /// The sprite layer it is drawn in, with the sprites' ordering.
    pub layer: i32,
    /// World z of the body (with sprites that sort by z).
    pub z: f32,
}

impl Default for Water2d {
    fn default() -> Self {
        Water2d {
            origin: Vec2::ZERO,
            width: 12.0,
            depth: 3.0,
            surface: vec![0.0; 2],
            top: Vec4::new(0.16, 0.5, 0.72, 0.72),
            bottom: Vec4::new(0.02, 0.1, 0.25, 0.92),
            lip: Vec4::new(0.8, 0.95, 1.0, 0.9),
            lip_width: 0.06,
            layer: 0,
            z: 0.0,
        }
    }
}

/// `WaterUniform` in water.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WaterGpu {
    model: [[f32; 4]; 4],
    size: [f32; 4],
    shallow: [f32; 4],
    deep: [f32; 4],
    foam: [f32; 4],
    waves: [f32; 4],
    look: [f32; 4],
    screen: [f32; 4],
    /// x: shadows, y: 1 drawn with the planar reflection, zw: unused.
    extra: [f32; 4],
    /// rgb: the underwater colour, w: the glint (0: off).
    under: [f32; 4],
    /// The normal map: x repeats, yz the flow (zero: the lake's layers),
    /// w 1 when there is one.
    normals: [f32; 4],
}

/// The most grid cells per side (256² cells, 131,072 triangles per
/// surface). It caps [`MAX_CELL`]'s split: a surface longer than 65.5 km
/// (256 × 256 m) gets cells longer than 256 m again.
const MAX_SEGMENTS: u32 = 256;
/// The largest height field accepted.
const MAX_RESOLUTION: u32 = 512;

/// The longest a grid cell may be, in world metres. A rasterizer
/// interpolates a triangle's attributes and depth from its corners in f32,
/// so on a kilometres-long triangle that the near plane clips both land
/// metres off at the far side: the 24 km sea of og-tests' `terrain-far`
/// lost its shore foam and the lake on llvmpipe and drew a foam band some
/// pixels too wide on Metal. Cells of at most 256 m draw the same thin band
/// everywhere.
const MAX_CELL: f32 = 256.0;

/// Grid cells per side: the surface's own `segments`, or more where a cell
/// would be longer than [`MAX_CELL`] (at most [`MAX_SEGMENTS`]).
fn grid_segments(water: &Water) -> u32 {
    let scale = |axis: Vec4| axis.truncate().length();
    let extent = (water.size.x * scale(water.transform.x_axis))
        .max(water.size.y * scale(water.transform.z_axis));
    let wanted = if extent.is_finite() {
        (extent / MAX_CELL).ceil() as u32
    } else {
        1
    };
    water.segments.max(wanted).clamp(1, MAX_SEGMENTS)
}

struct Slot {
    uniform: wgpu::Buffer,
    heights: Option<(u32, wgpu::Texture, wgpu::TextureView)>,
}

/// Draws [`Water`]: one pipeline, a uniform buffer and height texture per
/// surface, kept across frames.
pub(crate) struct WaterRenderer {
    layout: wgpu::BindGroupLayout,
    pub(crate) pipeline: crate::deferred::Deferred<wgpu::RenderPipeline>,
    /// For surfaces drawn over a mesh ([`Water::mesh`]).
    pub(crate) mesh_pipeline: crate::deferred::Deferred<wgpu::RenderPipeline>,
    /// The surfaces' view depth for the screen AO (`fs_ao_depth`; grid, mesh): into an R32 copy of the prepass
    /// depth, tested against the prepass, writing no depth.
    ao_depth_pipelines: crate::deferred::Deferred<[wgpu::RenderPipeline; 2]>,
    ripple_sampler: wgpu::Sampler,
    sampler: wgpu::Sampler,
    flat: wgpu::TextureView,
    slots: Vec<Slot>,
}

/// One surface ready to draw.
pub(crate) struct WaterDraw {
    group: wgpu::BindGroup,
    vertices: u32,
    /// Its mesh's vertex and index buffers and index count.
    mesh: Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
}

impl WaterRenderer {
    /// Makes its pipeline if no frame used it yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.pipeline, &self.mesh_pipeline])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        lighting_wgsl: &str,
        samples: u32,
    ) -> WaterRenderer {
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                tex(1, false),
                tex(2, true),
                tex(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                tex(5, true),
                tex(6, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // The frame's screen-space AO (white without): the reflection is darkened by it as
                // the scene behind already is (water.wgsl).
                tex(8, true),
            ],
        });
        let module =
            crate::deferred::module(device, "water", water_source(frame_wgsl, lighting_wgsl));
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        let mesh_pipeline = crate::deferred_hdr(device, &pl, module.clone(), move |layout, module| {
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water mesh",
                layout,
                module,
                vs: "vs_mesh",
                fs: "fs",
                buffers: &[Some(crate::POSITION_NORMAL_UV_LAYOUT)],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            }
        });
        let ao_depth_pipelines = {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            crate::deferred::Deferred::new(move || {
                use crate::deferred::AsModule;
                let m = module.module();
                [("water ao depth", "vs", &[][..]), ("water mesh ao depth", "vs_mesh", &[Some(crate::POSITION_NORMAL_UV_LAYOUT)][..])].map(
                    |(label, vs, buffers)| {
                        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some(label),
                            layout: Some(&pl),
                            vertex: wgpu::VertexState {
                                module: m,
                                entry_point: Some(vs),
                                compilation_options: Default::default(),
                                buffers,
                            },
                            primitive: wgpu::PrimitiveState::default(),
                            depth_stencil: Some(wgpu::DepthStencilState {
                                format: crate::DEPTH_FORMAT,
                                depth_write_enabled: Some(false),
                                depth_compare: Some(wgpu::CompareFunction::Less),
                                stencil: Default::default(),
                                bias: Default::default(),
                            }),
                            multisample: wgpu::MultisampleState::default(),
                            fragment: Some(wgpu::FragmentState {
                                module: m,
                                entry_point: Some("fs_ao_depth"),
                                compilation_options: Default::default(),
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: crate::post::LINDEPTH_FORMAT,
                                    blend: None,
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                            }),
                            multiview_mask: None,
                            cache: crate::pipeline_cache::of(&device).as_ref(),
                        })
                    },
                )
            })
        };
        let pipeline = crate::deferred_hdr(device, &pl, module, move |layout, module| {
            crate::HdrPipeline {
                alpha_to_coverage: false,
                label: "water",
                layout,
                module,
                vs: "vs",
                fs: "fs",
                buffers: &[],
                cull: None,
                depth_write: true,
                depth_compare: wgpu::CompareFunction::Less,
                blend: None,
                samples,
            }
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // The ripples' normal map: tiled, trilinear and 16x anisotropic (three.js water normals are RepeatWrapping
        // with mips): the clamped, mip-less water sampler blocked them up at grazing angles and seamed every tile.
        let ripple_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water ripples"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        let flat = height_texture(device, 1).1;
        WaterRenderer {
            ripple_sampler,
            layout,
            pipeline,
            mesh_pipeline,
            ao_depth_pipelines,
            sampler,
            flat,
            slots: Vec::new(),
        }
    }

    /// Writes this frame's uniforms and height fields and makes the bind
    /// groups (the scene copy and the depth prepass are this frame's;
    /// `planar`: the surfaces drawn with this frame's planar reflections and
    /// their textures, `fallback` bound where there is none).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare<'t>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        waters: &[Water],
        screen: (u32, u32),
        ssr: bool,
        scene_color: &wgpu::TextureView,
        scene_depth: &wgpu::TextureView,
        planar: &[(usize, &wgpu::TextureView)],
        ao: &wgpu::TextureView,
        fallback: &wgpu::TextureView,
        mesh_buffers: &dyn Fn(crate::scene::MeshHandle) -> Option<(wgpu::Buffer, wgpu::Buffer, u32)>,
        textures: &dyn Fn(crate::TextureHandle) -> Option<&'t wgpu::TextureView>,
    ) -> Vec<WaterDraw> {
        while self.slots.len() < waters.len() {
            self.slots.push(Slot {
                uniform: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("water"),
                    size: std::mem::size_of::<WaterGpu>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                heights: None,
            });
        }
        let (w, h) = (screen.0 as f32, screen.1 as f32);
        let mut draws = Vec::with_capacity(waters.len());
        for (index, (water, slot)) in waters.iter().zip(&mut self.slots).enumerate() {
            let reflection = planar.iter().find(|(i, _)| *i == index).map(|(_, v)| *v);
            let segments = grid_segments(water);
            let mesh = water.mesh.and_then(mesh_buffers);
            let field = height_field(water).filter(|_| mesh.is_none());
            if let Some(f) = field {
                if slot
                    .heights
                    .as_ref()
                    .is_none_or(|(r, ..)| *r != f.resolution)
                {
                    let (t, v) = height_texture(device, f.resolution);
                    slot.heights = Some((f.resolution, t, v));
                }
                let (_, texture, _) = slot.heights.as_ref().expect("made above");
                queue.write_texture(
                    texture.as_image_copy(),
                    bytemuck::cast_slice(&f.heights),
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(f.resolution * 4),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: f.resolution,
                        height: f.resolution,
                        depth_or_array_layers: 1,
                    },
                );
            }
            let normal_view = water.normal_map.and_then(textures);
            let gpu = WaterGpu {
                model: water.transform.to_cols_array_2d(),
                size: [
                    water.size.x.max(0.0),
                    water.size.y.max(0.0),
                    segments as f32,
                    f32::from(field.is_some()),
                ],
                shallow: water.shallow.extend(water.depth_fade.max(1e-3)).to_array(),
                deep: water.deep.extend(water.opacity.clamp(0.0, 1.0)).to_array(),
                foam: water.foam.extend(water.foam_width.max(0.0)).to_array(),
                waves: [
                    water.wave_scale.max(0.0),
                    water.wave_speed,
                    water.wave_strength.max(0.0),
                    water.refraction.max(0.0),
                ],
                look: [
                    water.reflectivity.max(0.0),
                    f32::from(ssr),
                    field.map_or(1.0, |f| f.size),
                    field.map_or(2.0, |f| f.resolution as f32),
                ],
                screen: [w, h, 1.0 / w.max(1.0), 1.0 / h.max(1.0)],
                extra: [
                    water.shadows.clamp(0.0, 1.0),
                    f32::from(reflection.is_some()),
                    water.f0.clamp(0.0, 1.0),
                    water.reflection_bias.clamp(0.0, 1.0),
                ],
                under: water.underwater_color.extend(water.glint.max(0.0)).to_array(),
                normals: [
                    water.normal_scale.max(0.0),
                    water.flow.x,
                    water.flow.y,
                    f32::from(normal_view.is_some()),
                ],
            };
            queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&gpu));
            let heights = match (&slot.heights, field) {
                (Some((_, _, v)), Some(_)) => v,
                _ => &self.flat,
            };
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("water"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: slot.uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(heights),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(scene_color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(scene_depth),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(
                            reflection.unwrap_or(fallback),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(normal_view.unwrap_or(fallback)),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&self.ripple_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::TextureView(ao),
                    },
                ],
            });
            draws.push(WaterDraw {
                group,
                vertices: segments * segments * 6,
                mesh,
            });
        }
        draws
    }

    /// Draws the surfaces' view depth for the screen AO (`fs_ao_depth`; group 0: the frame). Returns the draw calls.
    pub(crate) fn draw_ao_depth(&self, pass: &mut wgpu::RenderPass, draws: &[WaterDraw]) -> u32 {
        let [grid, mesh] = self.ao_depth_pipelines.get();
        for d in draws {
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(mesh);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(grid);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        draws.len() as u32
    }

    /// Draws the surfaces (group 0: the frame). Returns the draw calls.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass, draws: &[WaterDraw]) -> u32 {
        for d in draws {
            pass.set_bind_group(1, &d.group, &[]);
            match &d.mesh {
                Some((vertices, indices, count)) => {
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_vertex_buffer(0, vertices.slice(..));
                    pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                None => {
                    pass.set_pipeline(&self.pipeline);
                    pass.draw(0..d.vertices, 0..1);
                }
            }
        }
        draws.len() as u32
    }
}

/// The surface's plane in world space: (unit normal facing up out of the
/// water, a point on it).
pub(crate) fn plane(water: &Water) -> (Vec3, Vec3) {
    let up = water
        .transform
        .transform_vector3(Vec3::Y)
        .normalize_or(Vec3::Y);
    (up, water.transform.transform_point3(Vec3::ZERO))
}

/// The surface as the screen AO and the planar reflection see it: its plane (normal, a point) and its world xz extent.
/// A surface over a mesh ([`Water::mesh`], `mesh` its data) lies at the mean height of the vertices it draws (its
/// indices; the entity's own origin can be anywhere, e.g. a mesh in world coordinates under an entity at 0) and spans
/// their extent; a grid spans its size around the entity.
pub(crate) fn surface(water: &Water, mesh: Option<&crate::scene::MeshData>) -> (Vec3, Vec3, Vec2, Vec2) {
    let (up, origin) = plane(water);
    if let Some(m) = mesh.filter(|m| !m.indices.is_empty()) {
        let (mut sum, mut lo, mut hi) = (Vec3::ZERO, Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
        let mut n = 0.0;
        for &i in &m.indices {
            let Some(v) = m.vertices.get(i as usize) else { continue };
            let p = Vec3::from(v.position);
            sum += p;
            n += 1.0;
            let w = water.transform.transform_point3(p);
            lo = lo.min(Vec2::new(w.x, w.z));
            hi = hi.max(Vec2::new(w.x, w.z));
        }
        if n > 0.0 {
            return (up, water.transform.transform_point3(sum / n), lo, hi);
        }
    }
    let h = water.size * 0.5;
    let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for c in [Vec3::new(-h.x, 0.0, -h.y), Vec3::new(h.x, 0.0, -h.y), Vec3::new(-h.x, 0.0, h.y), Vec3::new(h.x, 0.0, h.y)] {
        let w = water.transform.transform_point3(c);
        lo = lo.min(Vec2::new(w.x, w.z));
        hi = hi.max(Vec2::new(w.x, w.z));
    }
    (up, origin, lo, hi)
}

/// The surface's height field when it is one the renderer uses: 2 to
/// [`MAX_RESOLUTION`] heights per side, all of them, over a positive size.
fn height_field(water: &Water) -> Option<&WaterHeights> {
    water.heights.as_ref().filter(|f| {
        f.resolution >= 2
            && f.resolution <= MAX_RESOLUTION
            && f.heights.len() == (f.resolution * f.resolution) as usize
            && f.size > 0.0
    })
}

/// The surface's height above its rest plane at local xz (the height
/// field, bilinear; 0 without one).
fn local_height(water: &Water, local: Vec2) -> f32 {
    let Some(f) = height_field(water) else {
        return 0.0;
    };
    let n = f.resolution as usize;
    let cell = f.size / (n - 1) as f32;
    let g =
        ((local + Vec2::splat(f.size * 0.5)) / cell).clamp(Vec2::ZERO, Vec2::splat((n - 1) as f32));
    let i = g.floor().min(Vec2::splat((n - 2) as f32));
    let t = g - i;
    let (x, z) = (i.x as usize, i.y as usize);
    let at = |x: usize, z: usize| f.heights[z * n + x];
    let lo = at(x, z) + (at(x + 1, z) - at(x, z)) * t.x;
    let hi = at(x, z + 1) + (at(x + 1, z + 1) - at(x, z + 1)) * t.x;
    lo + (hi - lo) * t.y
}

/// The surface the camera at `eye` is below (inside its extent, under its
/// waves), if any, among those with an underwater view.
pub(crate) fn underwater(waters: &[Water], eye: Vec3) -> Option<usize> {
    waters.iter().position(|w| {
        if w.underwater_density.is_nan() || w.underwater_density <= 0.0 {
            return false;
        }
        let local = w.transform.inverse().transform_point3(eye);
        local.x.abs() <= w.size.x * 0.5
            && local.z.abs() <= w.size.y * 0.5
            && local.y < local_height(w, Vec2::new(local.x, local.z))
    })
}

/// The fog of the underwater view: the water's colour, thickening by its
/// density from the eye, the sky left clear (Snell's window shows it).
pub(crate) fn underwater_fog(w: &Water) -> crate::Fog {
    crate::Fog {
        color: w.underwater_color,
        density: w.underwater_density,
        start: 0.0,
        end: 0.0,
        max_opacity: 1.0,
        height_density: 0.0,
        sky: 0.0,
        sun_color: w.underwater_color,
        sun_power: 8.0,
        ..Default::default()
    }
}

/// The frame's caustics: the first surface that has them, as
/// `FrameUniform::caustics` (height, strength, scale, fade; then the xz
/// rectangle). Zero without one.
pub(crate) fn caustics(waters: &[Water]) -> [[f32; 4]; 2] {
    let Some(w) = waters.iter().find(|w| w.caustics > 0.0) else {
        return [[0.0; 4]; 2];
    };
    let (_, at) = plane(w);
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for (x, z) in [(-0.5f32, -0.5f32), (0.5, -0.5), (-0.5, 0.5), (0.5, 0.5)] {
        let p = w
            .transform
            .transform_point3(Vec3::new(x * w.size.x, 0.0, z * w.size.y));
        lo = lo.min(Vec2::new(p.x, p.z));
        hi = hi.max(Vec2::new(p.x, p.z));
    }
    [
        [at.y, w.caustics, w.caustics_scale.max(1e-3), 0.15],
        [lo.x, lo.y, hi.x, hi.y],
    ]
}

/// The camera mirrored in the plane through `at` with unit normal `n`.
pub(crate) fn mirrored_camera(camera: &crate::Camera, n: Vec3, at: Vec3) -> crate::Camera {
    let point = |p: Vec3| p - n * (2.0 * (p - at).dot(n));
    let dir = |d: Vec3| d - n * (2.0 * d.dot(n));
    crate::Camera {
        position: point(camera.position),
        target: point(camera.target),
        up: dir(camera.up),
        ..*camera
    }
}

/// `proj` with its near plane replaced by the world plane (`n`, `at`)
/// seen through `view` (Lengyel's oblique near-plane clipping, for depth
/// 0..1): what lies below the plane is clipped and the far plane is
/// tilted to keep the depth range. `lift` raises the plane (m) to hide
/// the seam where geometry meets the water.
pub(crate) fn oblique(proj: Mat4, view: Mat4, n: Vec3, at: Vec3, lift: f32) -> Mat4 {
    let world = n.extend(-(n.dot(at) + lift));
    // Planes transform by the inverse transpose.
    let c = view.inverse().transpose() * world;
    let q = proj.inverse() * Vec4::new(c.x.signum(), c.y.signum(), 1.0, 1.0);
    let c = c * (1.0 / c.dot(q));
    let mut rows = [proj.row(0), proj.row(1), c, proj.row(3)];
    if !rows[2].is_finite() {
        rows[2] = proj.row(2);
    }
    Mat4::from_cols(rows[0], rows[1], rows[2], rows[3]).transpose()
}

fn height_texture(device: &wgpu::Device, n: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("water heights"),
        size: wgpu::Extent3d {
            width: n,
            height: n,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let v = t.create_view(&Default::default());
    (t, v)
}

/// Appends the strips of the 2D water bodies to `frame` as runs in the
/// sprite order: the body (a gradient from `top` at the surface to
/// `bottom`) and the lip along the surface (soft across its width).
pub(crate) fn water_2d_strips(waters: &[Water2d], frame: &mut crate::ribbons::RibbonFrame) {
    use crate::ribbons::{RibbonRun, RibbonVertex};
    for w in waters {
        let n = w.surface.len();
        if n < 2 || w.width.is_nan() || w.depth.is_nan() || w.width <= 0.0 || w.depth <= 0.0 {
            continue;
        }
        let first = frame.vertices.len() as u32;
        let at = |i: usize| {
            let x = w.origin.x + w.width * i as f32 / (n - 1) as f32;
            let h = w.surface[i];
            (x, w.origin.y + if h.is_finite() { h } else { 0.0 })
        };
        let bottom_y = w.origin.y - w.depth;
        let top = w.top.to_array();
        let bottom = w.bottom.to_array();
        let lip = w.lip.to_array();
        let half = w.lip_width.max(0.0) * 0.5;
        for i in 0..n - 1 {
            let (x0, y0) = at(i);
            let (x1, y1) = at(i + 1);
            let v = |x: f32, y: f32| glam::Vec3::new(x, y, w.z);
            // Body: two triangles from the surface down.
            let (a, b, c, d) = (v(x0, y0), v(x1, y1), v(x0, bottom_y), v(x1, bottom_y));
            for (p, col) in [
                (a, top),
                (c, bottom),
                (b, top),
                (b, top),
                (c, bottom),
                (d, bottom),
            ] {
                frame
                    .vertices
                    .push(RibbonVertex::new(p, 0.5, 0.5, col, true));
            }
            if half > 0.0 {
                let (a, b) = (v(x0, y0 + half), v(x1, y1 + half));
                let (c, d) = (v(x0, y0 - half), v(x1, y1 - half));
                for (p, vv) in [(a, 0.0), (c, 1.0), (b, 0.0), (b, 0.0), (c, 1.0), (d, 1.0)] {
                    frame
                        .vertices
                        .push(RibbonVertex::new(p, 0.5, vv, lip, false));
                }
            }
        }
        frame.runs.push(RibbonRun {
            texture: None,
            blend: crate::ParticleBlend::Alpha,
            layer: Some(w.layer),
            first,
            count: frame.vertices.len() as u32 - first,
        });
    }
}

/// The water module after the frame group and the lighting.
pub(crate) fn water_source(frame_wgsl: &str, lighting_wgsl: &str) -> String {
    format!("{frame_wgsl}{lighting_wgsl}{}", wgsl!("water.wgsl"))
}
