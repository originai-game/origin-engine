//! The meshlet path's buffers and layouts, kept in step with the scene's clustered meshes.

use super::*;

impl MeshletGpu {
    pub fn new(
        device: &wgpu::Device,
        max_binding: u64,
        wind_field: wgpu::TextureView,
    ) -> MeshletGpu {
        let c = wgpu::ShaderStages::COMPUTE;
        let cull_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet cull"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: c,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, true, c),
                storage(2, true, c),
                storage(3, true, c),
                storage(4, true, c),
                storage(5, true, c),
                storage(6, false, c),
                storage(7, false, c),
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: c,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(9, false, c),
            ],
        });
        let v = wgpu::ShaderStages::VERTEX;
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet draw"),
            entries: &[
                uniform(8, v),
                storage(9, true, v),
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: v,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(11, true, v),
                storage(12, true, v),
                storage(13, true, v),
                storage(14, true, v),
                // The vertex pool's foliage regions (meshlet_sway.wgsl).
                storage(17, true, v),
                // The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
                wgpu::BindGroupLayoutEntry {
                    binding: 18,
                    visibility: v,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet cull"),
            source: wgpu::ShaderSource::Wgsl(MESHLET_CULL_WGSL.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet cull"),
            bind_group_layouts: &[Some(&cull_layout)],
            immediate_size: 0,
        });
        let cull = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("meshlet cull"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cull_clusters"),
            compilation_options: Default::default(),
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        let no_pyramid = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("meshlet: no depth pyramid"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let params = buffer(
            device,
            "meshlet params",
            PARAMS_SLOT * 2,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let depth_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet early depth"),
            entries: &[uniform(0, v)],
        });
        let depth_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meshlet early depth"),
            layout: &depth_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &params,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            }],
        });
        let depth_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("meshlet early depth"),
                bind_group_layouts: &[Some(&depth_layout), Some(&draw_layout)],
                immediate_size: 0,
            });
        let depth_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet early depth"),
            source: wgpu::ShaderSource::Wgsl(depth_source().into()),
        });
        MeshletGpu {
            cull_layout,
            cull,
            compact: compact::Compact::new(device),
            draw_layout,
            lit: HashMap::new(),
            prepass: HashMap::new(),
            lit_module: None,
            shadow: HashMap::new(),
            shadow_module: None,
            pools: None,
            params,
            vis: buffer(device, "meshlet visibility", 0, storage_usage()),
            vis_half: 0,
            vis_parity: false,
            vis_clear: true,
            depth_module,
            depth_pipeline_layout,
            depth_group,
            depth_pipelines: HashMap::new(),
            draw_params: buffer(
                device,
                "meshlet draw params",
                std::mem::size_of::<DrawParams>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            views: buffer(device, "meshlet views", 0, storage_usage()),
            batches: buffer(device, "meshlet batches", 0, storage_usage()),
            items: buffer(device, "meshlet items", 0, storage_usage()),
            draws: buffer(device, "meshlet draws", 0, Self::draws_usage()),
            no_pyramid,
            cull_group: None,
            mesh_groups: HashMap::new(),
            window: 0,
            frame: Vec::new(),
            max_binding,
            vis_buffer: vis::VisBuffer::new(device, wind_field.clone()),
            wind_field,
        }
    }

    pub(super) fn draws_usage() -> wgpu::BufferUsages {
        // COPY_SRC: the soft list's count, into the rasteriser's params.
        wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
    }

    /// Rebuilds the pools when the scene's clustered meshes changed.
    pub fn sync(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, scene: &RenderScene) {
        let key: Vec<(usize, Arc<MeshClusters>)> = scene
            .mesh_clusters_all()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.clone().map(|c| (i, c)))
            .collect();
        let foliage_all = scene.mesh_foliage_all();
        let foliage_key: Vec<Option<Arc<crate::meshlet::MeshFoliage>>> = key
            .iter()
            .map(|(i, _)| foliage_all.get(*i).cloned().flatten())
            .collect();
        let same = self.pools.as_ref().is_some_and(|p| {
            p.key.len() == key.len()
                && p.key
                    .iter()
                    .zip(&key)
                    .all(|(a, b)| a.0 == b.0 && Arc::ptr_eq(&a.1, &b.1))
                && p.foliage_key.len() == foliage_key.len()
                && p.foliage_key
                    .iter()
                    .zip(&foliage_key)
                    .all(|(a, b)| match (a, b) {
                        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    })
        });
        if same {
            return;
        }
        let mut slots = HashMap::new();
        let mut mesh_rows: Vec<[u32; 4]> = Vec::new();
        let mut cluster_rows: Vec<[u32; 4]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<crate::Vertex> = Vec::new();
        // The foliage regions (after every vertex) and, per foliage mesh,
        // its cluster rows' range and its region in `regions`.
        let mut regions: Vec<[f32; 4]> = Vec::new();
        let mut pointed: Vec<(std::ops::Range<usize>, u32)> = Vec::new();
        let mut foliage = Vec::new();
        for ((i, c), f) in key.iter().zip(&foliage_key) {
            let Some(source) = scene.meshes().get(*i) else {
                continue;
            };
            let Some((mesh, rows)) = pool_rows(
                c,
                indices.len() as u32,
                (cluster_rows.len() / CLUSTER_ROWS) as u32,
                vertices.len() as u32,
            ) else {
                continue;
            };
            let f_rows = f.as_ref().map(|f| f.pool_rows());
            let bytes = ((indices.len() + c.indices.len()) * 4)
                .max((cluster_rows.len() + rows.len()) * 16)
                .max(
                    (vertices.len() + source.vertices.len()) * 64
                        + (regions.len() + f_rows.as_ref().map_or(0, Vec::len)) * 16,
                ) as u64;
            if bytes > self.max_binding {
                break;
            }
            // As the mesh's own buffer holds them (uploads.rs): with
            // tangents generated where it has none.
            if source.lacks_tangents() {
                let mut m = source.clone();
                m.generate_tangents();
                vertices.extend_from_slice(&m.vertices);
            } else {
                vertices.extend_from_slice(&source.vertices);
            }
            let most = c
                .levels
                .iter()
                .map(|l| l.clusters.end - l.clusters.start)
                .max()
                .unwrap_or(0);
            let slot = (mesh_rows.len() / MESH_ROWS) as u32;
            slots.insert(*i, (slot, most));
            if let (Some(f), Some(f_rows)) = (f, f_rows) {
                let at = cluster_rows.len();
                pointed.push((at..at + rows.len(), regions.len() as u32));
                regions.extend(f_rows);
                let positions = source.vertices.iter().map(|v| Vec3::from(v.position));
                foliage.push((slot, f.bounds(positions), 0.0));
            }
            mesh_rows.extend_from_slice(&mesh);
            cluster_rows.extend(rows);
            indices.extend_from_slice(&c.indices);
        }
        // The regions follow the vertices (four vec4s each).
        for (range, region) in pointed {
            pool_foliage(&mut cluster_rows[range], vertices.len() as u32 * 4 + region);
        }
        let mut pool: Vec<u8> = bytemuck::cast_slice(&vertices).to_vec();
        pool.extend_from_slice(bytemuck::cast_slice(&regions));
        let make = |label: &str, bytes: &[u8]| {
            let b = buffer(device, label, bytes.len() as u64, storage_usage());
            queue.write_buffer(&b, 0, bytes);
            b
        };
        self.pools = Some(Pools {
            key,
            foliage_key,
            foliage,
            slots,
            meshes: make("meshlet meshes", bytemuck::cast_slice(&mesh_rows)),
            clusters: make("meshlet clusters", bytemuck::cast_slice(&cluster_rows)),
            indices: make("meshlet indices", bytemuck::cast_slice(&indices)),
            vertices: make("meshlet vertex pool", &pool),
        });
        self.cull_group = None;
        self.mesh_groups.clear();
    }

    /// Whether `mesh` is in the pools (clustered, and within their limits).
    pub fn clustered(&self, mesh: usize) -> bool {
        self.pools
            .as_ref()
            .is_some_and(|p| p.slots.contains_key(&mesh))
    }
}
