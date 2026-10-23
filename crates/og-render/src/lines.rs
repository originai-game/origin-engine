//! Lines (three.js `Line`, `LineSegments`, `LineLoop` with a
//! `LineBasicMaterial` or `LineDashedMaterial`) and the pass that draws
//! them with the point clouds (`points.rs`) and the wireframe materials.
//!
//! Every segment and point is one instance of a screen-space quad
//! (`shaders/lines.wgsl`): a width in CSS pixels stays that wide at any
//! distance, a world width shrinks with it, and the edge is feathered by a
//! pixel on every tier (WebGL2 has no wide lines, WebGPU no line width at
//! all). Sets draw in one of two places ([`LineLayer`]):
//!
//! - **Scene**: in the main pass after the transparent meshes, particles
//!   and strips, into the HDR target (exposed, bloomed and tone mapped with
//!   the scene), depth tested or drawn over it (`depth_test`).
//! - **Overlay**: over the finished frame after every post effect and
//!   before the UI overlay, never depth tested, colours as given: debug
//!   draw and screen lines, which read the same in every capture.
//!
//! A material with `wireframe` draws its instances' triangle edges here
//! (scene, depth tested, one CSS pixel) instead of their triangles: the
//! unique edges of a mesh are found once per mesh revision.

use crate::deferred::Deferred;
use crate::scene::{MeshData, RenderScene};
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// How the points of a [`Lines`] join (three.js's three line objects).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineMode {
    /// One polyline through every point (three.js `Line`).
    #[default]
    Strip,
    /// Pairs of points, each its own segment (three.js `LineSegments`).
    Segments,
    /// A polyline closed back to its first point (three.js `LineLoop`).
    Loop,
}

/// Where a line or point set draws (module docs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineLayer {
    #[default]
    Scene,
    Overlay,
}

/// One set of lines, drawn this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Lines {
    /// The points, in the set's space (`transform`), or CSS pixels from
    /// the top left with y down when `screen`.
    pub points: Vec<Vec3>,
    /// Per point linear RGBA, blended along each segment; empty: `color`.
    pub colors: Vec<[f32; 4]>,
    pub color: [f32; 4],
    pub mode: LineMode,
    /// CSS pixels, or world units with `world_width`.
    pub width: f32,
    pub world_width: bool,
    /// Hidden behind nearer geometry (scene sets only).
    pub depth_test: bool,
    pub screen: bool,
    /// Dashes as (dash, gap) along the line, in world units (CSS pixels on
    /// screen), counted from the first point (three.js `LineDashedMaterial`);
    /// `None` is solid.
    pub dashed: Option<(f32, f32)>,
    pub layer: LineLayer,
    pub transform: Mat4,
}

impl Default for Lines {
    fn default() -> Self {
        Lines {
            points: Vec::new(),
            colors: Vec::new(),
            color: [1.0; 4],
            mode: LineMode::Strip,
            width: 1.0,
            world_width: false,
            depth_test: true,
            screen: false,
            dashed: None,
            layer: LineLayer::Scene,
            transform: Mat4::IDENTITY,
        }
    }
}

pub(crate) const FLAG_WORLD: u32 = 1;
pub(crate) const FLAG_POINT: u32 = 2;
pub(crate) const FLAG_SCREEN: u32 = 4;

/// `struct LineIn` in lines.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LineInstance {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
    pub params: [f32; 4],
}

pub(crate) const LINE_STRIDE: u64 = std::mem::size_of::<LineInstance>() as u64;

const LINE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: LINE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4],
};

impl Lines {
    pub(crate) fn layer(&self) -> LineLayer {
        if self.screen {
            LineLayer::Overlay
        } else {
            self.layer
        }
    }

    /// The set's segments as instances, with the distance along the line
    /// at each end (after the transform).
    pub(crate) fn instances(&self, out: &mut Vec<LineInstance>) {
        let pts: Vec<Vec3> = if self.screen {
            self.points.clone()
        } else {
            self.points
                .iter()
                .map(|&p| self.transform.transform_point3(p))
                .collect()
        };
        let n = pts.len();
        let color = |i: usize| self.colors.get(i).copied().unwrap_or(self.color);
        let mut flags = 0;
        if self.screen {
            flags |= FLAG_SCREEN;
        } else if self.world_width {
            flags |= FLAG_WORLD;
        }
        let (dash, gap) = self
            .dashed
            .filter(|(d, g)| *d > 0.0 && *g > 0.0)
            .unwrap_or((0.0, 0.0));
        let params = [self.width.max(0.0), flags as f32, dash, gap];
        let pairs: Vec<(usize, usize)> = match self.mode {
            LineMode::Segments => (0..n / 2).map(|k| (2 * k, 2 * k + 1)).collect(),
            LineMode::Strip => (1..n).map(|k| (k - 1, k)).collect(),
            LineMode::Loop if n >= 2 => (1..n).map(|k| (k - 1, k)).chain([(n - 1, 0)]).collect(),
            LineMode::Loop => Vec::new(),
        };
        let mut dist = 0.0f32;
        for (i, j) in pairs {
            let (a, b) = (pts[i], pts[j]);
            if !(a.is_finite() && b.is_finite()) {
                continue;
            }
            let len = a.distance(b);
            out.push(LineInstance {
                a: a.extend(dist).to_array(),
                b: b.extend(dist + len).to_array(),
                color_a: color(i),
                color_b: color(j),
                params,
            });
            dist += len;
        }
    }
}

/// The unique edges of a mesh's triangles (three.js `WireframeGeometry`),
/// each once whichever triangles share it.
pub fn wire_edges(mesh: &MeshData) -> Vec<[u32; 2]> {
    let n = mesh.vertices.len() as u32;
    let mut edges: Vec<[u32; 2]> = Vec::with_capacity(mesh.indices.len());
    for t in mesh.indices.as_chunks::<3>().0 {
        if t.iter().any(|&i| i >= n) {
            continue;
        }
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            edges.push([a.min(b), a.max(b)]);
        }
    }
    edges.sort_unstable();
    edges.dedup();
    edges
}

/// The frame's instances, by where they draw.
#[derive(Default)]
pub(crate) struct LineFrame {
    /// Scene sets depth tested, then scene sets drawn over the scene, then
    /// the overlay's: (first, count) of each in the buffer.
    pub depth: (u32, u32),
    pub over: (u32, u32),
    pub overlay: (u32, u32),
    /// Segments and points drawn (wireframe edges included).
    pub instances: u32,
    /// The scene sets' instance ranges in the buffer with their layer
    /// masks, in buffer order: (mask, first, count). A camera draws the
    /// ranges whose masks share a bit with its own (`views.rs`).
    pub sets: Vec<(u32, u32, u32)>,
}

impl LineFrame {
    pub fn scene_empty(&self) -> bool {
        self.depth.1 == 0 && self.over.1 == 0
    }
}

/// The uniform of one pass (`struct LineFrame` in lines.wgsl).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LineUniform {
    pub view_proj: [[f32; 4]; 4],
    pub viewport: [f32; 4],
    pub proj: [f32; 4],
}

impl LineUniform {
    /// For a target of `width`×`height` device pixels whose height is
    /// `css_height` CSS pixels (0: one device pixel each), seen through
    /// `view_proj` whose projection is `proj`.
    pub fn new(
        view_proj: Mat4,
        proj: Mat4,
        width: u32,
        height: u32,
        css_height: f32,
        encode: bool,
    ) -> LineUniform {
        let scale = if css_height > 0.0 {
            height as f32 / css_height
        } else {
            1.0
        };
        LineUniform {
            view_proj: view_proj.to_cols_array_2d(),
            viewport: [width as f32, height as f32, scale, encode as u32 as f32],
            proj: [proj.y_axis.y, 0.0, 0.0, 0.0],
        }
    }
}

/// A mesh's unique edges, shared by its instances.
type Edges = std::rc::Rc<Vec<[u32; 2]>>;

pub(crate) struct LineRenderer {
    /// The uniform's layout, for a secondary camera's group (`views.rs`).
    pub layout: wgpu::BindGroupLayout,
    scene_uniform: wgpu::Buffer,
    scene_group: wgpu::BindGroup,
    overlay_uniform: wgpu::Buffer,
    overlay_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    /// Scene pipelines: depth tested, and drawn over the scene.
    depth: Deferred<wgpu::RenderPipeline>,
    over: Deferred<wgpu::RenderPipeline>,
    /// Overlay pipelines: in the output's format and its sRGB view's.
    output: Deferred<wgpu::RenderPipeline>,
    linear: Option<Deferred<wgpu::RenderPipeline>>,
    /// Per mesh index: its revision and unique edges (wireframe).
    edges: HashMap<usize, (u64, Edges)>,
}

/// The lines module (`crate::shaders` validates it).
pub(crate) const LINES_WGSL: &str = wgsl!("lines.wgsl");

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        samples: u32,
        output_format: wgpu::TextureFormat,
        linear_format: Option<wgpu::TextureFormat>,
    ) -> LineRenderer {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lines"),
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
        let uniform = |label: &str| {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: std::mem::size_of::<LineUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            (buffer, group)
        };
        let (scene_uniform, scene_group) = uniform("lines (scene)");
        let (overlay_uniform, overlay_group) = uniform("lines (overlay)");
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lines"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = crate::deferred::module(device, "lines", LINES_WGSL.to_string());
        const BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(LINE_LAYOUT)];
        let pipeline = |label: &'static str,
                        format: wgpu::TextureFormat,
                        depth: Option<wgpu::CompareFunction>,
                        samples: u32| {
            let (device, pl, module) = (device.clone(), pl.clone(), module.clone());
            Deferred::new(move || {
                use crate::deferred::AsModule;
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: module.module(),
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: BUFFERS,
                    },
                    primitive: Default::default(),
                    depth_stencil: depth.map(|compare| wgpu::DepthStencilState {
                        format: crate::DEPTH_FORMAT,
                        depth_write_enabled: Some(false),
                        depth_compare: Some(compare),
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: samples,
                        ..Default::default()
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: module.module(),
                        entry_point: Some("fs"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lines"),
            size: LINE_STRIDE * 256,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        LineRenderer {
            layout,
            scene_uniform,
            scene_group,
            overlay_uniform,
            overlay_group,
            buffer,
            depth: pipeline(
                "lines (depth tested)",
                crate::HDR_FORMAT,
                Some(wgpu::CompareFunction::LessEqual),
                samples,
            ),
            over: pipeline(
                "lines (over the scene)",
                crate::HDR_FORMAT,
                Some(wgpu::CompareFunction::Always),
                samples,
            ),
            output: pipeline("lines (overlay)", output_format, None, 1),
            linear: linear_format.map(|f| pipeline("lines (overlay, sRGB view)", f, None, 1)),
            edges: HashMap::new(),
        }
    }

    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub fn warm(&self) -> bool {
        let mut all: Vec<&dyn crate::deferred::Warm> = vec![&self.depth, &self.over, &self.output];
        if let Some(l) = &self.linear {
            all.push(l);
        }
        crate::deferred::warm_one(&all)
    }

    /// Builds and uploads the frame's instances: the scene's line and point
    /// sets and its wireframe instances (`wire(material index)`: the
    /// material's wireframe colour, `None` for a material drawn as
    /// triangles; `any_wire`: some material is).
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        any_wire: bool,
        wire: impl Fn(usize) -> Option<[f32; 4]>,
    ) -> LineFrame {
        let mut groups: [Vec<LineInstance>; 3] = Default::default();
        // Per scene slot: (mask, first in the slot, count) of each set.
        let mut sets: [Vec<(u32, u32, u32)>; 2] = Default::default();
        let slot = |layer: LineLayer, depth_test: bool| match (layer, depth_test) {
            (LineLayer::Overlay, _) => 2,
            (LineLayer::Scene, true) => 0,
            (LineLayer::Scene, false) => 1,
        };
        let mut record =
            |groups: &mut [Vec<LineInstance>; 3], k: usize, mask: u32, before: usize| {
                let n = groups[k].len() - before;
                if k < 2 && n > 0 {
                    sets[k].push((mask, before as u32, n as u32));
                }
            };
        for (i, l) in scene.lines.iter().enumerate() {
            let k = slot(l.layer(), l.depth_test);
            let before = groups[k].len();
            l.instances(&mut groups[k]);
            record(
                &mut groups,
                k,
                crate::views::layer_of(&scene.line_layers, i),
                before,
            );
        }
        for (i, p) in scene.points.iter().enumerate() {
            let k = slot(p.layer(), p.depth_test);
            let before = groups[k].len();
            p.instances(&mut groups[k]);
            record(
                &mut groups,
                k,
                crate::views::layer_of(&scene.point_layers, i),
                before,
            );
        }
        if any_wire {
            let live: std::collections::HashSet<usize> =
                scene.instances.iter().map(|i| i.mesh.index()).collect();
            self.edges.retain(|m, _| live.contains(m));
            for (i, inst) in scene.instances.iter().enumerate() {
                let Some(color) = wire(inst.material.index()) else {
                    continue;
                };
                let before = groups[0].len();
                let Some(mesh) = scene.mesh(inst.mesh) else {
                    continue;
                };
                let revision = scene.mesh_revision(inst.mesh.index());
                let edges = match self.edges.get(&inst.mesh.index()) {
                    Some((r, e)) if *r == revision => e.clone(),
                    _ => {
                        let e = std::rc::Rc::new(wire_edges(mesh));
                        self.edges.insert(inst.mesh.index(), (revision, e.clone()));
                        e
                    }
                };
                let out = &mut groups[0];
                out.reserve(edges.len());
                for &[a, b] in edges.iter() {
                    let pa = inst
                        .transform
                        .transform_point3(Vec3::from(mesh.vertices[a as usize].position));
                    let pb = inst
                        .transform
                        .transform_point3(Vec3::from(mesh.vertices[b as usize].position));
                    out.push(LineInstance {
                        a: pa.extend(0.0).to_array(),
                        b: pb.extend(0.0).to_array(),
                        color_a: color,
                        color_b: color,
                        params: [1.0, 0.0, 0.0, 0.0],
                    });
                }
                let mask = crate::views::layer_of(&scene.instance_layers, i);
                record(&mut groups, 0, mask, before);
            }
        }
        let counts = groups.each_ref().map(|g| g.len() as u32);
        let all: Vec<LineInstance> = groups.concat();
        let mut frame = LineFrame {
            depth: (0, counts[0]),
            over: (counts[0], counts[1]),
            overlay: (counts[0] + counts[1], counts[2]),
            instances: all.len() as u32,
            sets: sets[0]
                .iter()
                .copied()
                .chain(sets[1].iter().map(|&(m, f, n)| (m, f + counts[0], n)))
                .collect(),
        };
        if all.is_empty() {
            frame.instances = 0;
            return frame;
        }
        let bytes = all.len() as u64 * LINE_STRIDE;
        if self.buffer.size() < bytes {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("lines"),
                size: bytes.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&all));
        frame
    }

    pub fn set_scene_uniform(&self, queue: &wgpu::Queue, u: &LineUniform) {
        queue.write_buffer(&self.scene_uniform, 0, bytemuck::bytes_of(u));
    }

    pub fn set_overlay_uniform(&self, queue: &wgpu::Queue, u: &LineUniform) {
        queue.write_buffer(&self.overlay_uniform, 0, bytemuck::bytes_of(u));
    }

    fn draw(&self, pass: &mut wgpu::RenderPass, range: (u32, u32)) {
        let start = range.0 as u64 * LINE_STRIDE;
        pass.set_vertex_buffer(
            0,
            self.buffer
                .slice(start..start + range.1 as u64 * LINE_STRIDE),
        );
        pass.draw(0..6, 0..range.1);
    }

    /// The scene's sets on the main camera's layers (`mask`), in the main
    /// pass (its group 0 is rebound: the caller sets its own again after).
    /// Answers the draws made.
    pub fn draw_scene(&self, pass: &mut wgpu::RenderPass, frame: &LineFrame, mask: u32) -> u32 {
        self.draw_scene_with(pass, frame, &self.scene_group, mask)
    }

    /// The scene's sets on a camera's layers (`mask`) with that camera's
    /// uniform `group` (a secondary camera's, [`LineRenderer::view_group`]).
    /// The admitted sets next to each other are one draw.
    pub fn draw_scene_with(
        &self,
        pass: &mut wgpu::RenderPass,
        frame: &LineFrame,
        group: &wgpu::BindGroup,
        mask: u32,
    ) -> u32 {
        if frame.scene_empty() {
            return 0;
        }
        pass.set_bind_group(0, group, &[]);
        let mut draws = 0;
        for (pipeline, (lo, n)) in [(&self.depth, frame.depth), (&self.over, frame.over)] {
            let mut ranges: Vec<(u32, u32)> = Vec::new();
            for &(m, first, count) in &frame.sets {
                if m & mask == 0 || first < lo || first >= lo + n {
                    continue;
                }
                match ranges.last_mut() {
                    Some(r) if r.0 + r.1 == first => r.1 += count,
                    _ => ranges.push((first, count)),
                }
            }
            if !ranges.is_empty() {
                pass.set_pipeline(pipeline);
            }
            for r in ranges {
                self.draw(pass, r);
                draws += 1;
            }
        }
        draws
    }

    /// A secondary camera's uniform group over its `buffer`.
    pub fn view_group(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lines (view)"),
            layout: &self.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// The overlay's sets over the finished frame in `target` (through its
    /// sRGB view when `srgb_view`). Answers the draws made.
    pub fn draw_overlay(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        srgb_view: bool,
        frame: &LineFrame,
    ) -> u32 {
        if frame.overlay.1 == 0 {
            return 0;
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lines (overlay)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        let pipeline = match (&self.linear, srgb_view) {
            (Some(l), true) => l,
            _ => &self.output,
        };
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &self.overlay_group, &[]);
        self.draw(&mut pass, frame.overlay);
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes;

    #[test]
    fn a_strip_carries_its_distance_along_and_a_loop_closes() {
        let l = Lines {
            points: vec![Vec3::ZERO, Vec3::X, Vec3::new(1.0, 2.0, 0.0)],
            mode: LineMode::Loop,
            transform: Mat4::from_scale(Vec3::splat(2.0)),
            ..Lines::default()
        };
        let mut out = Vec::new();
        l.instances(&mut out);
        assert_eq!(out.len(), 3);
        // Distances after the transform: 2, then 4 more, then back sqrt(4 + 16).
        assert_eq!(out[0].a[3], 0.0);
        assert_eq!(out[0].b[3], 2.0);
        assert_eq!(out[1].b[3], 6.0);
        assert!((out[2].b[3] - (6.0 + 20f32.sqrt())).abs() < 1e-5);
        let seg = Lines {
            points: vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            mode: LineMode::Segments,
            ..Lines::default()
        };
        out.clear();
        seg.instances(&mut out);
        // An odd last point has no partner.
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn a_box_face_has_its_four_sides_and_one_diagonal() {
        // The box's faces have vertices of their own: 6 faces of 4 sides
        // and the diagonal their two triangles share, each edge once.
        assert_eq!(wire_edges(&shapes::cuboid(Vec3::ONE)).len(), 6 * 5);
    }
}
