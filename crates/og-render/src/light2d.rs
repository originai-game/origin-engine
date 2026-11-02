//! The 2D light map: ambient plus every [`Light2d`], shadowed by
//! [`Occluder2d`]s with soft penumbrae (`shaders/light2d.wgsl`). Lit sprite
//! layers multiply their colour with it in the sprite shader.
//!
//! Shadows: each shadow-casting light has a row of a polar shadow map
//! (`shaders/light2d_shadow.wgsl`, [`Lighting2d::shadow_angles`] texels
//! wide), traced once a frame against the edges of the occluders on its
//! layers that reach its radius ([`shadow_edges`]): rectangles and
//! polygons alike, concave ones too, at a cost per light-map pixel that
//! does not grow with the occluders. A pixel inside the first occluder a
//! ray meets is not shadowed by it.
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
use glam::{Mat4, Vec2, Vec3};
use std::cell::Cell;

pub(crate) const LIGHT2D_WGSL: &str = wgsl!("light2d.wgsl");
pub(crate) const LIGHT2D_SHADOW_WGSL: &str = wgsl!("light2d_shadow.wgsl");

/// The most occluder edges one frame's shadow rows trace, over all lights
/// (an edge reaching two lights counts twice); further ones are dropped.
pub(crate) const MAX_SHADOW_EDGES_2D: usize = 16384;
const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Lights2dUniform {
    view_proj: [[f32; 4]; 4],
    /// x: the shadow map's angles.
    info: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ShadowUniform {
    /// x: angles, y: rows.
    info: [u32; 4],
    /// Per row: the light's centre and radius.
    rows: [[f32; 4]; MAX_LIGHTS_2D],
    /// Per row: its first edge and how many.
    ranges: [[u32; 4]; MAX_LIGHTS_2D],
}

/// One occluder edge a row traces: its ends, and its occluder (`id.x`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ShadowEdge {
    ab: [f32; 4],
    id: [u32; 4],
}

/// What the frame's shadow rows trace: the edges, each light's row (-1
/// for a light that casts none) and each row's light and edge range.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct ShadowEdges {
    pub edges: Vec<ShadowEdge>,
    pub row_of: Vec<i32>,
    pub rows: Vec<([f32; 4], [u32; 4])>,
}

/// Whether `p` is inside the closed polygon `poly` (even-odd).
fn contains(poly: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            inside = !inside;
        }
    }
    inside
}

/// The distance from `p` to the segment ab.
fn to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let s = b - a;
    let t = ((p - a).dot(s) / s.length_squared().max(1e-12)).clamp(0.0, 1.0);
    (a + s * t).distance(p)
}

/// Each shadow-casting light's row and edges: the edges of the occluders
/// on its layers that come within its radius, without an occluder that
/// holds it (as before polar maps, it lights out of a box it sits in).
pub(crate) fn shadow_edges(lights: &[Light2d], occluders: &[Occluder2d]) -> ShadowEdges {
    let outlines: Vec<Vec<Vec2>> = occluders.iter().map(Occluder2d::outline).collect();
    let mut out = ShadowEdges::default();
    for l in lights {
        if !l.cast_shadows || l.kind == Light2dKind::Global {
            out.row_of.push(-1);
            continue;
        }
        let start = out.edges.len() as u32;
        let radius = l.radius.max(1e-3);
        for (id, (o, outline)) in occluders.iter().zip(&outlines).enumerate() {
            if o.layers & l.layers == 0 || contains(outline, l.position) {
                continue;
            }
            for i in 0..outline.len() {
                let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
                if to_segment(l.position, a, b) < radius && out.edges.len() < MAX_SHADOW_EDGES_2D {
                    out.edges.push(ShadowEdge {
                        ab: [a.x, a.y, b.x, b.y],
                        id: [id as u32, 0, 0, 0],
                    });
                }
            }
        }
        out.row_of.push(out.rows.len() as i32);
        out.rows.push((
            [l.position.x, l.position.y, radius, 0.0],
            [start, out.edges.len() as u32 - start, 0, 0],
        ));
    }
    out
}

/// The polar shadow map and its pass.
struct Shadows {
    pipeline: Deferred<wgpu::RenderPipeline>,
    uniform: wgpu::Buffer,
    edges: wgpu::Buffer,
    group: wgpu::BindGroup,
    /// The map at its width (angles), 64 rows.
    map: (u32, wgpu::Texture, wgpu::TextureView),
    rows: u32,
    /// Traced already this frame (the first light map to draw traces it;
    /// views share it).
    traced: Cell<bool>,
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
    /// The same into the light map and its directions (`fs_dir`), for
    /// frames with normal-mapped receivers.
    pipeline_dir: Deferred<wgpu::RenderPipeline>,
    shadows: Shadows,
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
        crate::deferred::warm_one(&[&self.pipeline, &self.shadows.pipeline, &self.pipeline_dir])
    }

    pub fn new(device: &wgpu::Device) -> Light2dRenderer {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lights 2d"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
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
        let shadows = Shadows::new(device);
        let group = light_group(device, &layout, &uniform, &shadows.map.2);
        // Made when a frame first draws 2D lights (deferred.rs).
        let module = crate::deferred::module(device, "lights 2d", LIGHT2D_WGSL.into());
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lights 2d"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let sum = wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        };
        let add = Some(wgpu::BlendState {
            color: sum,
            alpha: wgpu::BlendComponent::REPLACE,
        });
        // The directions add their weight in alpha too.
        let add_all = Some(wgpu::BlendState {
            color: sum,
            alpha: sum,
        });
        let pipeline_with = |fs: &'static str, dir: bool| {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            Deferred::new(move || {
                let module = module.module();
                let target = |blend| {
                    Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                };
                let targets = [target(add), target(add_all)];
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
                        entry_point: Some(fs),
                        compilation_options: Default::default(),
                        targets: &targets[..if dir { 2 } else { 1 }],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Light2dRenderer {
            pipeline: pipeline_with("fs", false),
            pipeline_dir: pipeline_with("fs_dir", true),
            shadows,
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
    /// dropped) for the strips of `slots`, with shadow rows `angles` wide;
    /// returns (lights, occluders) drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        lights: &[Light2d],
        occluders: &[Occluder2d],
        slots: &LightSlots,
        angles: u32,
    ) -> (u32, u32) {
        let angles = angles.clamp(64, 4096);
        if self.shadows.map.0 != angles {
            self.shadows.map = shadow_map(device, angles);
            self.group = light_group(device, &self.layout, &self.uniform, &self.shadows.map.2);
            // The views' groups name the old map: made again by write_view.
            self.views.clear();
        }
        let lights = &lights[..lights.len().min(MAX_LIGHTS_2D)];
        let occ = &occluders[..occluders.len().min(MAX_OCCLUDERS_2D)];
        let mut u: Lights2dUniform = bytemuck::Zeroable::zeroed();
        u.view_proj = view_proj.to_cols_array_2d();
        u.info = [angles as f32, 0.0, 0.0, 0.0];
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        self.written = u;
        let traced = shadow_edges(lights, occ);
        self.shadows.write(queue, &traced, angles);
        let mut instances: Vec<LightInstance> = Vec::new();
        self.ranges.clear();
        for &mask in &slots.0 {
            let start = instances.len() as u32;
            instances.extend(
                lights
                    .iter()
                    .zip(&traced.row_of)
                    .filter(|(l, _)| l.layers & mask != 0)
                    .map(|(l, &row)| instance(l, row)),
            );
            self.ranges.push(start..instances.len() as u32);
        }
        if !instances.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
        }
        self.count = instances.len() as u32;
        (lights.len() as u32, occ.len() as u32)
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
            let group = light_group(device, &self.layout, &uniform, &self.shadows.map.2);
            self.views.push((uniform, group));
        }
        let mut u = self.written;
        u.view_proj = view_proj.to_cols_array_2d();
        queue.write_buffer(&self.views[k].0, 0, bytemuck::bytes_of(&u));
    }

    /// Clears `target` (`size`: its strips side by side) to `ambient`,
    /// and `dir` (the lights' directions, the same size) to zero, and adds
    /// the lights as view `k` sees them ([`Light2dRenderer::write_view`]).
    pub fn encode_view(
        &self,
        encoder: &mut crate::Encoder,
        target: &wgpu::TextureView,
        dir: Option<&wgpu::TextureView>,
        size: (u32, u32),
        ambient: Vec3,
        k: usize,
    ) {
        self.encode_with(encoder, (target, dir), size, ambient, &self.views[k].1);
    }

    /// Clears `target` (`size`: its strips side by side) to `ambient`,
    /// and `dir` to zero, and adds the lights.
    pub fn encode(
        &self,
        encoder: &mut crate::Encoder,
        target: &wgpu::TextureView,
        dir: Option<&wgpu::TextureView>,
        size: (u32, u32),
        ambient: Vec3,
    ) {
        self.encode_with(encoder, (target, dir), size, ambient, &self.group);
    }

    fn encode_with(
        &self,
        encoder: &mut crate::Encoder,
        (target, dir): (&wgpu::TextureView, Option<&wgpu::TextureView>),
        size: (u32, u32),
        ambient: Vec3,
        group: &wgpu::BindGroup,
    ) {
        self.shadows.trace(encoder);
        let attachment = |view, c: Vec3, a: f64| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: c.x as f64,
                        g: c.y as f64,
                        b: c.z as f64,
                        a,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })
        };
        let attachments = [
            attachment(target, ambient, 1.0),
            dir.and_then(|d| attachment(d, Vec3::ZERO, 0.0)),
        ];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lights 2d"),
            color_attachments: &attachments[..1 + dir.is_some() as usize],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(if dir.is_some() {
            &self.pipeline_dir
        } else {
            &self.pipeline
        });
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

/// One light's instance, with its shadow row (-1: none).
fn instance(l: &Light2d, row: i32) -> LightInstance {
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
        direction: [d.x, d.y, l.height.max(0.0), row as f32],
    }
}

/// The lights' bind group: their uniform and the shadow map.
fn light_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    map: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("lights 2d"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(map),
            },
        ],
    })
}

/// A polar shadow map `angles` wide, a row per light.
fn shadow_map(device: &wgpu::Device, angles: u32) -> (u32, wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("lights 2d shadows"),
        size: wgpu::Extent3d {
            width: angles,
            height: MAX_LIGHTS_2D as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SHADOW_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (angles, texture, view)
}

impl Shadows {
    fn new(device: &wgpu::Device) -> Shadows {
        let buffer = |binding: u32, ty: wgpu::BufferBindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lights 2d shadows"),
            entries: &[
                buffer(0, wgpu::BufferBindingType::Uniform),
                buffer(1, wgpu::BufferBindingType::Storage { read_only: true }),
            ],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d shadows"),
            size: std::mem::size_of::<ShadowUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let edges = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights 2d shadow edges"),
            size: (std::mem::size_of::<ShadowEdge>() * MAX_SHADOW_EDGES_2D) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights 2d shadows"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: edges.as_entire_binding(),
                },
            ],
        });
        let module =
            crate::deferred::module(device, "lights 2d shadows", LIGHT2D_SHADOW_WGSL.into());
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lights 2d shadows"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = {
            let device = device.clone();
            Deferred::new(move || {
                let module = module.module();
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("lights 2d shadows"),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some("fs"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: SHADOW_FORMAT,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Shadows {
            pipeline,
            uniform,
            edges,
            group,
            map: shadow_map(device, 1024),
            rows: 0,
            traced: Cell::new(true),
        }
    }

    /// The frame's rows and edges; the next light map traces them.
    fn write(&mut self, queue: &wgpu::Queue, traced: &ShadowEdges, angles: u32) {
        let mut u: ShadowUniform = bytemuck::Zeroable::zeroed();
        u.info = [angles, traced.rows.len() as u32, 0, 0];
        for (i, (row, range)) in traced.rows.iter().enumerate() {
            u.rows[i] = *row;
            u.ranges[i] = *range;
        }
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        if !traced.edges.is_empty() {
            queue.write_buffer(&self.edges, 0, bytemuck::cast_slice(&traced.edges));
        }
        self.rows = traced.rows.len() as u32;
        self.traced.set(self.rows == 0);
    }

    /// Traces the rows into the map, once a frame.
    fn trace(&self, encoder: &mut crate::Encoder) {
        if self.traced.replace(true) {
            return;
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lights 2d shadows"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.map.2,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1e20,
                        g: 1e20,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.group, &[]);
        pass.set_viewport(0.0, 0.0, self.map.0 as f32, self.rows as f32, 0.0, 1.0);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(x: f32, layers: u32) -> Occluder2d {
        Occluder2d {
            center: Vec2::new(x, 0.0),
            half_size: Vec2::new(0.5, 1.0),
            layers,
            ..Default::default()
        }
    }

    #[test]
    fn each_shadow_row_traces_the_edges_on_its_layers_within_its_radius() {
        let lights = [
            Light2d {
                radius: 3.0,
                layers: 1,
                ..Default::default()
            },
            Light2d {
                cast_shadows: false,
                ..Default::default()
            },
            Light2d {
                kind: Light2dKind::Global,
                ..Default::default()
            },
            Light2d {
                position: Vec2::new(10.0, 0.0),
                radius: 3.0,
                layers: 2,
                ..Default::default()
            },
        ];
        // Near the first light on its layer; near it on another layer; far
        // from both; holding the second shadow-casting light.
        let occ = [
            wall(2.0, 1),
            wall(-2.0, 2),
            wall(30.0, u32::MAX),
            wall(10.0, 2),
        ];
        let t = shadow_edges(&lights, &occ);
        assert_eq!(t.row_of, [0, -1, -1, 1]);
        assert_eq!(t.rows.len(), 2);
        // The near wall's four edges, all within 3 of the light at the origin
        // (its far side, x = 2.5, is 2.5 away).
        assert_eq!(t.rows[0].1[..2], [0, 4]);
        assert!(t.edges[..4].iter().all(|e| e.id[0] == 0));
        // The second light sits inside the wall that alone could shadow it.
        assert_eq!(t.rows[1].1[..2], [4, 0]);
    }

    #[test]
    fn a_polygon_outline_is_its_points_turned_about_its_centre() {
        let tri = [
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(-1.0, -1.0),
        ];
        let o = Occluder2d::polygon(Vec2::new(5.0, 2.0), std::f32::consts::FRAC_PI_2, &tri);
        let got = o.outline();
        let want = [
            Vec2::new(5.0, 3.0),
            Vec2::new(4.0, 2.0),
            Vec2::new(6.0, 1.0),
        ];
        for (g, w) in got.iter().zip(want) {
            assert!(g.distance(w) < 1e-6, "{got:?}");
        }
        assert_eq!(got.len(), 3);
        // Two points are no polygon: the rectangle stays.
        assert_eq!(
            Occluder2d::polygon(Vec2::ZERO, 0.0, &tri[..2])
                .outline()
                .len(),
            4
        );
    }
}
