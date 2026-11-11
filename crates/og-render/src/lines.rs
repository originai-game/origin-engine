//! Lines (three.js `Line`, `LineSegments`, `LineLoop` with a
//! `LineBasicMaterial` or `LineDashedMaterial`) and the pass that draws
//! them with the point clouds (`points.rs`) and the wireframe materials.
//!
//! Every segment and point is one instance of a screen-space quad
//! (`shaders/lines.wgsl`): a width in CSS pixels stays that wide at any
//! distance, a world width shrinks with it, and the edge is feathered by a
//! pixel on every backend (WebGPU has no line width at all). Sets draw in one of two places ([`LineLayer`]):
//!
//! - **Scene**: in the main pass after the transparent meshes, particles
//!   and strips, into the HDR target (exposed, bloomed and tone mapped with
//!   the scene), depth tested or drawn over it (`depth_test`).
//! - **Overlay**: over the finished frame after every post effect and
//!   before the UI overlay, never depth tested, colours as given: debug
//!   draw and screen lines, which read the same in every capture.
//!
//! A material with `wireframe` draws its instances' triangle edges here
//! (scene, depth tested, one CSS pixel): instead of their triangles (1),
//! or over them (2, a look's wireframe). The unique edges of a mesh are
//! found once per mesh revision. An edge takes the colour its surface
//! shows along it, sampled from the material's own group (its colour,
//! map and vertex colours plus its emissive light and map: a textured
//! crate's wire is its texture's colour there, not the material's white),
//! or the material's one `wireframeColor` ([`Wire`]).

use crate::deferred::Sampled;
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

/// A material's wireframe (`GpuMaterial::wireframe`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Wire {
    /// One linear colour for every edge (`wireframeColor`, with the
    /// opacity); `None`: the colour the surface shows along each edge.
    pub color: Option<[f32; 4]>,
    /// The feature angle ([`feature_edges`]).
    pub angle: f32,
}

/// A wireframe edge (`vs_wire` in lines.wgsl): a line whose `params.z` is
/// 1 when it takes the surface's colour (`color_a`, `color_b` then the
/// vertex colours), with the surface's uv at each end.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct WireInstance {
    pub line: LineInstance,
    /// xy at the start, zw at the end.
    pub uv: [f32; 4],
}

const WIRE_STRIDE: u64 = std::mem::size_of::<WireInstance>() as u64;

const WIRE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: WIRE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4],
};

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

/// A wireframe edge with the faces it joins, in the mesh's own space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct WireEdge {
    pub pair: [u32; 2],
    /// Its two faces' unit normals (by their winding); a border's second
    /// is zero, and both are for an edge of three or more faces or of none.
    pub faces: [Vec3; 2],
}

impl WireEdge {
    /// Whether the edge lies on the silhouette seen from `eye` (the mesh's
    /// own space; w 0: a direction towards an orthographic camera) through
    /// its point `p`: a border, or one face turned towards the eye and the
    /// other away (a plane's side is kept by any affine transform, so the
    /// test holds in the mesh's space whatever the instance's scale).
    pub fn silhouette(&self, eye: glam::Vec4, p: Vec3) -> bool {
        let [a, b] = self.faces;
        if a == Vec3::ZERO {
            return false;
        }
        if b == Vec3::ZERO {
            return true;
        }
        let to_eye = eye.truncate() - p * eye.w;
        a.dot(to_eye) * b.dot(to_eye) <= 0.0
    }
}

/// The feature edges of a mesh (a vector or neon wireframe): its borders
/// and the edges whose two faces meet at more than `angle_degrees`, the
/// triangles welded by position first, so a mesh split at its uv seams or
/// hard edges (a cooked or flat-shaded mesh) shows no seam and its hard
/// edges are found across the split. Each edge once, by the vertex indices
/// of its first triangle, with its faces ([`WireEdge`]). 0 is every edge
/// ([`wire_edges`]), their faces found across the weld. A cube gives its
/// 12 edges, a smooth sphere none.
pub(crate) fn feature_edges(mesh: &MeshData, angle_degrees: f32) -> Vec<WireEdge> {
    let n = mesh.vertices.len() as u32;
    let cos = angle_degrees.to_radians().cos();
    // Welded ids by position, to a millionth of the mesh's size (a seam's
    // two copies of a vertex differ in the last bits: cos 2π is not 1).
    let extent = mesh.vertices.iter().fold(0.0f32, |m, v| {
        v.position.iter().fold(m, |m, c| m.max(c.abs()))
    });
    let eps = (extent * 1e-6).max(1e-12);
    let mut weld: HashMap<[i64; 3], u32> = HashMap::new();
    let ids: Vec<u32> = mesh
        .vertices
        .iter()
        .map(|v| {
            let key = v.position.map(|c| (c / eps).round() as i64);
            let next = weld.len() as u32;
            *weld.entry(key).or_insert(next)
        })
        .collect();
    let welded = |[a, b]: [u32; 2]| {
        let (wa, wb) = (ids[a as usize], ids[b as usize]);
        [wa.min(wb), wa.max(wb)]
    };
    // Per welded edge: its first vertex pair and its faces' normals.
    let mut faces: HashMap<[u32; 2], ([u32; 2], Vec<Vec3>)> = HashMap::new();
    let mut order: Vec<[u32; 2]> = Vec::new();
    for t in mesh.indices.as_chunks::<3>().0 {
        if t.iter().any(|&i| i >= n) {
            continue;
        }
        // A triangle two of whose corners weld (a sphere's pole) has no
        // face: its normal would be noise.
        let w = t.map(|i| ids[i as usize]);
        if w[0] == w[1] || w[1] == w[2] || w[2] == w[0] {
            continue;
        }
        let p = t.map(|i| Vec3::from(mesh.vertices[i as usize].position));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]);
        if normal.length_squared() <= 1e-24 {
            continue;
        }
        let normal = normal.normalize();
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            let key = welded([a, b]);
            let e = faces.entry(key).or_insert_with(|| {
                order.push(key);
                ([a.min(b), a.max(b)], Vec::new())
            });
            e.1.push(normal);
        }
    }
    let of = |normals: &[Vec3]| match normals {
        [a] => [*a, Vec3::ZERO],
        [a, b] => [*a, *b],
        _ => [Vec3::ZERO; 2],
    };
    if angle_degrees <= 0.0 {
        return wire_edges(mesh)
            .into_iter()
            .map(|pair| WireEdge {
                pair,
                faces: faces
                    .get(&welded(pair))
                    .map_or([Vec3::ZERO; 2], |(_, f)| of(f)),
            })
            .collect();
    }
    order
        .into_iter()
        .filter_map(|k| {
            let (pair, normals) = &faces[&k];
            let feature = match normals.as_slice() {
                [_] => true,
                [a, b] => a.dot(*b) < cos,
                _ => true,
            };
            feature.then(|| WireEdge {
                pair: *pair,
                faces: of(normals),
            })
        })
        .collect()
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
    /// The wireframe edges' ranges in the wire buffer: (mask, material,
    /// first, count), drawn with the material's group.
    pub wires: Vec<(u32, u32, u32, u32)>,
    /// The scene sets' instance ranges in the buffer with their layer
    /// masks, in buffer order: (mask, first, count). A camera draws the
    /// ranges whose masks share a bit with its own (`views.rs`).
    pub sets: Vec<(u32, u32, u32)>,
}

impl LineFrame {
    pub fn scene_empty(&self) -> bool {
        self.depth.1 == 0 && self.over.1 == 0 && self.wires.is_empty()
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
type Edges = std::rc::Rc<Vec<WireEdge>>;

pub(crate) struct LineRenderer {
    /// The uniform's layout, for a secondary camera's group (`views.rs`).
    pub layout: wgpu::BindGroupLayout,
    scene_uniform: wgpu::Buffer,
    scene_group: wgpu::BindGroup,
    overlay_uniform: wgpu::Buffer,
    overlay_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    /// The wireframes' edges ([`WireInstance`]).
    wire_buffer: wgpu::Buffer,
    /// Scene pipelines: depth tested, and drawn over the scene.
    depth: Sampled<wgpu::RenderPipeline>,
    over: Sampled<wgpu::RenderPipeline>,
    /// Wireframe edges, depth tested, with their material's group (1).
    wire: Sampled<wgpu::RenderPipeline>,
    /// Overlay pipelines: in the output's format and its sRGB view's.
    output: Sampled<wgpu::RenderPipeline>,
    linear: Option<Sampled<wgpu::RenderPipeline>>,
    /// Per mesh index and feature angle (its bits): the mesh's revision
    /// and its edges (wireframe).
    edges: HashMap<(usize, u32), (u64, Edges)>,
}

/// The lines module (`crate::shaders` validates it), after the standard
/// material's uniform (the wires' group 1).
pub(crate) const LINES_WGSL: &str = concat!(wgsl!("material.wgsl"), wgsl!("lines.wgsl"));

impl LineRenderer {
    pub fn new(
        device: &wgpu::Device,
        material_layout: &wgpu::BindGroupLayout,
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
        let wire_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lines (wireframe)"),
            bind_group_layouts: &[Some(&layout), Some(material_layout)],
            immediate_size: 0,
        });
        let module = crate::deferred::module(device, "lines", LINES_WGSL.to_string());
        const BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(LINE_LAYOUT)];
        const WIRE_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] = &[Some(WIRE_LAYOUT)];
        let pipeline = |label: &'static str,
                        format: wgpu::TextureFormat,
                        depth: Option<wgpu::CompareFunction>,
                        samples: u32,
                        wire: bool| {
            let (device, module) = (device.clone(), module.clone());
            let pl = if wire { wire_pl.clone() } else { pl.clone() };
            let (vs, fs, buffers) = if wire {
                ("vs_wire", "fs_wire", WIRE_BUFFERS)
            } else {
                ("vs", "fs", BUFFERS)
            };
            Sampled::new(samples, move |samples| {
                use crate::deferred::AsModule;
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&pl),
                    vertex: wgpu::VertexState {
                        module: module.module(),
                        entry_point: Some(vs),
                        compilation_options: Default::default(),
                        buffers,
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
                        entry_point: Some(fs),
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
        let wire_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lines (wireframe)"),
            size: WIRE_STRIDE * 256,
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
            wire_buffer,
            depth: pipeline(
                "lines (depth tested)",
                crate::HDR_FORMAT,
                Some(wgpu::CompareFunction::LessEqual),
                samples,
                false,
            ),
            over: pipeline(
                "lines (over the scene)",
                crate::HDR_FORMAT,
                Some(wgpu::CompareFunction::Always),
                samples,
                false,
            ),
            wire: pipeline(
                "lines (wireframe)",
                crate::HDR_FORMAT,
                Some(wgpu::CompareFunction::LessEqual),
                samples,
                true,
            ),
            output: pipeline("lines (overlay)", output_format, None, 1, false),
            linear: linear_format
                .map(|f| pipeline("lines (overlay, sRGB view)", f, None, 1, false)),
            edges: HashMap::new(),
        }
    }

    /// Draw the scene's lines into a target of `samples` per pixel from
    /// now on (the overlay's are one sample whatever the scene's).
    pub fn set_samples(&self, samples: u32) {
        self.depth.set_samples(samples);
        self.over.set_samples(samples);
        self.wire.set_samples(samples);
    }

    /// Makes one of its pipelines no frame used yet (Renderer::warm).
    pub fn warm(&self) -> bool {
        let mut all: Vec<&dyn crate::deferred::Warm> =
            vec![&self.depth, &self.over, &self.wire, &self.output];
        if let Some(l) = &self.linear {
            all.push(l);
        }
        crate::deferred::warm_one(&all)
    }

    /// Builds and uploads the frame's instances: the scene's line and point
    /// sets and its wireframe instances (`wire(material index)`: the
    /// material's [`Wire`] and whether an outline lines it, `None` for a
    /// material drawn as triangles; `any_wire`: some material is).
    ///
    /// An outlined material's silhouette edges, seen from `eye` (the main
    /// camera's position, w 1, or the direction towards an orthographic
    /// one, w 0), are left to the outline: the line, a pixel wide and
    /// centred on the edge, would lie half outside the shape along the
    /// outline drawn inside it, a second edge in the wire's colour.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &RenderScene,
        any_wire: bool,
        eye: glam::Vec4,
        wire: impl Fn(usize) -> Option<(Wire, bool)>,
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
        for (i, l) in scene.effects.lines.iter().enumerate() {
            let k = slot(l.layer(), l.depth_test);
            let before = groups[k].len();
            l.instances(&mut groups[k]);
            record(
                &mut groups,
                k,
                crate::views::layer_of(&scene.effects.line_layers, i),
                before,
            );
        }
        for (i, p) in scene.effects.points.iter().enumerate() {
            let k = slot(p.layer(), p.depth_test);
            let before = groups[k].len();
            p.instances(&mut groups[k]);
            record(
                &mut groups,
                k,
                crate::views::layer_of(&scene.effects.point_layers, i),
                before,
            );
        }
        let mut wires: Vec<WireInstance> = Vec::new();
        let mut wire_sets: Vec<(u32, u32, u32, u32)> = Vec::new();
        if any_wire {
            let live: std::collections::HashSet<usize> =
                scene.instances.iter().map(|i| i.mesh.index()).collect();
            self.edges.retain(|(m, _), _| live.contains(m));
            for (i, inst) in scene.instances.iter().enumerate() {
                let Some((w, outlined)) = wire(inst.material.index()) else {
                    continue;
                };
                let angle = w.angle;
                let before = wires.len();
                let Some(mesh) = scene.mesh(inst.mesh) else {
                    continue;
                };
                let revision = scene.mesh_revision(inst.mesh.index());
                let key = (inst.mesh.index(), angle.to_bits());
                let edges = match self.edges.get(&key) {
                    Some((r, e)) if *r == revision => e.clone(),
                    _ => {
                        let e = std::rc::Rc::new(feature_edges(mesh, angle));
                        self.edges.insert(key, (revision, e.clone()));
                        e
                    }
                };
                let local_eye = outlined.then(|| inst.transform.inverse() * eye);
                wires.reserve(edges.len());
                let surface = w.color.is_none() as u32 as f32;
                for e in edges.iter() {
                    let [va, vb] = e.pair.map(|v| &mesh.vertices[v as usize]);
                    let a = Vec3::from(va.position);
                    if local_eye.is_some_and(|eye| e.silhouette(eye, a)) {
                        continue;
                    }
                    let pa = inst.transform.transform_point3(a);
                    let pb = inst.transform.transform_point3(Vec3::from(vb.position));
                    let (color_a, color_b) = w.color.map_or((va.color, vb.color), |c| (c, c));
                    wires.push(WireInstance {
                        line: LineInstance {
                            a: pa.extend(0.0).to_array(),
                            b: pb.extend(0.0).to_array(),
                            color_a,
                            color_b,
                            params: [1.0, 0.0, surface, 0.0],
                        },
                        uv: [va.uv[0], va.uv[1], vb.uv[0], vb.uv[1]],
                    });
                }
                let n = (wires.len() - before) as u32;
                if n > 0 {
                    let mask = crate::views::layer_of(&scene.instance_layers, i);
                    let material = inst.material.index() as u32;
                    match wire_sets.last_mut() {
                        Some(r) if r.0 == mask && r.1 == material => r.3 += n,
                        _ => wire_sets.push((mask, material, before as u32, n)),
                    }
                }
            }
        }
        let counts = groups.each_ref().map(|g| g.len() as u32);
        let all: Vec<LineInstance> = groups.concat();
        let frame = LineFrame {
            depth: (0, counts[0]),
            over: (counts[0], counts[1]),
            overlay: (counts[0] + counts[1], counts[2]),
            instances: (all.len() + wires.len()) as u32,
            wires: wire_sets,
            sets: sets[0]
                .iter()
                .copied()
                .chain(sets[1].iter().map(|&(m, f, n)| (m, f + counts[0], n)))
                .collect(),
        };
        if !wires.is_empty() {
            let bytes = wires.len() as u64 * WIRE_STRIDE;
            if self.wire_buffer.size() < bytes {
                self.wire_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("lines (wireframe)"),
                    size: bytes.next_power_of_two(),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
            }
            queue.write_buffer(&self.wire_buffer, 0, bytemuck::cast_slice(&wires));
        }
        if all.is_empty() {
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
    /// pass (its groups 0 and 1 are rebound: the caller sets its own again
    /// after), the wireframes with their material's standard group
    /// (`material(index)`). Answers the draws made.
    pub fn draw_scene<'a>(
        &self,
        pass: &mut wgpu::RenderPass,
        frame: &LineFrame,
        mask: u32,
        material: &dyn Fn(usize) -> &'a wgpu::BindGroup,
    ) -> u32 {
        self.draw_scene_with(pass, frame, &self.scene_group, mask, material)
    }

    /// The scene's sets on a camera's layers (`mask`) with that camera's
    /// uniform `group` (a secondary camera's, [`LineRenderer::view_group`]).
    /// The admitted sets next to each other are one draw.
    pub fn draw_scene_with<'a>(
        &self,
        pass: &mut wgpu::RenderPass,
        frame: &LineFrame,
        group: &wgpu::BindGroup,
        mask: u32,
        material: &dyn Fn(usize) -> &'a wgpu::BindGroup,
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
        let mut wire_set = false;
        for &(m, index, first, count) in &frame.wires {
            if m & mask == 0 {
                continue;
            }
            if !wire_set {
                pass.set_pipeline(&self.wire);
                wire_set = true;
            }
            pass.set_bind_group(1, material(index as usize), &[]);
            let start = first as u64 * WIRE_STRIDE;
            pass.set_vertex_buffer(
                0,
                self.wire_buffer
                    .slice(start..start + count as u64 * WIRE_STRIDE),
            );
            pass.draw(0..6, 0..count);
            draws += 1;
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
        encoder: &mut crate::Encoder,
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

    /// Feature edges: a box (its faces' vertices split) gives its 12 edges
    /// and no face diagonal, a smooth sphere none (its seam welded), a
    /// flat quad its 4 borders; at 0 every edge. Ways this could fail: no
    /// weld (the box's 24 face sides all borders), the diagonal counted
    /// (its two faces are coplanar), the sphere's seam left open.
    #[test]
    fn feature_edges_keep_the_creases_and_borders_only() {
        let cube = shapes::cuboid(Vec3::ONE);
        assert_eq!(feature_edges(&cube, 30.0).len(), 12);
        assert_eq!(feature_edges(&cube, 0.0).len(), 6 * 5);
        assert_eq!(feature_edges(&shapes::sphere(48, 32), 30.0).len(), 0);
        assert_eq!(feature_edges(&shapes::plane(1.0, 1.0), 30.0).len(), 4);
    }

    /// Silhouette edges (left to an outline): a cube seen from a corner
    /// shows a hexagon, its six rim edges, while the three edges meeting
    /// at the near corner (both faces towards the eye) stay lines; seen
    /// face on, the near face's four sides. A border is always one, and
    /// an orthographic eye (a direction) gives the same as a far point.
    /// Ways this could fail: the sign test inverted (the near creases
    /// dropped instead), the eye's w ignored (a direction read as a point
    /// near the origin).
    #[test]
    fn silhouette_edges_are_the_rim_seen_from_the_eye() {
        let cube = shapes::cuboid(Vec3::ONE);
        let edges = feature_edges(&cube, 30.0);
        let p = |e: &WireEdge| Vec3::from(cube.vertices[e.pair[0] as usize].position);
        let rim = |eye: glam::Vec4| edges.iter().filter(|e| e.silhouette(eye, p(e))).count();
        assert_eq!(rim(glam::Vec4::new(5.0, 4.0, 3.0, 1.0)), 6);
        assert_eq!(rim(glam::Vec4::new(5.0, 4.0, 3.0, 0.0)), 6);
        assert_eq!(rim(glam::Vec4::new(0.0, 0.0, 10.0, 1.0)), 4);
        // The near corner's three edges are kept, not dropped.
        let near = edges
            .iter()
            .filter(|e| {
                e.pair
                    .iter()
                    .any(|&i| Vec3::from(cube.vertices[i as usize].position) == Vec3::splat(0.5))
            })
            .filter(|e| !e.silhouette(glam::Vec4::new(5.0, 4.0, 3.0, 1.0), p(e)))
            .count();
        assert_eq!(near, 3);
        let quad = feature_edges(&shapes::plane(1.0, 1.0), 30.0);
        assert!(
            quad.iter()
                .all(|e| e.silhouette(glam::Vec4::new(0.0, 5.0, 0.0, 1.0), Vec3::ZERO))
        );
    }
}
