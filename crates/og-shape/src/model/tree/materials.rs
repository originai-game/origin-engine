//! Materials on parts, mapped textures and the UVs a model asks for.

use super::*;

impl Model {
    /// `material` on each node: a group's reaches every part under it
    /// that names none of its own (the nearest wins).
    pub fn material(&mut self, ids: &[Id], material: &Material) -> Result<(), ModelError> {
        self.material_mapped(ids, material, 1.0)
    }

    /// [`Model::material`] with its map (`material.map`, a base-colour
    /// texture) spanning `map_size` units of the parts' UVs, which are in
    /// the model's unit: export and measure divide the UVs of the parts
    /// it reaches by `map_size`, so the image covers that many units each
    /// way and repeats beyond. Refused: a size that is not positive, or a
    /// map on a part with no UVs.
    pub fn material_mapped(
        &mut self,
        ids: &[Id],
        material: &Material,
        map_size: f64,
    ) -> Result<(), ModelError> {
        if !(map_size.is_finite() && map_size > 0.0) {
            return Err(ModelError::BadValue(
                "mapSize (the units the map spans; positive)".into(),
            ));
        }
        let m = material;
        for v in m
            .color
            .iter()
            .flatten()
            .chain(&m.emissive)
            .chain(m.roughness.iter())
            .chain(m.opacity.iter())
            .chain([&m.metalness])
        {
            if !v.is_finite() {
                return Err(ModelError::BadValue("a material value".into()));
            }
        }
        for &id in ids {
            self.node(id)?;
        }
        if m.map.is_some() {
            for &id in ids {
                for i in self.subtree(id) {
                    let n = &self.nodes[i.0 as usize];
                    if n.solid.as_ref().is_some_and(|s| s.uvs.is_none()) {
                        return Err(ModelError::Refused(
                            n.name.clone(),
                            "a material with a map needs UVs on the parts it reaches: give the \
                             part `uvs: true` (or the model `uvs`) when it is made"
                                .into(),
                        ));
                    }
                }
            }
        }
        for &id in ids {
            self.nodes[id.0 as usize].material = Some((material.clone(), map_size));
        }
        Ok(())
    }

    pub(crate) fn material_of(&self, id: Id) -> Material {
        self.mapped_of(id).0
    }

    /// The nearest material and the size its map spans.
    pub(crate) fn mapped_of(&self, id: Id) -> (Material, f64) {
        let mut at = Some(id);
        while let Some(i) = at {
            let n = &self.nodes[i.0 as usize];
            if let Some(m) = &n.material {
                return m.clone();
            }
            at = n.parent;
        }
        (Material::default(), 1.0)
    }

    /// Asks for UVs on every part made from now on (`docs/design.md` §4):
    /// generators give them in the model's unit on the surface. Refused
    /// while a part without UVs exists: give it `uvs` itself when it is
    /// made, or ask first.
    pub fn ask_uvs(&mut self) -> Result<(), ModelError> {
        if let Some(n) = self
            .nodes
            .iter()
            .find(|n| !n.used_up && n.solid.as_ref().is_some_and(|s| s.uvs.is_none()))
        {
            return Err(ModelError::Refused(
                n.name.clone(),
                "it was made without UVs: ask for UVs before the first part, or give each part \
                 `uvs: true`"
                    .into(),
            ));
        }
        self.uvs = true;
        Ok(())
    }

    /// Whether parts get UVs unless they say otherwise ([`Model::ask_uvs`]).
    pub fn wants_uvs(&self) -> bool {
        self.uvs
    }

    /// UV measures of part `name` as exported: its UVs divided by its
    /// material's map size (`crate::tools::measure`). `None` for a part
    /// without UVs or no such part.
    pub fn uv_measures(&self, name: &str) -> Option<crate::tools::UvMeasures> {
        let id = self.find(name)?;
        let mut s = self.solid_of(id).ok()?.clone();
        let (material, size) = self.mapped_of(id);
        if material.map.is_some() && size != 1.0 {
            for q in s.uvs.as_mut()?.iter_mut().flatten() {
                *q /= size;
            }
        }
        crate::tools::measure(&s, &crate::tools::Measure::default()).uv
    }
}
