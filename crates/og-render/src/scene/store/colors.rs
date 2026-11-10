use super::*;

impl RenderScene {
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

    pub(crate) fn lut_live(&self, index: usize) -> bool {
        let g = self.lut_slots.generation(index);
        self.lut_slots.live(index, g)
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
}
