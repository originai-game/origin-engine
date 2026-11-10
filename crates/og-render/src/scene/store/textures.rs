use super::*;

impl RenderScene {
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

    /// Whether slot `index` holds a live LUT.
    /// Whether texture slot `index` holds a live texture (not removed): the
    /// renderer frees a removed slot's GPU texture instead of uploading.
    pub(crate) fn texture_live(&self, index: usize) -> bool {
        let g = self.texture_slots.generation(index);
        self.texture_slots.live(index, g)
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
                self.material_serial += 1;
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
                self.material_serial += 1;
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
}
