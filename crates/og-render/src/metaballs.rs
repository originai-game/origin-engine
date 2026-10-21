//! 2D fluid drawn as metaballs (the E5 round; `shaders/metaballs.wgsl`).
//! [`Metaballs`]: the particles of a fluid (og-physics' `og.Fluid2D`) are
//! added into one channel of a field target as soft kernels before the
//! main pass, then composited in the sprite order at the fluid's layer by a
//! threshold over that channel.
//!
//! Fluids of one `group` (phase 2) are one surface: each takes the channel
//! of its `lane` in a run of the field's four, a particle adds its kernel
//! into its lanes' channels by its `mix`, and the group is composited once,
//! thresholded on the sum of its channels and coloured by their weights, so
//! where two fluids blend the colour blends. Fields hold four channels
//! each; groups (and lone fluids) are packed into [`FIELDS`] of them, so up
//! to [`MAX_FLUIDS`] fluids are drawn a frame; more are not drawn.

use crate::deferred::{AsModule, Deferred};
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// Field targets a frame can use (four channels each).
pub const FIELDS: usize = 4;

/// The most fluids drawn in a frame (one field channel each).
pub const MAX_FLUIDS: usize = FIELDS * 4;

/// A 2D fluid drawn as metaballs. See the module documentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Metaballs {
    /// Particle positions in world units.
    pub particles: Arc<[Vec2]>,
    /// The radius of each particle's kernel.
    pub radius: f32,
    /// Where the surface is: the field value (each kernel peaks at 1).
    pub threshold: f32,
    /// Linear rgba of the body.
    pub color: Vec4,
    /// Linear colour of the bright rim inside the edge, and its width in
    /// field units (0: none).
    pub rim: Vec3,
    pub rim_width: f32,
    /// The sprite layer it is drawn in.
    pub layer: i32,
    /// World z of the particles.
    pub z: f32,
    /// Fluids sharing a group above 0 are one surface (the threshold, rim
    /// and layer of its lowest lane's fluid; each one's colour); 0 draws it
    /// alone.
    pub group: u32,
    /// Its lane in the group (0..4): the channel it adds to, from the run of
    /// four its group takes.
    pub lane: u32,
    /// Per particle, how much of it is each lane's fluid (summing to 1);
    /// empty: every particle is all this fluid.
    pub mix: Arc<[[f32; 4]]>,
}

impl Default for Metaballs {
    fn default() -> Self {
        Metaballs {
            particles: Arc::from(Vec::new()),
            radius: 0.3,
            threshold: 0.45,
            color: Vec4::new(0.12, 0.45, 0.85, 0.9),
            rim: Vec3::new(0.7, 0.92, 1.0),
            rim_width: 0.35,
            layer: 0,
            z: 0.0,
            group: 0,
            lane: 0,
            mix: Arc::from(Vec::new()),
        }
    }
}

/// One surface to composite: a lone fluid or a group, its field and the
/// first channel of its run.
#[derive(Clone, Debug, PartialEq)]
struct Surface {
    /// Indices into the frame's fluids, by lane (`None`: no fluid in that lane).
    lanes: [Option<usize>; 4],
    field: usize,
    first: usize,
    width: usize,
}

/// Packs the frame's fluids into surfaces and fields: in order of first
/// appearance, a group's run as wide as its highest lane, a new field when
/// the current one has no room; what does not fit in [`FIELDS`] is left out.
fn pack(fluids: &[Metaballs]) -> Vec<Surface> {
    let mut surfaces: Vec<Surface> = Vec::new();
    let mut by_group: Vec<(u32, usize)> = Vec::new();
    let (mut field, mut used) = (0usize, 0usize);
    for (i, f) in fluids.iter().enumerate() {
        let lane = if f.group == 0 {
            0
        } else {
            (f.lane as usize).min(3)
        };
        if f.group != 0
            && let Some(&(_, s)) = by_group.iter().find(|(g, _)| *g == f.group)
        {
            let surface = &mut surfaces[s];
            if lane < surface.width && surface.lanes[lane].is_none() {
                surface.lanes[lane] = Some(i);
            }
            continue;
        }
        // A new surface: a group reserves its four lanes' run (its members
        // can appear in any order), a lone fluid one channel.
        let width = if f.group == 0 { 1 } else { 4 };
        if used + width > 4 {
            field += 1;
            used = 0;
        }
        if field >= FIELDS {
            break;
        }
        let mut lanes = [None; 4];
        lanes[lane] = Some(i);
        surfaces.push(Surface {
            lanes,
            field,
            first: used,
            width,
        });
        if f.group != 0 {
            by_group.push((f.group, surfaces.len() - 1));
        }
        used += width;
    }
    surfaces
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Blob {
    position_radius: [f32; 4],
    channel: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FluidGpu {
    /// Per channel of the field, the colour of the fluid in it.
    colors: [[f32; 4]; 4],
    rim: [f32; 4],
    /// The surface's channels (1 each).
    channel: [f32; 4],
    /// x: threshold, y: 1/w, z: 1/h, w: 1 when one fluid (its colour is colors[0]).
    params: [f32; 4],
}

const SLOT: u64 = 256;

const BLOB_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Blob>() as u64,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
};

pub(crate) struct MetaballRenderer {
    field_pipeline: Deferred<wgpu::RenderPipeline>,
    composite_pipeline: Deferred<wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    blobs: wgpu::Buffer,
    params: wgpu::Buffer,
    /// Per field, its blobs' range in `blobs`.
    ranges: Vec<std::ops::Range<u32>>,
    /// Per surface (what [`layers`] numbers), its field.
    fields_of: Vec<usize>,
}

impl MetaballRenderer {
    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub(crate) fn warm(&self) -> bool {
        crate::deferred::warm_one(&[&self.field_pipeline, &self.composite_pipeline])
    }

    pub(crate) fn new(
        device: &wgpu::Device,
        frame_layout: &wgpu::BindGroupLayout,
        frame_wgsl: &str,
        samples: u32,
    ) -> MetaballRenderer {
        // Made when a frame first draws metaballs (deferred.rs).
        let module = crate::deferred::module(device, "metaballs", metaballs_source(frame_wgsl));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("metaballs"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<FluidGpu>() as u64
                        ),
                    },
                    count: None,
                },
            ],
        });
        let field_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("metaball field"),
            bind_group_layouts: &[Some(frame_layout)],
            immediate_size: 0,
        });
        let field_pipeline = {
            let (device, field_layout, module) =
                (device.clone(), field_layout.clone(), module.clone());
            Deferred::new(move || {
                let module = module.module();
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("metaball field"),
                    layout: Some(&field_layout),
                    vertex: wgpu::VertexState {
                        module,
                        entry_point: Some("vs_field"),
                        compilation_options: Default::default(),
                        buffers: &[Some(BLOB_LAYOUT)],
                    },
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(wgpu::FragmentState {
                        module,
                        entry_point: Some("fs_field"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: crate::HDR_FORMAT,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                                alpha: wgpu::BlendComponent {
                                    src_factor: wgpu::BlendFactor::One,
                                    dst_factor: wgpu::BlendFactor::One,
                                    operation: wgpu::BlendOperation::Add,
                                },
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("metaball composite"),
            bind_group_layouts: &[Some(frame_layout), Some(&layout)],
            immediate_size: 0,
        });
        let composite_pipeline =
            crate::deferred_hdr(device, &composite_layout, module, move |layout, module| {
                crate::HdrPipeline {
                    label: "metaball composite",
                    layout,
                    module,
                    vs: "vs_composite",
                    fs: "fs_composite",
                    buffers: &[],
                    cull: None,
                    depth_write: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    samples,
                }
            });
        let blobs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metaballs"),
            size: 4096,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("metaball params"),
            size: SLOT * MAX_FLUIDS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        MetaballRenderer {
            field_pipeline,
            composite_pipeline,
            layout,
            blobs,
            params,
            ranges: Vec::new(),
            fields_of: Vec::new(),
        }
    }

    /// Writes the frame's kernels and surface parameters. Returns the
    /// number of field targets the frame needs (0: no fluid).
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        fluids: &[Metaballs],
        screen: (u32, u32),
    ) -> usize {
        let surfaces = pack(fluids);
        let fields = surfaces.iter().map(|s| s.field + 1).max().unwrap_or(0);
        let mut blobs = Vec::new();
        self.ranges.clear();
        self.fields_of.clear();
        for field in 0..fields {
            let start = blobs.len() as u32;
            for s in surfaces.iter().filter(|s| s.field == field) {
                for (lane, &i) in s.lanes.iter().enumerate() {
                    let Some(i) = i else { continue };
                    let f = &fluids[i];
                    let radius = f.radius.max(1e-4);
                    let mut own = [0.0; 4];
                    own[s.first + lane] = 1.0;
                    for (k, p) in f.particles.iter().enumerate() {
                        if !p.is_finite() {
                            continue;
                        }
                        let channel = match f.mix.get(k) {
                            Some(m) if s.width == 4 => {
                                let mut c = [0.0; 4];
                                c.copy_from_slice(m);
                                c
                            }
                            _ => own,
                        };
                        blobs.push(Blob {
                            position_radius: [p.x, p.y, f.z, radius],
                            channel,
                        });
                    }
                }
            }
            self.ranges.push(start..blobs.len() as u32);
        }
        for (k, s) in surfaces.iter().enumerate() {
            let lead = s.lanes.iter().flatten().next().map(|&i| &fluids[i]);
            let Some(lead) = lead else { continue };
            let mut colors = [[0.0; 4]; 4];
            let mut channel = [0.0; 4];
            let single = s.width == 1;
            for (lane, i) in s.lanes.iter().enumerate().take(s.width) {
                channel[s.first + lane] = 1.0;
                if let Some(&i) = i.as_ref() {
                    colors[if single { 0 } else { s.first + lane }] = fluids[i].color.to_array();
                }
            }
            let gpu = FluidGpu {
                colors,
                rim: lead.rim.extend(lead.rim_width.max(0.0)).to_array(),
                channel,
                params: [
                    lead.threshold.max(1e-3),
                    1.0 / screen.0.max(1) as f32,
                    1.0 / screen.1.max(1) as f32,
                    if single { 1.0 } else { 0.0 },
                ],
            };
            queue.write_buffer(&self.params, k as u64 * SLOT, bytemuck::bytes_of(&gpu));
            self.fields_of.push(s.field);
        }
        let bytes: &[u8] = bytemuck::cast_slice(&blobs);
        if bytes.len() as u64 > self.blobs.size() {
            self.blobs = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("metaballs"),
                size: (bytes.len() as u64).next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !bytes.is_empty() {
            queue.write_buffer(&self.blobs, 0, bytes);
        }
        fields
    }

    /// The field passes: every kernel into its field (`fields[k]` is field
    /// `k`'s target).
    pub(crate) fn encode_field(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        fields: &[&wgpu::TextureView],
        frame_group: &wgpu::BindGroup,
    ) {
        for (field, range) in fields.iter().zip(&self.ranges) {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("metaball field"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: field,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if range.is_empty() {
                continue;
            }
            pass.set_pipeline(&self.field_pipeline);
            pass.set_bind_group(0, frame_group, &[]);
            pass.set_vertex_buffer(0, self.blobs.slice(..));
            pass.draw(0..6, range.clone());
        }
    }

    /// The field surface `surface` composites from.
    pub(crate) fn field_of(&self, surface: u8) -> Option<usize> {
        self.fields_of.get(surface as usize).copied()
    }

    /// The bind group the composite reads the field through.
    pub(crate) fn group(
        &self,
        device: &wgpu::Device,
        field: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("metaballs"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(field),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.params,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<FluidGpu>() as u64),
                    }),
                },
            ],
        })
    }

    /// Composites surface `i` (group 0: the frame) from its field's group.
    pub(crate) fn composite(&self, pass: &mut wgpu::RenderPass, group: &wgpu::BindGroup, i: u8) {
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(1, group, &[(i as u64 * SLOT) as u32]);
        pass.draw(0..3, 0..1);
    }
}

/// Where each surface (a lone fluid or a group, numbered as composited)
/// sits in the sprite order: its lowest lane's fluid's layer.
pub(crate) fn layers(fluids: &[Metaballs]) -> Vec<(usize, i32)> {
    pack(fluids)
        .iter()
        .enumerate()
        .filter_map(|(k, s)| {
            let lead = s.lanes.iter().flatten().next()?;
            Some((k, fluids[*lead].layer))
        })
        .collect()
}

/// The metaballs' module after the frame group, as the renderer compiles it.
pub(crate) fn metaballs_source(frame_wgsl: &str) -> String {
    format!("{frame_wgsl}{}", wgsl!("metaballs.wgsl"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fluid(group: u32, lane: u32, layer: i32) -> Metaballs {
        Metaballs {
            group,
            lane,
            layer,
            ..Metaballs::default()
        }
    }

    #[test]
    fn fluids_pack_into_fields_of_four_channels_and_groups_stay_together() {
        // Five lone fluids (four fill the first field, the fifth starts the
        // second), a group of two (lanes 1 and 0, out of order) that takes a
        // field's whole run of four (the third), and one more lone fluid (in
        // a fourth: packing only goes forward).
        let fluids = [
            fluid(0, 0, 1),
            fluid(0, 0, 2),
            fluid(0, 0, 3),
            fluid(0, 0, 4),
            fluid(0, 0, 5),
            fluid(7, 1, 6),
            fluid(7, 0, 9),
            fluid(0, 0, 7),
        ];
        let s = pack(&fluids);
        let fields: Vec<(usize, usize, usize)> =
            s.iter().map(|s| (s.field, s.first, s.width)).collect();
        assert_eq!(
            fields,
            [
                (0, 0, 1),
                (0, 1, 1),
                (0, 2, 1),
                (0, 3, 1),
                (1, 0, 1),
                (2, 0, 4),
                (3, 0, 1)
            ]
        );
        assert_eq!(s[5].lanes, [Some(6), Some(5), None, None]);
        // The group is drawn once, at the layer of its lane 0 fluid.
        let l = layers(&fluids);
        assert_eq!(l.len(), 7);
        assert_eq!(l[5], (5, 9));
        // Past the fields' room, fluids are left out.
        let many: Vec<Metaballs> = (0..20).map(|i| fluid(0, 0, i)).collect();
        assert_eq!(pack(&many).len(), MAX_FLUIDS);
    }
}
