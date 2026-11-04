//! Draw-only refinement of coarse simulation meshes. Phong tessellation,
//! Boubekeur and Alexa (2008), independently implemented from the paper;
//! see third_party/phong-tessellation.md. No simulation reads this output.

use crate::{Deferred, MeshData};
use glam::Vec3;

pub(crate) const WGSL: &str = include_str!("../shaders/surface.wgsl");

/// The CPU solve topology whose draw-only surface the renderer refines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationSurface {
    Cloth { columns: u32, rows: u32 },
    SoftBody,
}

pub(crate) fn subdivisions(value: u32) -> u32 {
    if value == 0 { 2 } else { value.min(8) }
}

/// Projection displacement is at most the coarse box's diagonal times alpha.
pub(crate) fn bounds(mesh: &MeshData) -> Option<(Vec3, Vec3)> {
    mesh.bounds().map(|(lo, hi)| {
        let half = (hi - lo) * 0.5;
        ((lo + hi) * 0.5, half + Vec3::splat(half.length() * 1.5))
    })
}

fn samples(n: u32) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity((n * n * 3) as usize);
    let bary = |x, y| {
        [
            (n - x - y) as f32 / n as f32,
            x as f32 / n as f32,
            y as f32 / n as f32,
        ]
    };
    for y in 0..n {
        for x in 0..n - y {
            out.extend([bary(x, y), bary(x + 1, y), bary(x, y + 1)]);
            if x + y + 1 < n {
                out.extend([bary(x + 1, y), bary(x + 1, y + 1), bary(x, y + 1)]);
            }
        }
    }
    out
}

/// Area-weighted normals, shared by all triangles incident on a solve point.
fn normals(mesh: &MeshData) -> Vec<Vec3> {
    let mut out = vec![Vec3::ZERO; mesh.vertices.len()];
    for t in mesh.indices.as_chunks::<3>().0 {
        let [a, b, c] = t.map(|i| Vec3::from(mesh.vertices[i as usize].position));
        let n = (b - a).cross(c - a);
        for &i in t {
            out[i as usize] += n;
        }
    }
    out.iter_mut().for_each(|n| *n = n.normalize_or(Vec3::Z));
    out
}

/// Coarse geometry with CPU normals on devices without compute/vertex storage.
pub(crate) fn fallback(mesh: &MeshData, kind: SimulationSurface) -> MeshData {
    let mut out = mesh.clone();
    match kind {
        SimulationSurface::SoftBody => {
            for (v, n) in out.vertices.iter_mut().zip(normals(mesh)) {
                v.normal = n.to_array();
            }
        }
        SimulationSurface::Cloth { columns, rows } => {
            let (w, h) = (columns as usize, rows as usize);
            let at = |x: usize, y: usize| Vec3::from(mesh.vertices[y * w + x].position);
            for y in 0..h {
                for x in 0..w {
                    let dx = at((x + 1).min(w - 1), y) - at(x.saturating_sub(1), y);
                    let dy = at(x, (y + 1).min(h - 1)) - at(x, y.saturating_sub(1));
                    out.vertices[y * w + x].normal = dy.cross(dx).normalize_or(Vec3::Z).to_array();
                }
            }
        }
    }
    out.generate_tangents();
    out
}

/// CSR adjacency after the triangle indices: header (vertices, triangles,
/// start of offsets, start of incident faces), then indices, offsets, faces.
fn topology(mesh: &MeshData, level: u32) -> Vec<u32> {
    let mut incident = vec![Vec::new(); mesh.vertices.len()];
    for (f, t) in mesh.indices.as_chunks::<3>().0.iter().enumerate() {
        for &i in t {
            incident[i as usize].push(f as u32);
        }
    }
    let offset = 8 + mesh.indices.len() as u32;
    let faces = offset + mesh.vertices.len() as u32 + 1;
    let mut out = vec![
        mesh.vertices.len() as u32,
        mesh.indices.len() as u32 / 3,
        offset,
        faces,
        0,
        level,
        3 * level * level,
        0,
    ];
    out.extend(&mesh.indices);
    let mut end = 0;
    for v in &incident {
        out.push(end);
        end += v.len() as u32;
    }
    out.push(end);
    out.extend(incident.into_iter().flatten());
    out[4] = out.len() as u32;
    out.extend(samples(level).into_iter().flatten().map(f32::to_bits));
    out
}

pub(crate) struct SurfacePipelines {
    layout: wgpu::BindGroupLayout,
    normals: Deferred<wgpu::ComputePipeline>,
    refine: Deferred<wgpu::ComputePipeline>,
}

pub(crate) struct SurfaceMesh {
    source: wgpu::Buffer,
    topology: wgpu::Buffer,
    // The topology is static across simulation steps; rebuild only on edits.
    indices: Vec<u32>,
    group: wgpu::BindGroup,
    vertices: u32,
    triangles: u32,
    level: u32,
    pub(crate) previous: wgpu::Buffer,
    initialized: std::cell::Cell<bool>,
}

impl SurfacePipelines {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let entries: Vec<_> = (0..4)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding < 2,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("simulation surface"),
            entries: &entries,
        });
        let pipeline = |entry: &'static str| {
            let device = device.clone();
            let layout = layout.clone();
            Deferred::new(move || {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("simulation surface"),
                            bind_group_layouts: &[Some(&layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("simulation surface"),
                        source: wgpu::ShaderSource::Wgsl(WGSL.into()),
                    }),
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Self {
            normals: pipeline("normals"),
            refine: pipeline("refine"),
            layout,
        }
    }

    pub(crate) fn mesh(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: &MeshData,
        output: &wgpu::Buffer,
        level: u32,
    ) -> SurfaceMesh {
        let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let source = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("surface coarse vertices"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: storage,
            },
        );
        let topology = crate::buffer_init(
            device,
            queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("surface topology"),
                contents: bytemuck::cast_slice(&topology(mesh, level)),
                usage: storage,
            },
        );
        let normals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("surface normals"),
            size: mesh.vertices.len() as u64 * 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let entries: Vec<_> = [&source, &topology, &normals, output]
            .into_iter()
            .enumerate()
            .map(|(i, b)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: b.as_entire_binding(),
            })
            .collect();
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("simulation surface"),
            layout: &self.layout,
            entries: &entries,
        });
        let previous = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("previous surface"),
            size: output.size(),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        SurfaceMesh {
            source,
            topology,
            indices: mesh.indices.clone(),
            group,
            level,
            previous,
            initialized: std::cell::Cell::new(false),
            vertices: mesh.vertices.len() as u32,
            triangles: mesh.indices.len() as u32 / 3,
        }
    }

    pub(crate) fn encode(
        &self,
        encoder: &mut crate::Encoder,
        mesh: &SurfaceMesh,
        output: &wgpu::Buffer,
        reset: bool,
    ) {
        let history = mesh.initialized.get() && !reset;
        if history {
            encoder.copy_buffer_to_buffer(output, 0, &mesh.previous, 0, output.size());
        }
        // Separate passes establish the storage dependency, without atomics.
        for (pipeline, count) in [
            (&self.normals, mesh.vertices),
            (&self.refine, mesh.triangles * 3 * mesh.level * mesh.level),
        ] {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("simulation surface"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &mesh.group, &[]);
            // A 2D grid also covers meshes above 65535 workgroups.
            let groups = count.div_ceil(64);
            pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
        }
        if !history {
            encoder.copy_buffer_to_buffer(output, 0, &mesh.previous, 0, output.size());
        }
        mesh.initialized.set(true);
    }
}

impl SurfaceMesh {
    pub(crate) fn matches(&self, mesh: &MeshData, level: u32) -> bool {
        self.vertices as usize == mesh.vertices.len()
            && self.indices == mesh.indices
            && self.level == level
    }

    pub(crate) fn update(&self, queue: &wgpu::Queue, mesh: &MeshData) {
        queue.write_buffer(&self.source, 0, bytemuck::cast_slice(&mesh.vertices));
    }

    pub(crate) fn bytes(&self) -> u64 {
        self.source.size()
            + self.topology.size()
            + u64::from(self.vertices) * 16
            + self.previous.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Vertex;

    #[test]
    fn a_surface_keeps_coarse_geometry_and_excludes_other_vertex_layouts() {
        let mesh = MeshData {
            vertices: [[0., 0., 0.], [2., 0., 0.], [0., 1., 0.], [0., 0., 3.]]
                .map(|p| Vertex {
                    position: p,
                    ..Default::default()
                })
                .to_vec(),
            indices: vec![0, 1, 2, 1, 0, 3],
        };
        let mut scene = crate::RenderScene::new();
        let handle = scene.add_mesh(mesh.clone());
        let foliage = crate::meshlet::MeshFoliage {
            vertices: vec![Default::default(); mesh.vertices.len()],
            top: 1.0,
            ..Default::default()
        };
        assert!(scene.set_mesh_foliage(handle, Some(foliage)));
        assert!(!scene.set_simulation_surface(handle, mesh.clone(), SimulationSurface::SoftBody));
        assert!(scene.mesh_foliage(handle).is_some());
        assert!(scene.set_mesh_foliage(handle, None));
        assert!(scene.set_simulation_surface(handle, mesh.clone(), SimulationSurface::SoftBody));
        assert_eq!(scene.mesh(handle), Some(&mesh));
        assert!(!scene.set_vertex_encoding(handle, crate::VertexEncoding::Compact));
        assert!(!scene.set_mesh_clusters(handle, None));
        assert!(!scene.set_mesh_foliage(handle, None));
        assert!(scene.set_mesh(handle, mesh.clone()));
        assert!(scene.set_vertex_encoding(handle, crate::VertexEncoding::Compact));
        assert!(scene.remove_mesh(handle));
        let reused = scene.add_mesh(mesh);
        assert!(!scene.contains_mesh(handle));
        assert!(scene.set_vertex_encoding(reused, crate::VertexEncoding::Compact));
    }

    #[test]
    fn fallback_keeps_coarse_positions_and_area_weighted_normals() {
        let mesh = MeshData {
            vertices: [[0., 0., 0.], [3., 0., 0.], [0., 2., 0.]]
                .map(|p| Vertex {
                    position: p,
                    ..Default::default()
                })
                .to_vec(),
            indices: vec![0, 1, 2, 0, 0, 0],
        };
        let coarse = fallback(&mesh, SimulationSurface::SoftBody);
        assert_eq!(coarse.indices, mesh.indices);
        assert_eq!(coarse.vertices.len(), 3);
        assert_eq!(coarse.vertices[1].position, [3., 0., 0.]);
        assert!(coarse.vertices.iter().all(|v| v.normal == [0., 0., 1.]));
    }
}
