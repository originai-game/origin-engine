//! Scene resource storage and mutation.

use super::*;

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderScene {
    /// Whether the renderer captures the sky as it draws it into the
    /// image-based light (the live sky, `sky_capture.rs`): the per-pixel
    /// procedural sky with no environment map lighting the scene. Its
    /// clouds are then in that light already ([`crate::Clouds::light_scene`]).
    pub fn sky_captured(&self) -> bool {
        self.environment_map.is_none()
            && matches!(self.environment.background, Background::Procedural { .. })
    }

    pub fn new() -> Self {
        RenderScene {
            id: NEXT_SCENE_ID.fetch_add(1, Ordering::Relaxed),
            meshes: Vec::new(),
            mesh_revisions: Vec::new(),
            mesh_encodings: Vec::new(),
            mesh_clusters: Vec::new(),
            mesh_foliage: Vec::new(),
            mesh_surfaces: Vec::new(),
            mesh_visual: Vec::new(),
            cloth_subdivisions: 0,
            textures: Vec::new(),
            texture_revisions: Vec::new(),
            materials: Vec::new(),
            material_shaders: Vec::new(),
            shaders: Vec::new(),
            gltf_materials: Vec::new(),
            material_revisions: Vec::new(),
            material_aliases: Vec::new(),
            fonts: Vec::new(),
            font_revisions: Vec::new(),
            instances: Vec::new(),
            camera: Camera::default(),
            hemisphere: ([0.0; 3], [0.0; 3], 0.0),
            light: DirectionalLight::default(),
            lights: Vec::new(),
            environment: Environment::default(),
            environment_map: None,
            post: PostEffects::default(),
            stylise: crate::Stylise::default(),
            sprites: Vec::new(),
            texts: Vec::new(),
            debug: DebugView::None,
            skins: Vec::new(),
            morphs: Vec::new(),
            emitters: Vec::new(),
            effect_materials: Vec::new(),
            luts: Vec::new(),
            lut_revisions: Vec::new(),
            palettes: Vec::new(),
            conformed: Vec::new(),
            conformed_revision: 0,
            palette_revisions: Vec::new(),
            environment_maps: Vec::new(),
            environment_map_revisions: Vec::new(),
            environment_equirects: Vec::new(),
            texture_arrays: Vec::new(),
            texture_array_revisions: Vec::new(),
            texture_array_filters: Vec::new(),
            trail_styles: Vec::new(),
            terrain_data: Vec::new(),
            terrain_revisions: Vec::new(),
            terrain_edits: Vec::new(),
            instance_layers: Vec::new(),
            skinned_layers: Vec::new(),
            sprite_layers: Vec::new(),
            sprite_light_masks: Vec::new(),
            sprite_facing: Vec::new(),
            text_facing: Vec::new(),
            line_layers: Vec::new(),
            point_layers: Vec::new(),
            particle_layers: Vec::new(),
            camera_layers: 1,
            meshlets: true,
            lod_bias: 1.0,
            lod_debug: false,
            visibility_buffer: true,
            posts: Vec::new(),
            post_params: Vec::new(),
            ui_vectors: Vec::new(),
            ui_materials: Vec::new(),
            custom_post: Vec::new(),
            origin: glam::Vec3::ZERO,
            views: Vec::new(),
            tick: 0,
            tick_rate: 60.0,
            frame_seconds: 1.0 / 60.0,
            interpolation: None,
            particles: Vec::new(),
            particle_history: Default::default(),
            colliders: Vec::new(),
            debris: Vec::new(),
            hosted_particles: Vec::new(),
            trails: Vec::new(),
            ribbons: Vec::new(),
            lines: Vec::new(),
            points: Vec::new(),
            css_height: 0.0,
            beams: Vec::new(),
            effect_meshes: Vec::new(),
            decals: Vec::new(),
            decal_atlas: None,
            lighting_2d: None,
            lights_2d: Vec::new(),
            occluders_2d: Vec::new(),
            sprite_fx: Vec::new(),
            sprite_meshes: Vec::new(),
            reflection_probes: Vec::new(),
            shake: CameraShake::default(),
            previous_transforms: Vec::new(),
            instance_levels: Vec::new(),
            previous_joint_palettes: Vec::new(),
            previous_morph_weights: Vec::new(),
            baked_clips: Vec::new(),
            crowds: Vec::new(),
            deformed_vertices: Vec::new(),
            previous_deformed_vertices: Vec::new(),
            camera_cut: false,
            joint_palettes: Vec::new(),
            skinned_instances: Vec::new(),
            overlay: None,
            fog: None,
            clouds: None,
            aurora: None,
            adaptation: 1.0,
            surface_weather: None,
            volumetric_fog: None,
            quality: crate::RenderQuality::default(),
            baked: crate::BakedLighting::default(),
            light_volumes: Vec::new(),
            light_cookies: Vec::new(),
            water: Vec::new(),
            terrains: Vec::new(),
            terrain_stamps: Vec::new(),
            scatters: Vec::new(),
            grass: Vec::new(),
            impostors: Vec::new(),
            wind: None,
            wind_motors: Vec::new(),
            grass_pushers: Vec::new(),
            water_2d: Vec::new(),
            metaballs: Vec::new(),
            fluids_3d: Vec::new(),
            sprite_order: Vec::new(),
            text_outlines: Vec::new(),
            tile_layers: Vec::new(),
            trail_layers: Vec::new(),
            sprite_arrays: Vec::new(),
            sprite_shadows: Vec::new(),
            sprite_outlines: Vec::new(),
            sprite_shading: Vec::new(),
            texture_filters: Vec::new(),
            texture_sizes: Default::default(),
            texture_normals: Vec::new(),
            paletted: std::collections::HashMap::new(),
            mesh_slots: Slots::default(),
            texture_slots: Slots::default(),
            stream_frames: Default::default(),
            stream_serial: 0,
            #[cfg(target_arch = "wasm32")]
            stream_external: Default::default(),
            material_slots: Slots::default(),
            font_slots: Slots::default(),
            environment_map_slots: Slots::default(),
            texture_array_slots: Slots::default(),
            palette_slots: Slots::default(),
            lut_slots: Slots::default(),
        }
    }

    /// Samples a texture of this scene nearest (pixel art) or linear (the
    /// default) wherever sprites, tiles and particles draw it. `false` for
    /// a handle of another scene.
    pub fn set_texture_filter(&mut self, texture: TextureHandle, filter: TextureFilter) -> bool {
        let i = texture.index();
        if !self.texture_slots.live(i, texture.generation()) {
            return false;
        }
        if self.texture_filters.len() <= i {
            self.texture_filters.resize(i + 1, TextureFilter::Linear);
        }
        self.texture_filters[i] = filter;
        true
    }

    /// Shades `texture` with the tangent-space normal map `normal` wherever
    /// a lit 2D layer draws it (sprites, tiles, 2D rig meshes): red towards
    /// +u, green up the image. `None` lights it flat. `false` for a handle
    /// of another scene.
    pub fn set_texture_normal(
        &mut self,
        texture: TextureHandle,
        normal: Option<TextureHandle>,
    ) -> bool {
        let i = texture.index();
        if !self.texture_slots.live(i, texture.generation())
            || normal.is_some_and(|n| !self.texture_slots.live(n.index(), n.generation()))
        {
            return false;
        }
        if self.texture_normals.len() <= i {
            self.texture_normals.resize(i + 1, None);
        }
        self.texture_normals[i] = normal;
        true
    }

    /// `texture`'s normal map, if any.
    pub fn texture_normal(&self, texture: TextureHandle) -> Option<TextureHandle> {
        self.texture_normals.get(texture.index()).copied().flatten()
    }

    /// Whether any texture has a normal map (the light map then keeps its
    /// light's direction too).
    pub fn has_texture_normals(&self) -> bool {
        self.texture_normals.iter().any(Option::is_some)
    }

    /// How sprites sample `texture`.
    pub fn texture_filter(&self, texture: TextureHandle) -> TextureFilter {
        self.texture_filters
            .get(texture.index())
            .copied()
            .unwrap_or_default()
    }

    /// A palette swap (docs/art.md §2.4): the texture `index` shows through
    /// row `row` of `palette`, for a sprite or a tile layer to draw instead
    /// of `index`. `index` is an index texture as the cook writes
    /// `<id>#index` (og-cook README §Art conformance): alpha the palette
    /// index plus one, 0 a clear texel. `palette` is N rows of colours,
    /// row r's texel i the colour index i takes in that row (team colours,
    /// a hit flash in palette, a day and a night ramp), its alpha too (0
    /// hides that index). A row past the last is the last; an index past the
    /// row's width keeps the index texture's own colour (the one the cook
    /// mapped it to), opaque. The texture is made once per (index, palette,
    /// row), sampled as `index` is ([`RenderScene::set_texture_filter`]),
    /// and made again when either is replaced. Both must be RGBA8 (a
    /// [`TextureData`] or an RGBA8 upload's first level): `InvalidData`
    /// otherwise, `InvalidHandle` for a removed or foreign one.
    pub fn paletted_texture(
        &mut self,
        index: TextureHandle,
        palette: TextureHandle,
        row: u32,
    ) -> Result<TextureHandle, crate::RenderError> {
        for t in [index, palette] {
            if !self.texture_slots.live(t.index(), t.generation()) {
                return Err(crate::RenderError::InvalidHandle(format!(
                    "palette swap: texture {}",
                    t.index()
                )));
            }
        }
        let revs = (
            self.texture_revisions[index.index()],
            self.texture_revisions[palette.index()],
        );
        let key = (index, palette, row);
        if let Some(&(a, b, made)) = self.paletted.get(&key) {
            if (a, b) == revs && self.texture_slots.live(made.index(), made.generation()) {
                self.set_texture_filter(made, self.texture_filter(index));
                return Ok(made);
            }
            self.remove_texture(made);
        }
        let rgba = |t: TextureHandle| -> Result<(u32, u32, &[u8], ColorSpace), crate::RenderError> {
            match &self.textures[t.index()] {
                TextureSource::Rgba(d) => Ok((d.width, d.height, &d.rgba[..], d.color_space)),
                TextureSource::Encoded(u) if u.desc.format == crate::TextureFormat::Rgba8 => Ok((
                    u.desc.width,
                    u.desc.height,
                    &u.levels[0][..],
                    u.desc.color_space,
                )),
                _ => Err(crate::RenderError::InvalidData(format!(
                    "palette swap: texture {} is not RGBA8 (cook it as kind pixel)",
                    t.index()
                ))),
            }
        };
        let (w, h, idx, _) = rgba(index)?;
        let (pw, ph, pal, space) = rgba(palette)?;
        let data = TextureData {
            width: w,
            height: h,
            rgba: palette_swap(idx, (pw, ph, pal), row),
            color_space: space,
            mips: Mips::None,
        };
        let filter = self.texture_filter(index);
        let made = self.add_texture(data);
        self.set_texture_filter(made, filter);
        self.paletted.insert(key, (revs.0, revs.1, made));
        Ok(made)
    }

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

    /// Adds terrain data (heights and a control map, an asset); draw it
    /// with a [`crate::Terrain`] in [`RenderScene::terrains`]. Checked at
    /// its first frame ([`crate::RenderError::InvalidData`]).
    pub fn add_terrain(&mut self, data: crate::TerrainData) -> TerrainHandle {
        self.terrain_data
            .push(crate::terrain::TerrainSource::data(data));
        self.terrain_revisions.push(0);
        self.terrain_edits.push(Vec::new());
        TerrainHandle(self.terrain_data.len() as u32 - 1)
    }

    /// Adds a streamed terrain: its tiles arrive one by one with
    /// [`RenderScene::set_terrain_tile`] (the top level's before it draws),
    /// `range` the lowest and highest height. A tile that has not arrived
    /// shows its parent's.
    pub fn add_terrain_stream(
        &mut self,
        layout: crate::TerrainLayout,
        range: (f32, f32),
    ) -> TerrainHandle {
        self.terrain_data
            .push(crate::terrain::TerrainSource::Stream {
                layout,
                range,
                tiles: Default::default(),
            });
        self.terrain_revisions.push(0);
        self.terrain_edits.push(Vec::new());
        TerrainHandle(self.terrain_data.len() as u32 - 1)
    }

    /// Makes a terrain streamed again from the start (no tile resident),
    /// as after [`RenderScene::add_terrain_stream`]. `false` for a handle
    /// of another scene.
    pub fn set_terrain_stream(
        &mut self,
        handle: TerrainHandle,
        layout: crate::TerrainLayout,
        range: (f32, f32),
    ) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(d) => {
                *d = crate::terrain::TerrainSource::Stream {
                    layout,
                    range,
                    tiles: Default::default(),
                };
                self.terrain_revisions[handle.index()] += 1;
                self.terrain_edits[handle.index()].clear();
                true
            }
            None => false,
        }
    }

    /// Hands a streamed terrain one of its tiles (`None`: drops it). The
    /// renderer draws what it holds of the tiles it wants. `false` for a
    /// handle of another scene, a terrain in memory, a tile index beyond
    /// its layout or a tile whose heights, normals or control texels are not
    /// the lengths its layout uploads (the tile is not taken).
    pub fn set_terrain_tile(
        &mut self,
        handle: TerrainHandle,
        index: u32,
        tile: Option<std::sync::Arc<crate::TerrainTile>>,
    ) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(crate::terrain::TerrainSource::Stream { layout, tiles, .. })
                if index < layout.tile_count() =>
            {
                match tile {
                    Some(t) if !t.fits(layout) => return false,
                    Some(t) => {
                        tiles.insert(index, t);
                    }
                    None => {
                        tiles.remove(&index);
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// The tiles a streamed terrain holds now (`None` for a terrain in
    /// memory).
    pub fn terrain_tiles(
        &self,
        handle: TerrainHandle,
    ) -> Option<&og_core::IdMap<u32, std::sync::Arc<crate::TerrainTile>>> {
        match self.terrain_data.get(handle.index())? {
            crate::terrain::TerrainSource::Stream { tiles, .. } => Some(tiles),
            crate::terrain::TerrainSource::Data(..) => None,
        }
    }

    /// The layout a terrain is drawn with (a field in memory's is chosen
    /// by the renderer: tiles of [`crate::TERRAIN_TILE`] quads, or larger).
    pub fn terrain_layout(&self, handle: TerrainHandle) -> Option<crate::TerrainLayout> {
        self.terrain_data.get(handle.index())?.layout().ok()
    }

    /// The clipmap [`RenderScene::terrains`]`[index]` is drawn with from
    /// `view` (its levels' extents and morph bands, placed by its height
    /// errors as seen from there: [`crate::Terrain::clipmap`]). The
    /// renderer, the id pass and diagnostics all lay a terrain out with it.
    pub fn terrain_clipmap(
        &self,
        index: usize,
        view: &crate::TerrainView,
    ) -> Option<crate::TerrainClipmap> {
        let t = self.terrains.get(index)?;
        let source = self.terrain_data.get(t.terrain.index())?;
        let layout = source.layout().ok()?;
        let model = t.transform;
        let along = crate::terrain::horizontal_scale(model);
        let up = model.transform_vector3(glam::Vec3::Y).length();
        let eye = model.inverse().transform_point3(view.camera);
        let reach = crate::terrain_error::error_reach(
            source.error_blocks(&layout).into_iter(),
            eye,
            source.range(),
            (along, up),
            view.pixel,
            t.lod.error_pixels(),
            view.far / along,
        );
        Some(t.clipmap(layout, view, &reach))
    }

    /// Replaces a terrain's data (a runtime edit of its heights or its
    /// control map); the renderer uploads it again before the next frame
    /// that draws it. `false` for a handle of another scene.
    pub fn set_terrain(&mut self, handle: TerrainHandle, data: crate::TerrainData) -> bool {
        match self.terrain_data.get_mut(handle.index()) {
            Some(d) => {
                *d = crate::terrain::TerrainSource::data(data);
                self.terrain_revisions[handle.index()] += 1;
                self.terrain_edits[handle.index()].clear();
                true
            }
            None => false,
        }
    }

    /// A terrain's field in memory (`None` for a streamed one).
    pub fn terrain(&self, handle: TerrainHandle) -> Option<&crate::TerrainData> {
        match self.terrain_data.get(handle.index())? {
            crate::terrain::TerrainSource::Data(d, ..) => Some(d),
            crate::terrain::TerrainSource::Stream { .. } => None,
        }
    }

    /// Replaces a terrain's data after an edit that changed it only inside
    /// `lo`..`hi` (the terrain's space, metres from its centre): the
    /// renderer builds again only the tiles that reach into it, where
    /// [`RenderScene::set_terrain`] builds every one. Data of another
    /// shape (size, samples, holes or none, control map size or pages) and
    /// a terrain streamed until now are replaced whole. `false` for a
    /// handle of another scene.
    pub fn edit_terrain(
        &mut self,
        handle: TerrainHandle,
        data: crate::TerrainData,
        lo: glam::Vec2,
        hi: glam::Vec2,
    ) -> bool {
        let i = handle.index();
        let shape = |d: &crate::TerrainData| {
            (
                d.size,
                d.columns,
                d.rows,
                d.holes.is_some(),
                d.control.as_ref().map(|c| (c.width, c.height, c.pages())),
            )
        };
        let in_place = match self.terrain_data.get_mut(i) {
            Some(s) => {
                // A field streamed until now, or of another shape, is new.
                let same = matches!(&*s, crate::terrain::TerrainSource::Data(d, ..) if shape(d) == shape(&data));
                *s = if same {
                    s.edited(data, lo, hi)
                } else {
                    crate::terrain::TerrainSource::data(data)
                };
                same
            }
            None => return false,
        };
        self.terrain_revisions[i] += 1;
        let log = &mut self.terrain_edits[i];
        if in_place {
            log.push((self.terrain_revisions[i], lo, hi));
            if log.len() > TERRAIN_EDIT_LOG {
                log.remove(0);
            }
        } else {
            log.clear();
        }
        true
    }

    pub(crate) fn terrain_sources(&self) -> (&[crate::terrain::TerrainSource], &[u64]) {
        (&self.terrain_data, &self.terrain_revisions)
    }

    /// The regions terrain `index` was edited in place
    /// ([`RenderScene::edit_terrain`]) after `revision`, oldest first;
    /// `None` when it was replaced whole since, or the log no longer
    /// reaches back that far.
    pub(crate) fn terrain_edits_since(
        &self,
        index: usize,
        revision: u64,
    ) -> Option<Vec<(glam::Vec2, glam::Vec2)>> {
        let now = *self.terrain_revisions.get(index)?;
        let log = self.terrain_edits.get(index)?;
        let after: Vec<_> = log.iter().filter(|e| e.0 > revision).collect();
        (after.len() as u64 == now.checked_sub(revision)?
            && after.first().is_none_or(|e| e.0 == revision + 1))
        .then(|| after.iter().map(|e| (e.1, e.2)).collect())
    }

    /// How far from the camera terrain `index` (of [`RenderScene::terrains`])
    /// needs its finest tiles for the copies scattered over it.
    pub fn scatter_detail(&self, index: usize) -> f32 {
        self.scatters
            .iter()
            .filter(|s| s.terrain == index)
            .map(|s| s.scatter.distance)
            .fold(0.0, f32::max)
    }

    /// Adds a particle emitter description (an asset).
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) -> EmitterHandle {
        self.emitters.push(emitter);
        EmitterHandle(self.emitters.len() as u32 - 1)
    }

    /// Replaces an emitter (running systems keep their particles while the
    /// capacity is unchanged). `false` for a handle of another scene.
    pub fn set_emitter(&mut self, handle: EmitterHandle, emitter: ParticleEmitter) -> bool {
        match self.emitters.get_mut(handle.index()) {
            Some(e) => {
                *e = emitter;
                true
            }
            None => false,
        }
    }

    pub fn emitter(&self, handle: EmitterHandle) -> Option<&ParticleEmitter> {
        self.emitters.get(handle.index())
    }

    /// Adds an effect material (`effect.rs`): the look particles, trails,
    /// ribbons, beams and effect meshes name.
    pub fn add_effect_material(
        &mut self,
        material: crate::EffectMaterial,
    ) -> crate::EffectMaterialHandle {
        self.effect_materials.push(material);
        crate::EffectMaterialHandle(self.effect_materials.len() as u32 - 1)
    }

    /// Replaces an effect material. `false` for a handle of another scene.
    pub fn set_effect_material(
        &mut self,
        handle: crate::EffectMaterialHandle,
        material: crate::EffectMaterial,
    ) -> bool {
        match self.effect_materials.get_mut(handle.index()) {
            Some(m) => {
                *m = material;
                true
            }
            None => false,
        }
    }

    pub fn effect_material(
        &self,
        handle: crate::EffectMaterialHandle,
    ) -> Option<&crate::EffectMaterial> {
        self.effect_materials.get(handle.index())
    }

    pub fn effect_materials(&self) -> &[crate::EffectMaterial] {
        &self.effect_materials
    }

    /// Adds a 32³ colour grading table ([`ColorLut`]). A slot freed by
    /// [`RenderScene::remove_lut`] is reused under a new generation.
    pub fn add_lut(&mut self, lut: ColorLut) -> LutHandle {
        let (i, generation, reused) = self.lut_slots.take(self.luts.len(), true);
        if reused {
            self.luts[i] = lut;
            self.lut_revisions[i] += 1;
        } else {
            self.luts.push(lut);
            self.lut_revisions.push(0);
        }
        LutHandle::from_parts(i, generation)
    }

    /// Removes a colour LUT: the renderer frees its 3D texture before the
    /// next frame and a later add reuses the slot under a new generation.
    /// A frame whose colour grading still names it is refused
    /// ([`crate::RenderError::InvalidHandle`]). False for a handle that is
    /// not live.
    pub fn remove_lut(&mut self, handle: LutHandle) -> bool {
        let i = handle.index();
        if !self.lut_slots.live(i, handle.generation()) {
            return false;
        }
        self.luts[i] = ColorLut {
            size: 0,
            rgba: Vec::new(),
        };
        self.lut_revisions[i] += 1;
        self.lut_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a LUT of this scene that was not removed.
    pub fn contains_lut(&self, handle: LutHandle) -> bool {
        self.lut_slots.live(handle.index(), handle.generation())
    }

    /// A live LUT.
    pub fn lut(&self, handle: LutHandle) -> Option<&ColorLut> {
        self.contains_lut(handle)
            .then(|| &self.luts[handle.index()])
    }

    pub(crate) fn lut_revisions(&self) -> &[u64] {
        &self.lut_revisions
    }

    /// Whether slot `index` holds a live LUT.
    /// Whether texture slot `index` holds a live texture (not removed): the
    /// renderer frees a removed slot's GPU texture instead of uploading.
    pub(crate) fn texture_live(&self, index: usize) -> bool {
        let g = self.texture_slots.generation(index);
        self.texture_slots.live(index, g)
    }

    pub(crate) fn lut_live(&self, index: usize) -> bool {
        let g = self.lut_slots.generation(index);
        self.lut_slots.live(index, g)
    }

    /// Adds a prefiltered environment map (an asset; og-cook's `type
    /// environment`); [`RenderScene::environment_map`] picks the one a
    /// frame is lit by. A slot freed by [`RenderScene::remove_environment_map`]
    /// is reused under a new generation.
    pub fn add_environment_map(&mut self, map: crate::EnvironmentMapData) -> EnvironmentMapHandle {
        let (i, generation, reused) = self
            .environment_map_slots
            .take(self.environment_maps.len(), true);
        if reused {
            self.environment_maps[i] = map;
            self.environment_map_revisions[i] += 1;
            self.environment_equirects[i] = None;
        } else {
            self.environment_maps.push(map);
            self.environment_map_revisions.push(0);
            self.environment_equirects.push(None);
        }
        EnvironmentMapHandle::from_parts(i, generation)
    }

    /// Adds an environment map from an equirectangular image the renderer
    /// prefilters itself, on the GPU when it uploads it (run-time loads of
    /// uncooked `.hdr` and `.exr`), with its `irradiance` already gathered
    /// ([`crate::EquirectIrradiance`]). [`RenderScene::environment_map_data`]
    /// answers its size and irradiance with no texels. Refused when the
    /// image or the size is not valid.
    pub fn add_environment_equirect(
        &mut self,
        image: crate::EnvironmentEquirect,
        irradiance: [[f32; 3]; 9],
    ) -> Result<EnvironmentMapHandle, String> {
        image.validate()?;
        if irradiance.iter().flatten().any(|v| !v.is_finite()) {
            return Err("an environment map's irradiance is not finite".into());
        }
        let size = image.size;
        let handle = self.add_environment_map(crate::EnvironmentMapData {
            size,
            texels: Vec::new(),
            irradiance,
        });
        self.environment_equirects[handle.index()] = Some(std::sync::Arc::new(image));
        Ok(handle)
    }

    /// Slot `index`'s equirectangular image, when the renderer bakes it.
    pub(crate) fn environment_equirect(&self, index: usize) -> Option<&crate::EnvironmentEquirect> {
        self.environment_equirects.get(index)?.as_deref()
    }

    /// Removes an environment map: the renderer frees its cube before the
    /// next frame and a later add reuses the slot under a new generation.
    /// A frame whose [`RenderScene::environment_map`] still names it is
    /// refused ([`crate::RenderError::InvalidHandle`]): light the scene
    /// with another map or the sky first. False for a handle that is not
    /// live.
    pub fn remove_environment_map(&mut self, handle: EnvironmentMapHandle) -> bool {
        let i = handle.index();
        if !self.environment_map_slots.live(i, handle.generation()) {
            return false;
        }
        self.environment_maps[i] = crate::EnvironmentMapData {
            size: 0,
            texels: Vec::new(),
            irradiance: [[0.0; 3]; 9],
        };
        self.environment_map_revisions[i] += 1;
        self.environment_equirects[i] = None;
        self.environment_map_slots.remove(i, true);
        true
    }

    /// Whether `handle` names an environment map of this scene that was
    /// not removed.
    pub fn contains_environment_map(&self, handle: EnvironmentMapHandle) -> bool {
        self.environment_map_slots
            .live(handle.index(), handle.generation())
    }

    /// A live environment map's data.
    pub fn environment_map_data(
        &self,
        handle: EnvironmentMapHandle,
    ) -> Option<&crate::EnvironmentMapData> {
        self.contains_environment_map(handle)
            .then(|| &self.environment_maps[handle.index()])
    }

    pub(crate) fn environment_maps(&self) -> &[crate::EnvironmentMapData] {
        &self.environment_maps
    }

    pub(crate) fn environment_map_revisions(&self) -> &[u64] {
        &self.environment_map_revisions
    }

    /// Whether slot `index` holds a live environment map, and its generation.
    pub(crate) fn environment_map_slot(&self, index: usize) -> (bool, u8) {
        let g = self.environment_map_slots.generation(index);
        (self.environment_map_slots.live(index, g), g)
    }

    /// Adds a texture array (an asset; og-cook's `type texture-array`):
    /// layers of one size, encoding and mip count, uploaded as one
    /// `D2Array` texture. Refused when the layers disagree. A slot freed
    /// by [`RenderScene::remove_texture_array`] is reused under a new
    /// generation.
    pub fn add_texture_array(
        &mut self,
        data: crate::TextureArrayData,
    ) -> Result<crate::TextureArrayHandle, String> {
        data.info()?;
        let (i, generation, reused) = self
            .texture_array_slots
            .take(self.texture_arrays.len(), true);
        if reused {
            self.texture_arrays[i] = data;
            self.texture_array_revisions[i] += 1;
            if let Some(f) = self.texture_array_filters.get_mut(i) {
                *f = TextureFilter::Linear;
            }
        } else {
            self.texture_arrays.push(data);
            self.texture_array_revisions.push(0);
        }
        Ok(crate::TextureArrayHandle::from_parts(i, generation))
    }

    /// Replaces a live texture array's layers (the same array in another
    /// encoding, after the device changed): the renderer uploads it again
    /// before the next frame and every sprite, emitter and material that
    /// samples it draws the new layers. Refused when the layers disagree;
    /// `Ok(false)` for a handle that is not live.
    pub fn set_texture_array(
        &mut self,
        handle: crate::TextureArrayHandle,
        data: crate::TextureArrayData,
    ) -> Result<bool, String> {
        data.info()?;
        let i = handle.index();
        if !self.texture_array_slots.live(i, handle.generation()) {
            return Ok(false);
        }
        self.texture_arrays[i] = data;
        self.texture_array_revisions[i] += 1;
        Ok(true)
    }

    /// Removes a texture array: the renderer frees it before the next frame
    /// and a later add reuses the slot under a new generation. A sprite
    /// still drawn from it ([`RenderScene::sprite_arrays`]) is refused at
    /// the next frame ([`crate::RenderError::InvalidHandle`]); a material
    /// sampling it forgets it (its array maps read as none), as a removed
    /// texture's. False for a handle that is not live.
    pub fn remove_texture_array(&mut self, handle: crate::TextureArrayHandle) -> bool {
        let i = handle.index();
        if !self.texture_array_slots.live(i, handle.generation()) {
            return false;
        }
        self.texture_arrays[i] = crate::TextureArrayData { layers: Vec::new() };
        self.texture_array_revisions[i] += 1;
        if let Some(f) = self.texture_array_filters.get_mut(i) {
            *f = TextureFilter::Linear;
        }
        self.texture_array_slots.remove(i, true);
        // Materials sampling it draw without it from now on.
        let slot = handle.slot();
        for (m, revision) in self
            .materials
            .iter_mut()
            .zip(self.material_revisions.iter_mut())
        {
            if m.forget_texture_array(slot) {
                *revision += 1;
            }
        }
        true
    }

    /// Whether `handle` names a texture array of this scene that was not
    /// removed.
    pub fn contains_texture_array(&self, handle: crate::TextureArrayHandle) -> bool {
        self.texture_array_slots
            .live(handle.index(), handle.generation())
    }

    /// A live texture array's layers.
    pub fn texture_array(
        &self,
        handle: crate::TextureArrayHandle,
    ) -> Option<&crate::TextureArrayData> {
        self.contains_texture_array(handle)
            .then(|| &self.texture_arrays[handle.index()])
    }

    /// Samples a texture array nearest (pixel art) or linear (the default)
    /// wherever sprites draw it. False for a handle that is not live.
    pub fn set_texture_array_filter(
        &mut self,
        handle: crate::TextureArrayHandle,
        filter: TextureFilter,
    ) -> bool {
        let i = handle.index();
        if !self.texture_array_slots.live(i, handle.generation()) {
            return false;
        }
        if self.texture_array_filters.len() <= i {
            self.texture_array_filters
                .resize(i + 1, TextureFilter::Linear);
        }
        self.texture_array_filters[i] = filter;
        true
    }

    /// How sprites sample a texture array.
    pub fn texture_array_filter(&self, handle: crate::TextureArrayHandle) -> TextureFilter {
        self.texture_array_filters
            .get(handle.index())
            .copied()
            .unwrap_or_default()
    }

    /// Every slot's layers, by handle index (a removed slot holds none).
    pub fn texture_arrays(&self) -> &[crate::TextureArrayData] {
        &self.texture_arrays
    }

    pub(crate) fn texture_array_revisions(&self) -> &[u64] {
        &self.texture_array_revisions
    }

    /// Whether slot `index` holds a live texture array, and its generation.
    pub(crate) fn texture_array_slot(&self, index: usize) -> (bool, u8) {
        let g = self.texture_array_slots.generation(index);
        (self.texture_array_slots.live(index, g), g)
    }

    /// Adds a trail style (an asset).
    pub fn add_trail_style(&mut self, style: TrailStyle) -> TrailStyleHandle {
        self.trail_styles.push(style);
        TrailStyleHandle(self.trail_styles.len() as u32 - 1)
    }

    pub(crate) fn emitters(&self) -> &[ParticleEmitter] {
        &self.emitters
    }

    pub(crate) fn luts(&self) -> &[ColorLut] {
        &self.luts
    }

    /// Adds a colour palette (an asset; og-cook's `.hex` and `.gpl`) for
    /// [`crate::Stylise`]'s quantisation; at most
    /// [`crate::MAX_PALETTE_COLORS`] colours are kept.
    pub fn add_palette(&mut self, mut palette: crate::Palette) -> crate::PaletteHandle {
        palette.colors.truncate(crate::MAX_PALETTE_COLORS);
        let (i, generation, reused) = self.palette_slots.take(self.palettes.len(), true);
        if reused {
            self.palettes[i] = palette;
            self.palette_revisions[i] += 1;
        } else {
            self.palettes.push(palette);
            self.palette_revisions.push(0);
        }
        crate::PaletteHandle::from_parts(i, generation)
    }

    /// Removes a palette: the renderer frees its texture before the next
    /// frame and a later add reuses the slot under a new generation. A
    /// frame whose [`RenderScene::stylise`] still quantises to it is
    /// refused ([`crate::RenderError::InvalidHandle`]). False for a handle
    /// that is not live.
    pub fn remove_palette(&mut self, handle: crate::PaletteHandle) -> bool {
        let i = handle.index();
        if !self.palette_slots.live(i, handle.generation()) {
            return false;
        }
        self.palettes[i] = crate::Palette::default();
        self.palette_revisions[i] += 1;
        self.palette_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a palette of this scene that was not removed.
    pub fn contains_palette(&self, handle: crate::PaletteHandle) -> bool {
        self.palette_slots.live(handle.index(), handle.generation())
    }

    /// A live palette.
    pub fn palette(&self, handle: crate::PaletteHandle) -> Option<&crate::Palette> {
        self.contains_palette(handle)
            .then(|| &self.palettes[handle.index()])
    }

    /// Adds colours of art the cook already conformed to a look's palette
    /// limits (a texture's own `<id>#palette`): the stylise quantiser
    /// leaves a pixel of exactly one of them as drawn, neither posterised
    /// nor dithered, and steps and dithers every other (lit, fogged or
    /// unconformed materials), so the look's frame quantise stays the
    /// fallback for what the cook did not conform.
    pub fn add_conformed_colors(&mut self, colors: &[[u8; 3]]) {
        let before = self.conformed.len();
        self.conformed.extend_from_slice(colors);
        self.conformed.sort_unstable();
        self.conformed.dedup();
        if self.conformed.len() != before {
            self.conformed_revision += 1;
        }
    }

    /// The conformed colours ([`Scene::add_conformed_colors`]) and the
    /// set's revision.
    pub fn conformed_colors(&self) -> (&[[u8; 3]], u64) {
        (&self.conformed, self.conformed_revision)
    }

    /// Every slot's palette, by handle index (a removed slot holds no
    /// colours).
    pub fn palettes(&self) -> &[crate::Palette] {
        &self.palettes
    }

    pub(crate) fn palette_revisions(&self) -> &[u64] {
        &self.palette_revisions
    }

    /// Whether slot `index` holds a live palette.
    pub(crate) fn palette_live(&self, index: usize) -> bool {
        let g = self.palette_slots.generation(index);
        self.palette_slots.live(index, g)
    }

    pub(crate) fn trail_styles(&self) -> &[TrailStyle] {
        &self.trail_styles
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
        self.meshlets && self.mesh_clusters(mesh).is_some() && self.contains_material(material) && {
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

    /// Adds an RGBA8 texture (mips generated on the GPU when asked).
    pub fn add_texture(&mut self, texture: TextureData) -> TextureHandle {
        self.add_texture_source(TextureSource::Rgba(texture))
    }

    /// Replaces a texture (a streamed texture over its placeholder, or back):
    /// the renderer uploads it again before the next frame, and every
    /// material, sprite, tile, trail, font, decal and terrain layer that
    /// samples it draws the new one. False for a bad handle.
    pub fn set_texture(&mut self, handle: TextureHandle, texture: TextureData) -> bool {
        self.replace_texture(handle, TextureSource::Rgba(texture))
    }

    /// [`RenderScene::set_texture`] with mip levels already encoded.
    pub fn set_texture_upload(&mut self, handle: TextureHandle, upload: TextureUpload) -> bool {
        self.replace_texture(handle, TextureSource::Encoded(upload))
    }

    fn replace_texture(&mut self, handle: TextureHandle, source: TextureSource) -> bool {
        let i = handle.index();
        if !self.texture_slots.live(i, handle.generation()) {
            return false;
        }
        self.textures[i] = source;
        self.texture_revisions[i] += 1;
        true
    }

    pub(crate) fn texture_revisions(&self) -> &[u64] {
        &self.texture_revisions
    }

    /// Adds a texture whose mip levels are already encoded, in one of
    /// [`crate::RenderInfo::texture_formats`]. Checked (and refused with
    /// [`crate::RenderError::InvalidData`] or `Unsupported`) when uploaded.
    pub fn add_texture_upload(&mut self, upload: TextureUpload) -> TextureHandle {
        self.add_texture_source(TextureSource::Encoded(upload))
    }

    /// Adds a texture of either kind ([`TextureSource`]).
    pub fn add_texture_source(&mut self, source: TextureSource) -> TextureHandle {
        let (i, generation, reused) = self.texture_slots.take(self.textures.len(), true);
        if reused {
            self.textures[i] = source;
            self.texture_revisions[i] += 1;
            if let Some(f) = self.texture_filters.get_mut(i) {
                *f = TextureFilter::Linear;
            }
            if let Some(n) = self.texture_normals.get_mut(i) {
                *n = None;
            }
        } else {
            self.textures.push(source);
            self.texture_revisions.push(0);
        }
        TextureHandle::from_parts(i, generation)
    }

    /// Removes a texture: the renderer frees it before the next frame (a
    /// stale reference samples its one shared fallback, nothing is
    /// uploaded for the slot) and a later add reuses the slot under a new
    /// generation. Every material
    /// slot that sampled it is cleared (those materials draw without it from
    /// the next frame), and so is the decal atlas; a sprite or a font
    /// atlas still naming it is refused at the next frame
    /// ([`crate::RenderError::InvalidHandle`]): remove the font first.
    /// False for a handle that is not live.
    pub fn remove_texture(&mut self, handle: TextureHandle) -> bool {
        let i = handle.index();
        if !self.texture_slots.live(i, handle.generation()) {
            return false;
        }
        self.stream_frames.remove(&i);
        #[cfg(target_arch = "wasm32")]
        if let Some(f) = self.stream_external.get_mut().remove(&i) {
            f.close();
        }
        self.textures[i] = TextureSource::Rgba(TextureData {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
            color_space: ColorSpace::Srgb,
            mips: Mips::None,
        });
        self.texture_revisions[i] += 1;
        if let Some(f) = self.texture_filters.get_mut(i) {
            *f = TextureFilter::Linear;
        }
        self.texture_sizes.remove(&handle);
        if let Some(n) = self.texture_normals.get_mut(i) {
            *n = None;
        }
        let slot = handle.slot();
        for (m, revision) in self
            .materials
            .iter_mut()
            .zip(self.material_revisions.iter_mut())
        {
            if m.forget_texture(slot) {
                *revision += 1;
            }
        }
        for m in self.gltf_materials.iter_mut().flatten() {
            for t in [
                &mut m.base_color_texture,
                &mut m.metallic_roughness_texture,
                &mut m.normal_texture,
                &mut m.emissive_texture,
            ] {
                if *t == Some(handle) {
                    *t = None;
                }
            }
        }
        if self.decal_atlas == Some(handle) {
            self.decal_atlas = None;
        }
        self.texture_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a texture of this scene that was not removed.
    pub fn contains_texture(&self, handle: TextureHandle) -> bool {
        self.texture_slots.live(handle.index(), handle.generation())
    }

    /// Adds a glTF-shaped material (drawn as its [`StandardMaterial`] form).
    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        let standard = StandardMaterial::from(&material);
        self.push_material(standard, Some(material))
    }

    /// Adds a three.js-style standard material.
    pub fn add_standard_material(&mut self, material: StandardMaterial) -> MaterialHandle {
        self.push_material(material, None)
    }

    fn push_material(
        &mut self,
        material: StandardMaterial,
        gltf: Option<Material>,
    ) -> MaterialHandle {
        let (i, generation, reused) = self.material_slots.take(self.materials.len(), true);
        if reused {
            self.materials[i] = material;
            self.gltf_materials[i] = gltf;
            self.material_revisions[i] += 1;
        } else {
            self.materials.push(material);
            self.gltf_materials.push(gltf);
            self.material_revisions.push(0);
        }
        MaterialHandle::from_parts(i, generation)
    }

    /// Removes a material: the renderer frees its bind groups before the
    /// next frame and a later add reuses the slot under a new generation;
    /// an instance still drawing with this handle is refused
    /// ([`crate::RenderError::InvalidHandle`]). Its material shader and
    /// parameters go with it. False for a handle that is not live.
    pub fn remove_material(&mut self, handle: MaterialHandle) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = StandardMaterial::default();
        self.gltf_materials[i] = None;
        if let Some(s) = self.material_shaders.get_mut(i) {
            *s = None;
        }
        self.material_revisions[i] += 1;
        self.material_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a material of this scene that was not removed.
    pub fn contains_material(&self, handle: MaterialHandle) -> bool {
        self.material_slots
            .live(handle.index(), handle.generation())
    }

    /// Replaces a material with a glTF-shaped one; the renderer re-uploads it
    /// on the next frame. Returns `false` (and changes nothing) for a handle
    /// of another scene.
    pub fn set_material(&mut self, handle: MaterialHandle, material: Material) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = StandardMaterial::from(&material);
        self.gltf_materials[i] = Some(material);
        self.material_revisions[i] += 1;
        true
    }

    /// Replaces a material with a standard one (a material edit: the next
    /// frame draws it). Returns `false` for a handle of another scene.
    pub fn set_standard_material(
        &mut self,
        handle: MaterialHandle,
        material: StandardMaterial,
    ) -> bool {
        let i = handle.index();
        if !self.material_slots.live(i, handle.generation()) {
            return false;
        }
        self.materials[i] = material;
        self.gltf_materials[i] = None;
        self.material_revisions[i] += 1;
        true
    }

    /// Draws `copy` in the batches of `of` while neither changes: two
    /// materials of one value (models that each hold their own copy of one
    /// glTF material) draw in one instanced batch. Copy-on-write: any edit
    /// of either (a `set_*`, a shader, a removed texture or the removal of
    /// either) ends it, and each draws as its own from that frame on, so an
    /// edit of one never shows on the other. False, and nothing changes,
    /// unless both are live and draw the same now (equal values, no
    /// material shader).
    pub fn alias_material(&mut self, copy: MaterialHandle, of: MaterialHandle) -> bool {
        let live = |h: MaterialHandle| self.material_slots.live(h.index(), h.generation());
        if !live(copy) || !live(of) {
            return false;
        }
        let (c, o) = (copy.index(), self.drawn_material(of.index()));
        let unshaded = |i: usize| self.material_shaders.get(i).is_none_or(Option::is_none);
        if c == o || self.materials[c] != self.materials[o] || !unshaded(c) || !unshaded(o) {
            return false;
        }
        if self.material_aliases.len() <= c {
            self.material_aliases.resize(c + 1, None);
        }
        let revisions = &self.material_revisions;
        self.material_aliases[c] = Some((o as u32, revisions[c], revisions[o]));
        true
    }

    /// The material index material `i`'s instances batch under: the one
    /// [`RenderScene::alias_material`] gave it while that holds, else its own.
    pub(crate) fn drawn_material(&self, i: usize) -> usize {
        match self.material_aliases.get(i).copied().flatten() {
            Some((o, rc, ro))
                if self.material_revisions[i] == rc
                    && self.material_revisions[o as usize] == ro =>
            {
                o as usize
            }
            _ => i,
        }
    }

    /// Adds one instance (the same as pushing onto [`RenderScene::instances`]).
    pub fn add_instance(&mut self, mesh: MeshHandle, material: MaterialHandle, transform: Mat4) {
        self.instances.push(Instance {
            mesh,
            material,
            transform,
            material_layer: None,
            fx: InstanceFx::NONE,
            cast_shadows: true,
            receive_shadows: true,
            sway: 0.0,
        });
    }

    /// Adds an MSDF font whose atlas is a texture of this scene.
    pub fn add_font(&mut self, font: MsdfFont) -> FontHandle {
        let (i, generation, reused) = self.font_slots.take(self.fonts.len(), true);
        if reused {
            self.fonts[i] = font;
            self.font_revisions[i] += 1;
        } else {
            self.fonts.push(font);
            self.font_revisions.push(0);
        }
        FontHandle::from_parts(i, generation)
    }

    /// Removes a font (its glyph table; its atlas is a texture of its own,
    /// removed with [`RenderScene::remove_texture`] after): a later add
    /// reuses the slot under a new generation, and text still set in this
    /// font is refused at the next frame ([`crate::RenderError::InvalidHandle`]).
    /// False for a handle that is not live.
    pub fn remove_font(&mut self, handle: FontHandle) -> bool {
        let i = handle.index();
        if !self.font_slots.live(i, handle.generation()) {
            return false;
        }
        let f = &mut self.fonts[i];
        f.glyphs = Vec::new();
        f.kerning = Vec::new();
        self.font_revisions[i] += 1;
        self.font_slots.remove(i, true);
        true
    }

    /// Whether `handle` names a font of this scene that was not removed.
    pub fn contains_font(&self, handle: FontHandle) -> bool {
        self.font_slots.live(handle.index(), handle.generation())
    }

    /// Replaces a font's glyph table, kerning or atlas (glyphs rendered at
    /// run time joining a page): the renderer lays text out with the new
    /// table from the next frame. False for a bad handle.
    pub fn set_font(&mut self, handle: FontHandle, font: MsdfFont) -> bool {
        let i = handle.index();
        if !self.font_slots.live(i, handle.generation()) {
            return false;
        }
        self.fonts[i] = font;
        self.font_revisions[i] += 1;
        true
    }

    pub(crate) fn font_revisions(&self) -> &[u64] {
        &self.font_revisions
    }

    /// Adds one sprite (the same as pushing onto [`RenderScene::sprites`]).
    pub fn add_sprite(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
    }

    pub fn font(&self, handle: FontHandle) -> Option<&MsdfFont> {
        if !self.contains_font(handle) {
            return None;
        }
        self.fonts.get(handle.index())
    }

    /// Every per-frame reference to a removed mesh, material, texture,
    /// font, environment map, texture array, palette or LUT: instances,
    /// skinned instances, sprites (and the arrays they draw from), texts,
    /// the overlay's, the environment map, the colour grading's LUT and the
    /// stylisation's palette (nothing to look at until something was
    /// removed).
    pub(crate) fn stale_handle(&self) -> Option<String> {
        let (m, t, mat, f) = (
            &self.mesh_slots,
            &self.texture_slots,
            &self.material_slots,
            &self.font_slots,
        );
        if !(m.any_removed
            || t.any_removed
            || mat.any_removed
            || f.any_removed
            || self.environment_map_slots.any_removed
            || self.texture_array_slots.any_removed
            || self.palette_slots.any_removed
            || self.lut_slots.any_removed)
        {
            return None;
        }
        if let Some(e) = self.environment_map
            && !self.contains_environment_map(e.map)
        {
            return Some(format!("the environment map {} was removed", e.map.index()));
        }
        if let Some(g) = self.post.color_grading
            && !self.contains_lut(g.lut)
        {
            return Some(format!(
                "the colour grading's lut {} was removed",
                g.lut.index()
            ));
        }
        if let Some(p) = self.stylise.quantize.and_then(|q| q.palette)
            && !self.contains_palette(p)
        {
            return Some(format!(
                "the stylisation's palette {} was removed",
                p.index()
            ));
        }
        for s in &self.sprite_arrays {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        for s in self.overlay.iter().flat_map(|o| &o.sprite_arrays) {
            if !self.contains_texture_array(s.array) {
                return Some(format!(
                    "overlay sprite {}: texture array {} was removed",
                    s.sprite,
                    s.array.index()
                ));
            }
        }
        let draws = self
            .instances
            .iter()
            .map(|i| (i.mesh, i.material))
            .chain(self.skinned_instances.iter().map(|i| (i.mesh, i.material)));
        for (k, (mesh, material)) in draws.enumerate() {
            if !m.live(mesh.index(), mesh.generation()) {
                return Some(format!("instance {k}: mesh {} was removed", mesh.index()));
            }
            if !mat.live(material.index(), material.generation()) {
                return Some(format!(
                    "instance {k}: material {} was removed",
                    material.index()
                ));
            }
        }
        let overlay = self.overlay.as_ref();
        let sprites = self
            .sprites
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.sprites.iter()));
        for (k, s) in sprites.enumerate() {
            if let Some(h) = s.texture
                && !t.live(h.index(), h.generation())
            {
                return Some(format!("sprite {k}: texture {} was removed", h.index()));
            }
        }
        let texts = self
            .texts
            .iter()
            .chain(overlay.into_iter().flat_map(|o| o.texts.iter()));
        for (k, text) in texts.enumerate() {
            if !f.live(text.font.index(), text.font.generation()) {
                return Some(format!("text {k}: font {} was removed", text.font.index()));
            }
        }
        for (k, font) in self.fonts.iter().enumerate() {
            if f.live(k, f.generation(k)) && !t.live(font.atlas.index(), font.atlas.generation()) {
                return Some(format!(
                    "font {k}: its atlas texture {} was removed",
                    font.atlas.index()
                ));
            }
        }
        None
    }

    /// The handle of the scene's `index`-th mesh (meshes are numbered in
    /// the order they were added).
    pub fn mesh_handle(&self, index: usize) -> Option<MeshHandle> {
        let generation = self.mesh_slots.generation(index);
        self.mesh_slots
            .live(index, generation)
            .then(|| MeshHandle::from_parts(index, generation))
    }

    /// How many times the mesh at `index` was replaced (or removed).
    /// Bumped by every [`Self::set_mesh`] of the mesh at `index`.
    pub fn mesh_revision(&self, index: usize) -> u64 {
        self.mesh_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the texture at `index` was replaced (or removed).
    pub fn texture_revision(&self, index: usize) -> u64 {
        self.texture_revisions.get(index).copied().unwrap_or(0)
    }

    /// How many times the terrain at `index` was replaced or edited.
    pub fn terrain_revision(&self, index: usize) -> u64 {
        self.terrain_revisions.get(index).copied().unwrap_or(0)
    }

    pub fn mesh(&self, handle: MeshHandle) -> Option<&MeshData> {
        if !self.contains_mesh(handle) {
            return None;
        }
        self.meshes.get(handle.index())
    }

    /// An RGBA8 texture (`None` for a [`TextureUpload`] or a foreign handle).
    pub fn texture(&self, handle: TextureHandle) -> Option<&TextureData> {
        match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => Some(t),
            TextureSource::Encoded(_) | TextureSource::Target(_) => None,
        }
    }

    /// How a texture's texels are sampled: decoded from sRGB or as stored
    /// (a render target's are linear); `None` for a foreign handle.
    pub fn texture_color_space(&self, handle: TextureHandle) -> Option<ColorSpace> {
        Some(match self.textures.get(handle.index())? {
            TextureSource::Rgba(t) => t.color_space,
            TextureSource::Encoded(u) => u.desc.color_space,
            TextureSource::Target(_) => ColorSpace::Linear,
        })
    }

    /// A pre-encoded texture (`None` for an RGBA8 one or a foreign handle).
    pub fn texture_upload(&self, handle: TextureHandle) -> Option<&TextureUpload> {
        match self.textures.get(handle.index())? {
            TextureSource::Encoded(t) => Some(t),
            TextureSource::Rgba(_) | TextureSource::Target(_) => None,
        }
    }

    /// Width and height of any texture (a streamed one's whole chain's,
    /// [`RenderScene::set_texture_size`], whatever it holds now).
    pub fn texture_size(&self, handle: TextureHandle) -> Option<(u32, u32)> {
        let size = self.textures.get(handle.index()).map(TextureSource::size)?;
        Some(self.texture_sizes.get(&handle).copied().unwrap_or(size))
    }

    /// The size [`RenderScene::texture_size`] reports for `handle` whatever
    /// levels it holds (`None`: its own): a texture streamed by its mips
    /// (G3-3) reports its whole chain's while it holds its tail, so a
    /// sprite, an image or a layout sized from it does not change when
    /// the levels above arrive. False for a bad handle.
    pub fn set_texture_size(&mut self, handle: TextureHandle, size: Option<(u32, u32)>) -> bool {
        if !self.texture_slots.live(handle.index(), handle.generation()) {
            return false;
        }
        match size {
            Some(s) => self.texture_sizes.insert(handle, s),
            None => self.texture_sizes.remove(&handle),
        };
        true
    }

    /// Every texture `material` samples: its maps and its physical
    /// extensions' (texture arrays aside). Empty for a bad handle.
    pub fn material_textures(&self, material: MaterialHandle) -> Vec<TextureHandle> {
        self.standard_material(material)
            .map(|m| {
                m.textures()
                    .into_iter()
                    .chain(m.physical_textures())
                    .flatten()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The material as added with [`RenderScene::add_material`] or
    /// [`RenderScene::set_material`]; `None` for one that is a
    /// [`StandardMaterial`] (read it with [`RenderScene::standard_material`]).
    pub fn material(&self, handle: MaterialHandle) -> Option<&Material> {
        self.gltf_materials.get(handle.index())?.as_ref()
    }

    /// Any material, in the form it is drawn.
    pub fn standard_material(&self, handle: MaterialHandle) -> Option<&StandardMaterial> {
        self.materials.get(handle.index())
    }

    /// Adds a material shader (`hooks.rs`). The cook validates shader
    /// assets; one added here directly is compiled when first drawn.
    pub fn add_shader(&mut self, shader: crate::MaterialShader) -> ShaderHandle {
        self.shaders.push(shader);
        ShaderHandle(self.shaders.len() as u32 - 1)
    }

    pub fn shaders(&self) -> &[crate::MaterialShader] {
        &self.shaders
    }

    /// Draws `material` with `shader` (`None`: the built-in hooks again),
    /// its parameters at the shader's defaults. False for a bad handle.
    pub fn set_material_shader(
        &mut self,
        material: MaterialHandle,
        shader: Option<ShaderHandle>,
    ) -> bool {
        let i = material.index();
        if !self.material_slots.live(i, material.generation())
            || shader.is_some_and(|s| s.index() >= self.shaders.len())
        {
            return false;
        }
        if self.material_shaders.len() <= i {
            self.material_shaders.resize(i + 1, None);
        }
        self.material_shaders[i] = shader.map(|s| (s, self.shaders[s.index()].defaults()));
        self.material_revisions[i] += 1;
        true
    }

    /// Sets parameter `name` of `material`'s shader (as many numbers as its
    /// type takes; missing ones are 0). False when the material has no
    /// shader or the shader no such parameter. Leaves the material's
    /// revision: the renderer writes changed parameters into the material's
    /// uniform in place (a new buffer and bind group per change cost the
    /// browser's GPU process about 1 ms each).
    pub fn set_shader_param(
        &mut self,
        material: MaterialHandle,
        name: &str,
        value: &[f32],
    ) -> bool {
        let i = material.index();
        let Some(Some((shader, params))) = self.material_shaders.get_mut(i) else {
            return false;
        };
        let Some((slot, kind)) = self.shaders[shader.index()].param(name) else {
            return false;
        };
        let mut v = [0.0; 4];
        for (d, s) in v.iter_mut().zip(value.iter().take(kind.count())) {
            *d = *s;
        }
        params[slot] = v;
        true
    }

    /// `material`'s shader and parameters as a mesh draws it: none for a
    /// shader of [`crate::ShaderMode::Sprite`], which only sprites run.
    pub fn material_shader(
        &self,
        material: MaterialHandle,
    ) -> Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])> {
        self.material_shaders
            .get(material.index())
            .copied()
            .flatten()
            .filter(|(s, _)| self.shaders[s.index()].mode != crate::ShaderMode::Sprite)
    }

    /// `material`'s shader and parameters as a sprite draws it
    /// ([`crate::SpriteShading`]): a shader of hooks or of
    /// [`crate::ShaderMode::Sprite`], none for a full shader.
    pub fn sprite_shader(
        &self,
        material: MaterialHandle,
    ) -> Option<(ShaderHandle, [[f32; 4]; crate::MAX_SHADER_PARAMS])> {
        self.material_shaders
            .get(material.index())
            .copied()
            .flatten()
            .filter(|_| {
                self.material_slots
                    .live(material.index(), material.generation())
            })
            .filter(|(s, _)| {
                matches!(
                    self.shaders[s.index()].mode,
                    crate::ShaderMode::Hooks | crate::ShaderMode::Sprite
                )
            })
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Every material's handle, in insertion order (tools that edit all of them).
    /// Every live material's handle, in slot order.
    pub fn material_handles(&self) -> impl Iterator<Item = MaterialHandle> + use<> {
        let live: Vec<MaterialHandle> = (0..self.materials.len())
            .filter_map(|i| {
                let generation = self.material_slots.generation(i);
                self.material_slots
                    .live(i, generation)
                    .then(|| MaterialHandle::from_parts(i, generation))
            })
            .collect();
        live.into_iter()
    }

    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn meshes(&self) -> &[MeshData] {
        &self.meshes
    }

    pub(crate) fn fonts(&self) -> &[MsdfFont] {
        &self.fonts
    }

    pub(crate) fn textures(&self) -> &[TextureSource] {
        &self.textures
    }

    pub(crate) fn materials(&self) -> impl Iterator<Item = (&StandardMaterial, u64)> {
        self.materials
            .iter()
            .zip(self.material_revisions.iter().copied())
    }
}
