use super::*;

impl RenderScene {
    /// Adds a mesh with a per-vertex skin (four joints and weights each),
    /// drawn posed by [`RenderScene::skinned_instances`]. The skin must have
    /// one entry per vertex ([`crate::RenderError::InvalidData`] at upload).
    pub fn add_skinned_mesh(&mut self, mesh: MeshData, skin: SkinData) -> MeshHandle {
        // A new slot: the renderer sets a skin up once, for a new mesh.
        let h = self.push_mesh(mesh, false);
        self.skins[h.index()] = Some(skin);
        h
    }

    /// Gives a mesh blendshape targets (one delta per vertex per target, at
    /// most [`crate::MAX_MORPH_TARGETS`]); draw it with
    /// [`RenderScene::skinned_instances`] (weights per instance; a mesh
    /// without a skin stays in its bind pose). Checked at upload. Set them
    /// before the mesh's first frame; `false` for a handle of another scene.
    pub fn set_morph_targets(&mut self, mesh: MeshHandle, targets: MorphTargets) -> bool {
        if !self.mesh_slots.live(mesh.index(), mesh.generation()) {
            return false;
        }
        match self.morphs.get_mut(mesh.index()) {
            Some(m) => {
                *m = Some(targets);
                true
            }
            None => false,
        }
    }

    pub(crate) fn morphs(&self) -> &[Option<MorphTargets>] {
        &self.morphs
    }

    pub fn skin(&self, handle: MeshHandle) -> Option<&SkinData> {
        self.skins.get(handle.index())?.as_ref()
    }

    pub(crate) fn skins(&self) -> &[Option<SkinData>] {
        &self.skins
    }

    pub fn add_mesh(&mut self, mesh: MeshData) -> MeshHandle {
        self.push_mesh(mesh, true)
    }

    fn push_mesh(&mut self, mesh: MeshData, reuse: bool) -> MeshHandle {
        let (i, generation, reused) = self.mesh_slots.take(self.meshes.len(), reuse);
        if reused {
            self.meshes[i] = mesh;
            self.mesh_revisions[i] += 1;
            self.mesh_encodings[i] = VertexEncoding::Full;
            self.mesh_clusters[i] = None;
            self.mesh_foliage[i] = None;
            self.mesh_surfaces[i] = None;
            self.mesh_visual[i] = None;
        } else {
            self.meshes.push(mesh);
            self.skins.push(None);
            self.morphs.push(None);
            self.mesh_revisions.push(0);
            self.mesh_encodings.push(VertexEncoding::Full);
            self.mesh_clusters.push(None);
            self.mesh_foliage.push(None);
            self.mesh_surfaces.push(None);
            self.mesh_visual.push(None);
        }
        MeshHandle::from_parts(i, generation)
    }

    /// Removes a mesh: the renderer frees its buffers before the next frame
    /// and a later [`RenderScene::add_mesh`] reuses the slot under a new
    /// generation, so this handle (and every copy of it) is refused from
    /// now on: by `set_mesh` and the other setters, and as an instance's
    /// mesh ([`crate::RenderError::InvalidHandle`]). A skinned or morphed
    /// mesh's slot is not reused (its skin and blendshape data stay with
    /// the renderer). False for a handle that is not live.
    pub fn remove_mesh(&mut self, handle: MeshHandle) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation()) {
            return false;
        }
        let reuse = self.skins[i].is_none() && self.morphs[i].is_none();
        self.meshes[i] = MeshData::default();
        self.mesh_revisions[i] += 1;
        self.mesh_clusters[i] = None;
        self.mesh_foliage[i] = None;
        self.mesh_surfaces[i] = None;
        self.mesh_visual[i] = None;
        self.mesh_slots.remove(i, reuse);
        true
    }

    /// Whether `handle` names a mesh of this scene that was not removed.
    pub fn contains_mesh(&self, handle: MeshHandle) -> bool {
        self.mesh_slots.live(handle.index(), handle.generation())
    }

    /// Replaces a mesh's vertices and indices (cloth, generated geometry);
    /// the renderer uploads it again before the next frame, in place when
    /// the sizes match. False for a bad handle or a skinned or morphed mesh.
    pub fn set_mesh(&mut self, handle: MeshHandle, mesh: MeshData) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation())
            || self.skins[i].is_some()
            || self.morphs[i].is_some()
        {
            return false;
        }
        self.meshes[i] = mesh;
        self.mesh_revisions[i] += 1;
        // Its clusters described the old indices, its foliage the old vertices.
        self.mesh_clusters[i] = None;
        self.mesh_foliage[i] = None;
        self.mesh_surfaces[i] = None;
        self.mesh_visual[i] = None;
        true
    }

    /// Replaces a simulation's coarse draw mesh. The renderer computes its
    /// normals and finer surface without changing this geometry or the solve.
    /// Unsupported devices draw these coarse vertices with CPU normals.
    pub fn set_simulation_surface(
        &mut self,
        handle: MeshHandle,
        mesh: MeshData,
        kind: crate::SimulationSurface,
    ) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation())
            || self.mesh_clusters[i].is_some()
            || self.mesh_foliage[i].is_some()
            || !mesh.indices.len().is_multiple_of(3)
            || mesh
                .indices
                .iter()
                .any(|&i| i as usize >= mesh.vertices.len())
        {
            return false;
        }
        if let crate::SimulationSurface::Cloth { columns, rows } = kind
            && (columns < 2
                || rows < 2
                || u64::from(columns) * u64::from(rows) != mesh.vertices.len() as u64)
        {
            return false;
        }
        if !self.set_mesh(handle, mesh) {
            return false;
        }
        self.mesh_encodings[handle.index()] = VertexEncoding::Full;
        self.mesh_surfaces[handle.index()] = Some(kind);
        true
    }

    /// Makes a simulation surface a visual body: the renderer simulates it
    /// on the GPU from the coarse mesh last given to
    /// [`RenderScene::set_simulation_surface`], which is then its rest
    /// shape and its pinned points' targets this tick, and draws that
    /// simulation (never read back). `None` draws the mesh as given again.
    /// Cleared with the surface (give it after each
    /// `set_simulation_surface`). False unless `handle` is a live
    /// simulation surface.
    pub fn set_visual_body(
        &mut self,
        handle: MeshHandle,
        body: Option<std::sync::Arc<crate::VisualBody>>,
    ) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation()) || self.mesh_surfaces[i].is_none() {
            return false;
        }
        self.mesh_visual[i] = body;
        true
    }

    /// Whether the frame has a visual body ([`RenderScene::set_visual_body`]
    /// or a garment's [`crate::GarmentDeformation::visual`]): the host then
    /// gives [`RenderScene::colliders`].
    pub fn has_visual_bodies(&self) -> bool {
        self.mesh_visual.iter().any(Option::is_some)
            || self
                .objects
                .deformed_vertices
                .iter()
                .any(|d| d.garment.as_ref().is_some_and(|g| g.visual.is_some()))
    }

    /// Gives a mesh its levels cut into clusters (the cook's: og-bundle's
    /// `CookedPrimitive::mesh_clusters`), or takes them away with `None`.
    /// Its instances then draw through the meshlet path (`meshlet.rs`)
    /// where their material allows ([`RenderScene::meshlet_drawable`]),
    /// which picks each instance's level and culls its clusters on the GPU;
    /// the mesh's own indices stay level 0 for every other pass. Refused
    /// (false) for a bad handle, a skinned or morphed mesh, clusters that
    /// do not describe their indices ([`crate::meshlet::MeshClusters::validate`])
    /// or indices past the mesh's vertices. [`RenderScene::set_mesh`]
    /// drops them.
    pub fn set_mesh_clusters(
        &mut self,
        handle: MeshHandle,
        clusters: Option<crate::meshlet::MeshClusters>,
    ) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation())
            || self.skins[i].is_some()
            || self.morphs[i].is_some()
            || self.mesh_surfaces[i].is_some()
        {
            return false;
        }
        if let Some(c) = &clusters {
            let vertices = self.meshes[i].vertices.len() as u32;
            if c.validate(c.indices.len() as u32).is_err()
                || c.indices.iter().any(|&v| v >= vertices)
            {
                return false;
            }
        }
        self.mesh_clusters[i] = clusters.map(std::sync::Arc::new);
        true
    }

    /// A mesh's clusters ([`RenderScene::set_mesh_clusters`]).
    pub fn mesh_clusters(&self, handle: MeshHandle) -> Option<&crate::meshlet::MeshClusters> {
        self.contains_mesh(handle)
            .then(|| self.mesh_clusters[handle.index()].as_deref())
            .flatten()
    }

    /// Gives a clustered mesh its foliage deformation
    /// ([`crate::meshlet::MeshFoliage`]: each vertex's branch pivot and leaf
    /// anchor), or takes it away with `None`: on the meshlet path its
    /// vertices then sway in [`RenderScene::wind`]. Refused (false) for a
    /// bad handle, a skinned or morphed mesh, or foliage that does not
    /// describe its vertices ([`crate::meshlet::MeshFoliage::validate`]).
    /// [`RenderScene::set_mesh`] drops it.
    pub fn set_mesh_foliage(
        &mut self,
        handle: MeshHandle,
        foliage: Option<crate::meshlet::MeshFoliage>,
    ) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation())
            || self.skins[i].is_some()
            || self.morphs[i].is_some()
            || self.mesh_surfaces[i].is_some()
        {
            return false;
        }
        if let Some(f) = &foliage
            && f.validate(self.meshes[i].vertices.len()).is_err()
        {
            return false;
        }
        self.mesh_foliage[i] = foliage.map(std::sync::Arc::new);
        true
    }

    /// A mesh's foliage ([`RenderScene::set_mesh_foliage`]).
    pub fn mesh_foliage(&self, handle: MeshHandle) -> Option<&crate::meshlet::MeshFoliage> {
        self.contains_mesh(handle)
            .then(|| self.mesh_foliage[handle.index()].as_deref())
            .flatten()
    }

    pub(crate) fn mesh_foliage_all(
        &self,
    ) -> &[Option<std::sync::Arc<crate::meshlet::MeshFoliage>>] {
        &self.mesh_foliage
    }

    pub(crate) fn mesh_clusters_all(
        &self,
    ) -> &[Option<std::sync::Arc<crate::meshlet::MeshClusters>>] {
        &self.mesh_clusters
    }

    /// Whether instances of `mesh` with `material` draw through the meshlet
    /// path: the scene's [`RenderScene::meshlets`] on, the mesh clustered,
    /// and the material one the path draws (opaque, standard: no physical
    /// lobes, material shader, texture array or wireframe; a lightmapped
    /// mesh is decided by the renderer). Instances it refuses draw their
    /// mesh's level 0 as any mesh does, so a caller picking levels on the
    /// CPU (og-runtime's per-entity LOD) keeps doing so for them.
    pub fn meshlet_drawable(&self, mesh: MeshHandle, material: MaterialHandle) -> bool {
        self.objects.meshlets
            && self.mesh_clusters(mesh).is_some()
            && self.contains_material(material)
            && {
                let i = material.index();
                let m = &self.materials[i];
                !m.is_transparent()
                    && !m.is_physical()
                    && !m.draws_array_maps()
                    && m.wireframe != 1
                    && self.material_shaders.get(i).is_none_or(Option::is_none)
            }
    }

    pub(crate) fn mesh_revisions(&self) -> &[u64] {
        &self.mesh_revisions
    }

    /// Sets how a mesh's vertices are held on the GPU (default
    /// [`VertexEncoding::Full`]); the renderer uploads it again before the
    /// next frame when this changes it. False for a bad handle.
    pub fn set_vertex_encoding(&mut self, handle: MeshHandle, encoding: VertexEncoding) -> bool {
        let i = handle.index();
        if !self.mesh_slots.live(i, handle.generation()) || self.mesh_surfaces[i].is_some() {
            return false;
        }
        if self.mesh_encodings[i] != encoding {
            self.mesh_encodings[i] = encoding;
            self.mesh_revisions[i] += 1;
        }
        true
    }

    /// A mesh's [`VertexEncoding`] (`None`: a bad handle).
    pub fn vertex_encoding(&self, handle: MeshHandle) -> Option<VertexEncoding> {
        self.contains_mesh(handle)
            .then(|| self.mesh_encodings[handle.index()])
    }

    pub(crate) fn mesh_encodings(&self) -> &[VertexEncoding] {
        &self.mesh_encodings
    }
}
