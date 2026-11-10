use super::*;

impl RenderScene {
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
}
