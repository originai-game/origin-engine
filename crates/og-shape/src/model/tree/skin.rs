//! Skinning, humanoid auto-rigging and morph targets on model parts.

use super::*;
use crate::mesh::Morph;
use crate::skin::{BindReport, FitReport, Joint, Rig, RigBone, Skeleton};

impl Model {
    /// Skins part `id` with region-based `rig` (pivots in world space,
    /// `blend` and `max_edge` in the model's unit).
    pub fn skin(&mut self, id: Id, rig: &Rig) -> Result<(), ModelError> {
        let placement = self.node(id)?.placement;
        let inv = placement.inverse();
        let local_rig = Rig {
            bones: rig
                .bones
                .iter()
                .map(|b| RigBone {
                    name: b.name.clone(),
                    parent: b.parent.clone(),
                    pivot: inv.compose(&b.pivot),
                    regions: b.regions.clone(),
                })
                .collect(),
            blend: rig.blend,
            max_edge: rig.max_edge,
        };
        let solid = self.solid_of(id)?;
        let skinned = crate::skin::skin(solid, &local_rig)?;
        self.nodes[id.0 as usize].solid = Some(skinned);
        Ok(())
    }

    /// Binds part `id` to point-based `skeleton` via bone-heat skinning
    /// (joint positions in world space).
    pub fn bind(&mut self, id: Id, skeleton: &Skeleton) -> Result<BindReport, ModelError> {
        let placement = self.node(id)?.placement;
        let inv = placement.inverse();
        let local = Skeleton {
            joints: skeleton
                .joints
                .iter()
                .map(|j| Joint {
                    name: j.name.clone(),
                    parent: j.parent.clone(),
                    at: inv.point_to_world(j.at),
                })
                .collect(),
        };
        let solid = self.solid_of(id)?;
        let (skinned, report) = crate::skin::bind(solid, &local)?;
        self.nodes[id.0 as usize].solid = Some(skinned);
        Ok(report)
    }

    /// Fits a Mixamo-named humanoid skeleton to part `id` (which must stand up
    /// +Z, facing -Y in world space) and, when `bind` is true, binds its skin
    /// weights via bone-heat.
    pub fn humanoid(
        &mut self,
        id: Id,
        bind: bool,
    ) -> Result<(Skeleton, FitReport, Option<BindReport>), ModelError> {
        let world_solid = self.world_solid(id)?;
        let (skeleton, fit) = crate::skin::humanoid(&world_solid)?;
        let bound = if bind {
            Some(self.bind(id, &skeleton)?)
        } else {
            None
        };
        Ok((skeleton, fit, bound))
    }

    /// Adds a named morph target to part `id` from another live part `target`
    /// with the same vertex count (deltas computed in `id`'s local space).
    pub fn morph_target(
        &mut self,
        id: Id,
        name: &str,
        target: Id,
        weight: f64,
    ) -> Result<(), ModelError> {
        let placement = self.node(id)?.placement;
        let inv = placement.inverse();
        let target_world = self.world_solid(target)?;
        let base = self.solid_of(id)?;
        if base.positions.len() != target_world.positions.len() {
            return Err(ModelError::Morph(crate::mesh::MorphError::VertexCount {
                target: name.to_string(),
                deltas: target_world.positions.len(),
                vertices: base.positions.len(),
            }));
        }
        let deltas: Vec<DVec3> = base
            .positions
            .iter()
            .zip(&target_world.positions)
            .map(|(&p, &tp)| inv.point_to_world(tp) - p)
            .collect();
        self.push_morph(id, name, weight, deltas)
    }

    /// Adds a named morph target to part `id` from explicit per-vertex
    /// world-space offset vectors.
    pub fn morph_deltas(
        &mut self,
        id: Id,
        name: &str,
        world_deltas: &[DVec3],
        weight: f64,
    ) -> Result<(), ModelError> {
        let placement = self.node(id)?.placement;
        let inv = placement.inverse();
        let deltas: Vec<DVec3> = world_deltas
            .iter()
            .map(|&d| inv.vector_to_world(d))
            .collect();
        self.push_morph(id, name, weight, deltas)
    }

    /// Adds a named morph target to part `id` by translating and/or scaling
    /// vertices belonging to faces matching `regions` (empty: all vertices).
    pub fn morph_regions(
        &mut self,
        id: Id,
        name: &str,
        regions: &[String],
        world_offset: DVec3,
        world_scale: Option<DVec3>,
        weight: f64,
    ) -> Result<(), ModelError> {
        let placement = self.node(id)?.placement;
        let inv = placement.inverse();
        let base = self.solid_of(id)?;
        let n = base.positions.len();
        let mut mask = vec![false; n];
        if regions.is_empty() {
            mask.fill(true);
        } else {
            let tag_matches = |tag: &str, region: &str| {
                tag == region
                    || (tag.len() > region.len()
                        && tag.starts_with(region)
                        && tag.as_bytes()[region.len()] == b'.')
            };
            for (ti, tri) in base.triangles.iter().enumerate() {
                let matched = base.face_tags.get(ti).is_some_and(|tags| {
                    tags.iter().any(|&t| {
                        let name = base.names.name(t);
                        regions.iter().any(|r| tag_matches(name, r))
                    })
                });
                if matched {
                    for &vi in tri {
                        if let Some(slot) = mask.get_mut(vi as usize) {
                            *slot = true;
                        }
                    }
                }
            }
        }
        let selected: Vec<usize> = mask
            .iter()
            .enumerate()
            .filter_map(|(i, &on)| on.then_some(i))
            .collect();
        if selected.is_empty() {
            let mut available: Vec<String> = base
                .face_tags
                .iter()
                .flat_map(|tags| tags.iter())
                .map(|&t| base.names.name(t).to_string())
                .filter(|s| !s.is_empty())
                .collect();
            available.sort();
            available.dedup();
            return Err(ModelError::NoSuchFace {
                part: self.node(id)?.name.clone(),
                name: regions.join(", "),
                available,
            });
        }
        let world_positions: Vec<DVec3> = base
            .positions
            .iter()
            .map(|&p| placement.point_to_world(p))
            .collect();
        let centroid = selected
            .iter()
            .map(|&i| world_positions[i])
            .fold(DVec3::ZERO, |a, b| a + b)
            / selected.len() as f64;
        let mut deltas = vec![DVec3::ZERO; n];
        for i in selected {
            let wp = world_positions[i];
            let scaled = match world_scale {
                Some(s) => centroid + (wp - centroid) * s,
                None => wp,
            };
            let moved = scaled + world_offset;
            deltas[i] = inv.point_to_world(moved) - base.positions[i];
        }
        self.push_morph(id, name, weight, deltas)
    }

    fn push_morph(
        &mut self,
        id: Id,
        name: &str,
        weight: f64,
        deltas: Vec<DVec3>,
    ) -> Result<(), ModelError> {
        if name.is_empty() {
            return Err(ModelError::EmptyName);
        }
        let solid = self.solid_of(id)?;
        let mut next = solid.morphs.clone().unwrap_or_default();
        next.targets.push(Morph {
            name: name.to_string(),
            weight,
            deltas,
        });
        next.check(solid.positions.len())?;
        let solid = self.nodes[id.0 as usize]
            .solid
            .as_mut()
            .expect("solid_of succeeded");
        solid.morphs = Some(next);
        Ok(())
    }
}
