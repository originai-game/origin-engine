//! Mesh uploads; textures, materials and a frame's shadow layers and
//! environment in the modules below.

use super::*;

mod frame_resources;
mod materials;
mod textures;

impl Renderer {
    /// The f32 vertices of `mesh` on the GPU (its tangents generated where
    /// it lacks them); a 16-byte placeholder for a mesh without vertices.
    fn upload_vertices(&self, mesh: &MeshData) -> wgpu::Buffer {
        let tangents;
        let mesh = if mesh.lacks_tangents() {
            let mut m = mesh.clone();
            m.generate_tangents();
            tangents = m;
            &tangents
        } else {
            mesh
        };
        let bytes: &[u8] = bytemuck::cast_slice(&mesh.vertices);
        // Fork s74: on the web written with writeBuffer straight from wasm
        // memory (buffer_init): no 1.8 GB of short-lived ArrayBuffers in a
        // 1072-cell boot.
        crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: if bytes.is_empty() { &[0u8; 16] } else { bytes },
                // STORAGE: the meshlet path pulls them (meshlet.wgsl).
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            },
        )
    }

    /// `mesh` on the GPU; `compact`: as [`VertexEncoding::Compact`], whose
    /// f32 vertices wait for a pipeline that reads them.
    pub(crate) fn upload_mesh(&self, mesh: &MeshData, compact: bool) -> GpuMesh {
        // Never create an empty buffer: an empty mesh keeps a 16-byte placeholder and draws nothing.
        let compact = compact
            .then(|| compact::encode(mesh))
            .flatten()
            .map(|(v, dq)| {
                let buffer = crate::buffer_init(
                    &self.device,
                    &self.queue,
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("compact vertices"),
                        contents: bytemuck::cast_slice(&v),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );
                (buffer, dq)
            });
        let full = compact.is_none();
        let vertices = if full {
            self.upload_vertices(mesh)
        } else {
            self.upload_vertices(&MeshData::default())
        };
        let indices = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("indices"),
                contents: if mesh.indices.is_empty() {
                    &[0u8; 16]
                } else {
                    bytemuck::cast_slice(&mesh.indices)
                },
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            },
        );
        let bounds = mesh
            .bounds()
            .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
        GpuMesh {
            vertices,
            full,
            compact,
            indices,
            index_count: if mesh.vertices.is_empty() {
                0
            } else {
                mesh.indices.len() as u32
            },
            bounds,
            revision: 0,
            foliage: None,
            surface: None,
        }
    }

    pub(crate) fn upload_surface(
        &self,
        mesh: &MeshData,
        kind: SimulationSurface,
        level: u32,
    ) -> GpuMesh {
        let count = mesh.indices.len() as u64 * u64::from(level * level);
        let bytes = count * std::mem::size_of::<Vertex>() as u64;
        let Some(pipelines) =
            self.world.surfaces.as_ref().filter(|_| {
                bytes > 0 && bytes <= self.device.limits().max_storage_buffer_binding_size
            })
        else {
            return self.upload_mesh(&surface::fallback(mesh, kind), false);
        };
        let vertices = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("simulation surface vertices"),
            size: bytes,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let indices: Vec<u32> = (0..count as u32).collect();
        let indices = crate::buffer_init(
            &self.device,
            &self.queue,
            &wgpu::util::BufferInitDescriptor {
                label: Some("simulation surface indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        let surface = pipelines.mesh(&self.device, &self.queue, mesh, &vertices, level);
        GpuMesh {
            vertices,
            indices,
            index_count: count as u32,
            full: true,
            compact: None,
            bounds: surface::bounds(mesh),
            revision: 0,
            foliage: None,
            surface: Some(surface),
        }
    }

    /// Use the same conservative box for CPU sorting and every GPU cull pass.
    /// A shared mesh takes the largest current/previous sway of its instances;
    /// this avoids adding a per-instance cull stream just for the margin.
    pub(crate) fn refresh_foliage_bounds(&mut self, scene: &RenderScene) {
        let winds = crate::foliage::foliage_winds(
            scene,
            self.history.previous_wind,
            crate::wind_field::NO_GRID,
        )
        .0;
        let mut sway = vec![0.0f32; self.cache.meshes.len()];
        let previous = if self.instances.previous_sway.len() == scene.objects.instances.len() {
            self.instances.previous_sway.as_slice()
        } else {
            &[]
        };
        for (i, inst) in scene.objects.instances.iter().enumerate() {
            if scene.mesh_foliage(inst.mesh).is_some() {
                sway[inst.mesh.index()] = sway[inst.mesh.index()]
                    .max(inst.sway)
                    .max(previous.get(i).copied().unwrap_or(inst.sway));
            }
        }
        for (i, mesh) in self.cache.meshes.iter_mut().enumerate() {
            let Some(source) = &scene.mesh_foliage_all()[i] else {
                if mesh.foliage.take().is_some() {
                    // set_mesh may have changed the geometry as well.
                    mesh.bounds = scene.meshes()[i]
                        .bounds()
                        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
                }
                continue;
            };
            if mesh
                .foliage
                .as_ref()
                .is_none_or(|f| !std::sync::Arc::ptr_eq(&f.source, source))
            {
                let data = &scene.meshes()[i];
                mesh.foliage = Some(FoliageMesh {
                    source: source.clone(),
                    rest: data
                        .bounds()
                        .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5)),
                    deformation: source
                        .bounds(data.vertices.iter().map(|v| Vec3::from_array(v.position))),
                });
            }
            let f = mesh.foliage.as_ref().unwrap();
            let margin = winds
                .iter()
                .map(|w| {
                    f.deformation.deform_margin(
                        crate::wind_field::lean_cap(Some(*w).filter(|w| w.strength > 0.0), false)
                            .max(self.world.wind_field.lean_cap()),
                    )
                })
                .fold(0.0f32, f32::max)
                * sway[i].max(1.0);
            mesh.bounds = f
                .rest
                .map(|(center, half)| (center, half + Vec3::splat(margin)));
        }
    }

    /// Plans and writes the frame of every visual body: the simulation
    /// surfaces the scene made visual ([`RenderScene::set_visual_body`]),
    /// their rest shapes this tick, the colliders near them and their
    /// conservative bounds. After [`Renderer::refresh_meshes`].
    pub(crate) fn refresh_visual(&mut self, scene: &RenderScene) {
        let Renderer {
            world,
            cache,
            device,
            queue,
            ..
        } = self;
        let Some(pipelines) = world.visual.as_mut() else {
            return;
        };
        let mut bodies = Vec::new();
        for (i, gpu) in cache.meshes.iter_mut().enumerate() {
            let Some(surface) = gpu.surface.as_mut() else {
                continue;
            };
            let Some(body) = scene.mesh_visual.get(i).and_then(Option::as_ref) else {
                surface.visual = None;
                continue;
            };
            let kind = scene.mesh_surfaces[i];
            let (rest, triangles) = crate::visual_sim::rest_of(&scene.meshes()[i]);
            if surface
                .visual
                .as_ref()
                .is_none_or(|v| !v.topology.matches(kind, body, &rest, &triangles))
            {
                let topology = crate::visual_sim::Topology::build(kind, body, &rest, &triangles);
                surface.visual = Some(pipelines.mesh(device, queue, topology));
            }
            let v = surface.visual.as_mut().expect("made above");
            v.bounds = crate::visual_sim::grown_bounds(&rest, v.topology.margin);
            let (lo, hi) = v.bounds;
            gpu.bounds = Some(((lo + hi) * 0.5, (hi - lo) * 0.5));
            bodies.push((i, rest));
        }
        // Visual garments: within their reach of this frame's skinned pose.
        let garments: Vec<_> = scene
            .objects
            .deformed_vertices
            .iter()
            .filter_map(|d| d.garment.as_ref())
            .filter_map(|g| {
                let body = g.visual.as_ref()?;
                let reach = body.reach.iter().fold(0.0f32, |m, &r| m.max(r));
                let margin = reach + body.radius.max(0.0) + body.backstop.max(0.0);
                Some((g.key, crate::visual_sim::grown_bounds(&g.targets, margin)))
            })
            .collect();
        pipelines.garment_lists.clear();
        if bodies.is_empty() && garments.is_empty() {
            return;
        }
        let mut bounds: Vec<_> = bodies
            .iter()
            .map(|&(i, _)| {
                let s = cache.meshes[i].surface.as_ref().expect("a surface");
                s.visual.as_ref().expect("a visual body").bounds
            })
            .collect();
        bounds.extend(garments.iter().map(|&(_, b)| b));
        let mut lists = pipelines.write_colliders(device, queue, &scene.world.colliders, &bounds);
        let garment_lists = lists.split_off(bodies.len());
        pipelines.garment_lists = garments
            .iter()
            .map(|&(key, _)| key)
            .zip(garment_lists)
            .collect();
        let dt = if scene.frame.tick_rate > 0.0 {
            1.0 / scene.frame.tick_rate
        } else {
            1.0 / 60.0
        };
        for ((i, rest), list) in bodies.into_iter().zip(lists) {
            let body = scene.mesh_visual[i].as_ref().expect("visual");
            let surface = cache.meshes[i].surface.as_mut().expect("a surface");
            let crate::surface::SurfaceMesh { source, visual, .. } = surface;
            pipelines.prepare(
                device,
                queue,
                visual.as_mut().expect("a visual body"),
                body,
                &rest,
                &[],
                scene.frame.tick,
                dt,
                list,
                Mat4::IDENTITY,
                source,
            );
        }
    }

    /// Uploads meshes the scene replaced since they were last uploaded: in
    /// place when the sizes match, else as new buffers.
    pub(crate) fn refresh_meshes(&mut self, scene: &RenderScene) {
        for (i, &revision) in scene.mesh_revisions().iter().enumerate() {
            let Some(gpu) = self.cache.meshes.get(i) else {
                break;
            };
            let source = &scene.meshes()[i];
            if let Some(kind) = scene.mesh_surfaces[i] {
                let level = surface::subdivisions(scene.objects.cloth_subdivisions);
                if let Some(surface) = gpu.surface.as_ref().filter(|s| s.matches(source, level)) {
                    // A visual body's vertices are its GPU simulation's.
                    if gpu.revision != revision && scene.mesh_visual[i].is_none() {
                        surface.update(&self.queue, source);
                    }
                    let gpu = &mut self.cache.meshes[i];
                    gpu.bounds = surface::bounds(source);
                    gpu.revision = revision;
                } else {
                    let mut fresh = self.upload_surface(source, kind, level);
                    fresh.revision = revision;
                    self.cache.meshes[i] = fresh;
                }
                continue;
            }
            if gpu.surface.is_some() {
                let mut fresh = self.upload_mesh(source, false);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
                continue;
            }
            if gpu.revision == revision {
                continue;
            }
            if let Some(k) = &scene.skins()[i] {
                let positions: Vec<[f32; 3]> = source.vertices.iter().map(|v| v.position).collect();
                self.cache.joint_boxes[i] = skin::joint_boxes(&positions, k);
            }
            let compact = scene.mesh_encodings()[i] == VertexEncoding::Compact
                && scene.skins()[i].is_none()
                && scene.morphs()[i].is_none();
            if compact || gpu.compact.is_some() {
                let mut fresh = self.upload_mesh(source, compact);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
                continue;
            }
            let tangents;
            let mesh = if source.lacks_tangents() {
                let mut m = source.clone();
                m.generate_tangents();
                tangents = m;
                &tangents
            } else {
                source
            };
            let vb: &[u8] = bytemuck::cast_slice(&mesh.vertices);
            let ib: &[u8] = bytemuck::cast_slice(&mesh.indices);
            if !vb.is_empty()
                && vb.len() as u64 == gpu.vertices.size()
                && ib.len() as u64 == gpu.indices.size()
            {
                self.queue.write_buffer(&gpu.vertices, 0, vb);
                self.queue.write_buffer(&gpu.indices, 0, ib);
                let bounds = mesh
                    .bounds()
                    .map(|(lo, hi)| ((lo + hi) * 0.5, (hi - lo) * 0.5));
                let gpu = &mut self.cache.meshes[i];
                gpu.bounds = bounds;
                gpu.revision = revision;
            } else {
                let mut fresh = self.upload_mesh(mesh, false);
                fresh.revision = revision;
                self.cache.meshes[i] = fresh;
            }
        }
    }

    /// Uploads a compact mesh's f32 vertices, for a pipeline that reads
    /// [`Vertex`] (a material that is not plain, a scatter, a skinned
    /// instance); nothing for a mesh that has them.
    pub(crate) fn ensure_full(&mut self, scene: &RenderScene, mesh: usize) {
        if self.cache.meshes.get(mesh).is_none_or(|m| m.full) {
            return;
        }
        let vertices = self.upload_vertices(&scene.meshes()[mesh]);
        let gpu = &mut self.cache.meshes[mesh];
        gpu.vertices = vertices;
        gpu.full = true;
    }
}
