use super::*;

impl RenderScene {
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
        self.material_serial += 1;
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
        self.material_serial += 1;
        self.material_slots.remove(i, true);
        // Its aliases end for good: a later material in the slot is another.
        if let Some(a) = self.material_aliases.get_mut(i) {
            *a = None;
        }
        for c in self
            .material_copies
            .get_mut(i)
            .map(std::mem::take)
            .unwrap_or_default()
        {
            let a = &mut self.material_aliases[c as usize];
            if a.is_some_and(|(o, _, _)| o as usize == i) {
                *a = None;
            }
        }
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
        self.material_serial += 1;
        self.restamp_aliases(i);
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
        self.material_serial += 1;
        self.restamp_aliases(i);
        true
    }

    /// Draws `copy` in the batches of `of` while the two draw the same: two
    /// materials of one value (models that each hold their own copy of one
    /// glTF material) draw in one instanced batch. Copy-on-write: an edit
    /// of either that leaves them different (a `set_*`, a shader or a
    /// shader parameter) parts them, and each draws as its own from that
    /// frame on, so an edit of one never shows on the other; the same edit
    /// of the other joins them again. In the frame a shader parameter of
    /// either changes, the two draw apart all the same: equal parameters
    /// now do not make equal previous ones (`previous_shader_params()`).
    /// A removed texture parts them until an edit finds them alike; the
    /// removal of either ends it. False, and
    /// nothing changes, unless both are live and draw the same now (equal
    /// values, the same material shader with equal parameters, or none).
    pub fn alias_material(&mut self, copy: MaterialHandle, of: MaterialHandle) -> bool {
        match self.alias_pair(copy, of) {
            Some((c, o)) if self.draws_alike(c, o) => {
                self.record_alias(c, o);
                true
            }
            _ => false,
        }
    }

    /// As [`RenderScene::alias_material`], also while the two differ now:
    /// `copy` then draws in the batches of `of` from the first edit that
    /// leaves them drawing the same. A model loaded after the game edited
    /// the copies loaded before it (one loaded while the game runs) so
    /// batches with them once the game gives it the same edit. False, and
    /// nothing changes, unless both are live.
    pub fn alias_material_when_alike(&mut self, copy: MaterialHandle, of: MaterialHandle) -> bool {
        let Some((c, o)) = self.alias_pair(copy, of) else {
            return false;
        };
        self.record_alias(c, o);
        true
    }

    /// `copy` and the material `of` draws as, when both are live and differ.
    fn alias_pair(&self, copy: MaterialHandle, of: MaterialHandle) -> Option<(usize, usize)> {
        let live = |h: MaterialHandle| self.material_slots.live(h.index(), h.generation());
        if !live(copy) || !live(of) {
            return None;
        }
        let (c, o) = (copy.index(), self.drawn_material(of.index()));
        (c != o).then_some((c, o))
    }

    /// Material `c` aliased to `o`, holding now while the two draw the same.
    fn record_alias(&mut self, c: usize, o: usize) {
        if self.material_aliases.len() <= c {
            self.material_aliases.resize(c + 1, None);
        }
        let stamps = if self.draws_alike(c, o) {
            (self.material_revisions[c], self.material_revisions[o])
        } else {
            // No revision is ever this: drawn_material parts them.
            (u64::MAX, u64::MAX)
        };
        self.material_aliases[c] = Some((o as u32, stamps.0, stamps.1));
        if self.material_copies.len() <= o {
            self.material_copies.resize(o + 1, Vec::new());
        }
        self.material_copies[o].push(c as u32);
    }

    /// Whether materials `a` and `b` draw the same: equal values and the
    /// same shader with equal parameters (or neither has one).
    fn draws_alike(&self, a: usize, b: usize) -> bool {
        let shader = |i: usize| self.material_shaders.get(i).copied().flatten();
        shader(a) == shader(b) && self.materials[a] == self.materials[b]
    }

    /// After an edit of material `i` (its value, shader or a parameter):
    /// each alias it is in holds from now while the two draw the same and
    /// is parted while they differ ([`RenderScene::alias_material`]).
    pub(super) fn restamp_aliases(&mut self, i: usize) {
        let restamp = |s: &mut Self, c: usize, o: usize| {
            let stamps = if s.draws_alike(c, o) {
                (s.material_revisions[c], s.material_revisions[o])
            } else {
                // No revision is ever this: drawn_material parts them.
                (u64::MAX, u64::MAX)
            };
            s.material_aliases[c] = Some((o as u32, stamps.0, stamps.1));
        };
        if let Some(Some((o, _, _))) = self.material_aliases.get(i).copied() {
            restamp(self, i, o as usize);
        }
        for k in 0..self.material_copies.get(i).map_or(0, Vec::len) {
            let c = self.material_copies[i][k] as usize;
            if self.material_aliases[c].is_some_and(|(o, _, _)| o as usize == i) {
                restamp(self, c, i);
            }
        }
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
}
