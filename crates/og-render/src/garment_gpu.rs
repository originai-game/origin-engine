//! The garment's visual pose only. Four compute bindings; the lit and
//! prepass shaders keep reading their existing deformation palette slots.

use crate::visual_sim::{Topology, VisualMesh, VisualPipelines};
use crate::{Deferred, GarmentDeformation, GarmentTopology};
use std::{collections::HashMap, sync::Arc};

pub(crate) const WGSL: &str = include_str!("../shaders/garment.wgsl");

pub(crate) struct Mesh {
    topology: Arc<GarmentTopology>,
    input: wgpu::Buffer,
    adjacency: wgpu::Buffer,
    points: wgpu::Buffer,
    initialized: bool,
    /// The GPU simulation of a visual garment (SIMULATION_VISUAL).
    visual: Option<VisualMesh>,
}

impl Mesh {
    pub(crate) fn bytes(&self) -> u64 {
        self.input.size()
            + self.adjacency.size()
            + self.points.size()
            + self.visual.as_ref().map_or(0, VisualMesh::bytes)
    }
}

pub(crate) struct Pipelines {
    layout: wgpu::BindGroupLayout,
    pose: Deferred<wgpu::ComputePipeline>,
    scatter: Deferred<wgpu::ComputePipeline>,
}

fn adjacency(t: &GarmentTopology) -> Vec<u32> {
    let mut incident = vec![Vec::new(); t.rest.len()];
    for (f, triangle) in t.triangles.iter().enumerate() {
        for &p in triangle {
            incident[p as usize].push(f as u32);
        }
    }
    let offset = 4 + t.triangles.len() * 3;
    let faces = offset + t.rest.len() + 1;
    let mut data = vec![t.triangles.len() as u32, offset as u32, faces as u32, 0];
    data.extend(t.triangles.iter().flatten());
    let mut end = 0;
    for faces in &incident {
        data.push(end);
        end += faces.len() as u32;
    }
    data.push(end);
    data.extend(incident.into_iter().flatten());
    data[3] = data.len() as u32;
    for &(point, sign) in &t.vertices {
        data.extend([point, sign.to_bits()]);
    }
    data
}

impl Pipelines {
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
            label: Some("garment draw"),
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
                            label: Some("garment draw"),
                            bind_group_layouts: &[Some(&layout)],
                            immediate_size: 0,
                        }),
                    ),
                    module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("garment draw"),
                        source: wgpu::ShaderSource::Wgsl(WGSL.into()),
                    }),
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: crate::pipeline_cache::of(&device).as_ref(),
                })
            })
        };
        Self {
            pose: pipeline("pose"),
            scatter: pipeline("scatter"),
            layout,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        cache: &mut HashMap<(u64, usize), Mesh>,
        scene: &crate::RenderScene,
        draws: &[(u32, u32)],
        output: &wgpu::Buffer,
        previous: Option<u32>,
        visual: Option<&VisualPipelines>,
    ) {
        cache.retain(|key, _| {
            draws.iter().any(|&(d, _)| {
                scene.objects.deformed_vertices[d as usize]
                    .garment
                    .as_ref()
                    .is_some_and(|g| &g.key == key)
            })
        });
        for &(d, first) in draws {
            let g = scene.objects.deformed_vertices[d as usize]
                .garment
                .as_ref()
                .expect("garment draw");
            let rows = inputs(g, first, previous, scene.camera_cut);
            let bytes = bytemuck::cast_slice(&rows);
            let mesh = cache
                .entry(g.key)
                .or_insert_with(|| self.mesh(device, queue, g, bytes));
            if !Arc::ptr_eq(&mesh.topology, &g.topology) || mesh.input.size() != bytes.len() as u64
            {
                *mesh = self.mesh(device, queue, g, bytes);
            }
            let mut rows = rows;
            rows[1][3] = f32::from(!mesh.initialized || scene.camera_cut);
            queue.write_buffer(&mesh.input, 0, bytemuck::cast_slice(&rows));
            let entries: Vec<_> = [&mesh.input, &mesh.adjacency, &mesh.points, output]
                .into_iter()
                .enumerate()
                .map(|(i, b)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: b.as_entire_binding(),
                })
                .collect();
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("garment draw"),
                layout: &self.layout,
                entries: &entries,
            });
            // A visual garment's points are its solver's, from this frame's
            // skinned pose (its targets); the scatter then draws them.
            let solver = visual.filter(|v| simulate(device, queue, v, mesh, g, scene).is_some());
            let dispatch = |encoder: &mut crate::Encoder, pipeline, count: usize| {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("garment draw"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &group, &[]);
                let groups = (count as u32).div_ceil(64);
                pass.dispatch_workgroups(groups.min(65535), groups.div_ceil(65535), 1);
            };
            match solver {
                Some(v) => v.encode(encoder, mesh.visual.as_ref().expect("simulated"), true),
                None => dispatch(encoder, self.pose.get(), g.topology.rest.len()),
            }
            dispatch(encoder, self.scatter.get(), g.topology.vertices.len());
            mesh.initialized = true;
        }
    }

    fn mesh(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        g: &GarmentDeformation,
        input: &[u8],
    ) -> Mesh {
        let buffer = |label, data: &[u8]| {
            crate::buffer_init(
                device,
                queue,
                &wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents: data,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            )
        };
        Mesh {
            topology: g.topology.clone(),
            input: buffer("garment inputs", input),
            adjacency: buffer(
                "garment adjacency",
                bytemuck::cast_slice(&adjacency(&g.topology)),
            ),
            points: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("garment points and history"),
                size: (g.topology.rest.len() + g.topology.vertices.len()) as u64 * 16,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            initialized: false,
            visual: None,
        }
    }
}

/// Prepares a visual garment's solve this frame into `mesh.points`; None
/// for a garment the tick simulates, or one without its collider list or
/// shapes this frame (it then draws its skinned pose).
fn simulate(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipelines: &VisualPipelines,
    mesh: &mut Mesh,
    g: &GarmentDeformation,
    scene: &crate::RenderScene,
) -> Option<()> {
    let Some(body) = g.visual.as_deref() else {
        mesh.visual = None;
        return None;
    };
    let shapes = body.garment.as_ref()?;
    let n = g.topology.rest.len();
    let list = *pipelines.garment_lists.get(&g.key)?;
    if shapes.rest.len() != n || shapes.normals.len() != n || g.targets.len() != n {
        return None;
    }
    let triangles = &g.topology.triangles;
    if mesh
        .visual
        .as_ref()
        .is_none_or(|v| !v.topology.matches(None, body, &shapes.rest, triangles))
    {
        let topology = Topology::build(None, body, &shapes.rest, triangles);
        mesh.visual = Some(pipelines.mesh(device, queue, topology));
    }
    let dt = if scene.tick_rate > 0.0 {
        1.0 / scene.tick_rate
    } else {
        1.0 / 60.0
    };
    let Mesh { visual, points, .. } = mesh;
    pipelines.prepare(
        device,
        queue,
        visual.as_mut().expect("made above"),
        body,
        &g.targets,
        &shapes.normals,
        scene.tick,
        dt,
        list,
        g.world,
        points,
    );
    Some(())
}

fn inputs(g: &GarmentDeformation, first: u32, previous: Option<u32>, reset: bool) -> Vec<[f32; 4]> {
    let t = &g.topology;
    let mut rows = vec![
        [
            t.rest.len() as f32,
            t.vertices.len() as f32,
            g.palette.len() as f32,
            0.,
        ],
        [
            (first * 4) as f32,
            ((previous.unwrap_or(0) + first) * 4) as f32,
            f32::from(previous.is_some()),
            f32::from(reset),
        ],
    ];
    rows.extend(g.world.to_cols_array_2d());
    rows.extend(g.world.inverse().to_cols_array_2d());
    for ((p, n), (j, w)) in t.rest.iter().zip(&t.normals).zip(&t.skin) {
        rows.extend([
            p.extend(1.).to_array(),
            n.extend(0.).to_array(),
            j.map(f32::from),
            *w,
        ]);
    }
    for m in g.palette.iter() {
        rows.extend(m.to_cols_array_2d());
    }
    rows.extend(g.points.iter().map(|p| p.extend(0.).to_array()));
    rows.extend(g.targets.iter().map(|p| p.extend(0.).to_array()));
    rows
}
