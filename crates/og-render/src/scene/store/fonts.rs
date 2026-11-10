use super::*;

impl RenderScene {
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

    pub fn font(&self, handle: FontHandle) -> Option<&MsdfFont> {
        if !self.contains_font(handle) {
            return None;
        }
        self.fonts.get(handle.index())
    }
}
