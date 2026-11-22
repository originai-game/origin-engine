//! The model built into an export scene: measures at rest and posed, interference, and every export format.

use super::*;

/// One part's measures in the world, in the model's unit.
#[derive(Clone, Debug, PartialEq)]
pub struct PartMeasures {
    /// The part's name.
    pub name: String,
    /// Its measures at rest.
    pub measures: Measures,
}

/// A model built for export: every part in the world, in millimetres,
/// posed by the joints' values at export. [`Built::scene`] lends it to the
/// tools.
#[derive(Clone, Debug, Default)]
pub struct Built {
    pub(crate) name: String,
    pub(crate) parts: Vec<(String, Solid, Material, String, Option<crate::tools::Joint>)>,
}

impl Built {
    /// The scene the tools read: parents before children.
    pub fn scene(&self) -> Scene<'_> {
        Scene {
            name: self.name.clone(),
            parts: self
                .parts
                .iter()
                .map(|(name, solid, material, parent, joint)| Part {
                    material: material.clone(),
                    parent: parent.clone(),
                    joint: joint.clone(),
                    ..Part::new(name, solid)
                })
                .collect(),
        }
    }
}

/// What [`Model::glb_atlas`] wrote.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtlasExport {
    /// The GLB.
    pub glb: Vec<u8>,
    /// The packing: where each island went, and the atlas's measures.
    pub atlas: crate::tools::AtlasReport,
    /// Materials the packed parts share, once alike ones are merged.
    pub materials: usize,
    /// Parts with UVs left out of the atlas: their material has a map.
    pub own: Vec<String>,
}

impl Model {
    /// Measures of every part in the world at rest, in id order.
    pub fn measure(&self) -> Result<Vec<PartMeasures>, ModelError> {
        self.parts()
            .into_iter()
            .map(|i| {
                Ok(PartMeasures {
                    name: self.nodes[i.0 as usize].name.clone(),
                    measures: self.world_solid(i)?.measure(),
                })
            })
            .collect()
    }

    /// Measures of every part in the world at the pose (each joint at its
    /// value), in id order, in the model's unit. A part no non-zero joint
    /// value moves measures as at rest, bit for bit; a moved part as moved
    /// (`crate::tools::posed`): counts, area and closure are the rest's,
    /// bounds the pose's, and the volume may differ in its last bits.
    pub fn measure_posed(&self) -> Result<Vec<PartMeasures>, ModelError> {
        if !self.posed() {
            return self.measure();
        }
        let built = self.build()?;
        let solids = crate::tools::posed(&built.scene())?;
        let k = self.unit.millimetres();
        let mut out = Vec::new();
        for i in self.parts() {
            let n = &self.nodes[i.0 as usize];
            let measures = if self.moved_by(i).is_none() {
                self.world_solid(i)?.measure()
            } else {
                let at = built
                    .parts
                    .iter()
                    .position(|p| p.0 == n.name)
                    .ok_or_else(|| ModelError::Refused(n.name.clone(), "not built".into()))?;
                let mut s = solids[at].clone();
                if k != 1.0 {
                    s.map_positions(|p| p / k, false);
                }
                s.measure()
            };
            out.push(PartMeasures {
                name: n.name.clone(),
                measures,
            });
        }
        Ok(out)
    }

    /// Pairs of parts that pass through each other, touch, or hold one
    /// wholly inside the other (`crate::tools::interference`), with the
    /// joints in `pose` at their values and every other joint at rest
    /// (an empty pose is the rest). Bounds are in the model's unit.
    ///
    /// The model is not changed. Errors as [`Model::pose`] on a joint that
    /// does not exist, follows another, or is posed out of its range.
    pub fn interference(&self, pose: &[(String, f64)]) -> Result<Vec<Interfering>, ModelError> {
        let mut m = self.clone();
        for j in &mut m.joints {
            j.value = 0.0;
        }
        for (name, value) in pose {
            m.pose(name, *value)?;
        }
        let k = self.unit.millimetres();
        let mut out = crate::tools::interference(&m.build()?.scene())?;
        if k != 1.0 {
            for x in &mut out {
                x.bounds_min /= k;
                x.bounds_max /= k;
            }
        }
        Ok(out)
    }

    /// The model as an export scene: every live part in the world at rest
    /// (millimetres), each hanging on the body it moves with.
    ///
    /// - A jointed part carries its joint; a jointed group becomes a
    ///   part with no mesh, named after it, which its parts hang on.
    /// - A child with several joints gets one link per joint before the
    ///   last, named `<child>:<joint>` and with no mesh, chained from the
    ///   parent, the first nearest it.
    /// - Joint values are the pose; a follower's comes from its leader.
    pub fn build(&self) -> Result<Built, ModelError> {
        let k = self.unit.millimetres();
        let mm = |f: &Frame| Frame {
            origin: f.origin * k,
            ..*f
        };
        // (name, solid, material, the name it hangs on, joint).
        type Item = (String, Solid, Material, String, Option<crate::tools::Joint>);
        let to_scene = |j: &JointRec| -> crate::tools::Joint {
            let pivot = mm(&j.desc.on);
            let si = |kind: JointKind| if kind == JointKind::Prismatic { k } else { 1.0 };
            let s = si(j.desc.kind);
            crate::tools::Joint {
                name: j.desc.name.clone(),
                kind: j.desc.kind,
                pivot,
                axis: j.axis.map_or(DVec3::ZERO, |a| pivot.vector_to_local(a)),
                value: j.value * s,
                range: j.desc.range.map(|(lo, hi)| (lo * s, hi * s)),
                mimic: j.desc.mimic.as_ref().map(|m| {
                    let lead = self
                        .joints
                        .iter()
                        .find(|l| l.desc.name == m.joint)
                        .map_or(JointKind::Revolute, |l| l.desc.kind);
                    Mimic {
                        joint: m.joint.clone(),
                        multiplier: m.multiplier * s / si(lead),
                    }
                }),
            }
        };
        // Every scene entry with the name of the entry it hangs on, then
        // ordered parents first by passes in a fixed order.
        let mut entries: Vec<Item> = Vec::new();
        let mut bodies: Vec<Id> = self.joints.iter().map(|j| j.child).collect();
        bodies.sort();
        bodies.dedup();
        for &b in &bodies {
            let chain: Vec<&JointRec> = self.joints.iter().filter(|j| j.child == b).collect();
            let bn = self.nodes[b.0 as usize].name.clone();
            let mut hang = self.representative(chain[0].parent);
            for (i, j) in chain.iter().enumerate() {
                let last = i + 1 == chain.len();
                let (name, solid, material) = if last {
                    let solid = if self.nodes[b.0 as usize].solid.is_some() {
                        self.scaled(b, k)?
                    } else {
                        Solid::default()
                    };
                    (bn.clone(), solid, self.material_of(b))
                } else {
                    (
                        format!("{bn}:{}", j.desc.name),
                        Solid::default(),
                        Material::default(),
                    )
                };
                entries.push((name.clone(), solid, material, hang, Some(to_scene(j))));
                hang = name;
            }
        }
        for p in self.parts() {
            if bodies.contains(&p) {
                continue;
            }
            let above = self
                .body(p)
                .map(|x| self.nodes[x.0 as usize].name.clone())
                .unwrap_or_default();
            entries.push((
                self.nodes[p.0 as usize].name.clone(),
                self.scaled(p, k)?,
                self.material_of(p),
                above,
                None,
            ));
        }
        let mut out: Vec<Item> = Vec::with_capacity(entries.len());
        let mut emitted: BTreeSet<String> = BTreeSet::new();
        let mut left: Vec<Option<Item>> = entries.into_iter().map(Some).collect();
        while out.len() < left.len() {
            let before = out.len();
            for slot in &mut left {
                let ready = slot
                    .as_ref()
                    .is_some_and(|e| e.3.is_empty() || emitted.contains(&e.3));
                if ready && let Some(e) = slot.take() {
                    emitted.insert(e.0.clone());
                    out.push(e);
                }
            }
            if out.len() == before {
                return Err(ModelError::Joint(
                    String::new(),
                    "the joints hang parts on each other in a cycle".into(),
                ));
            }
        }
        Ok(Built {
            name: self.name.clone(),
            parts: out,
        })
    }

    pub(crate) fn scaled(&self, id: Id, k: f64) -> Result<Solid, ModelError> {
        let mut s = self.world_solid(id)?;
        if k != 1.0 {
            s.map_positions(|p| p * k, false);
            s = rounded(s);
        }
        // A map spanning `size` units: UVs in images, not units.
        let (material, size) = self.mapped_of(id);
        if material.map.is_some()
            && size != 1.0
            && let Some(uvs) = &mut s.uvs
        {
            for q in uvs.iter_mut().flatten() {
                *q /= size;
            }
        }
        Ok(s)
    }

    /// The model as GLB, posed (`crate::tools::glb`): metres, Y up, joints
    /// as `s3d_joint` extras.
    pub fn glb(&self) -> Result<Vec<u8>, ModelError> {
        Ok(crate::tools::glb_clips(
            &self.build()?.scene(),
            &self.scene_clips(),
        )?)
    }

    /// The clips in the scene's units: millimetres for a slide.
    pub(crate) fn scene_clips(&self) -> Vec<crate::tools::Clip> {
        let k = self.unit.millimetres();
        self.clips
            .iter()
            .map(|c| crate::tools::Clip {
                name: c.name.clone(),
                keys: c
                    .keys
                    .iter()
                    .map(|(time, pose)| crate::tools::ClipKey {
                        time: *time,
                        pose: pose
                            .iter()
                            .map(|(joint, value)| {
                                let slides = self.joints.iter().any(|j| {
                                    j.desc.name == *joint && j.desc.kind == JointKind::Prismatic
                                });
                                (joint.clone(), if slides { value * k } else { *value })
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// The model as GLB ([`Model::glb`]) with its parts' UVs packed into
    /// one atlas (`crate::tools::repack`: every UV island of every part
    /// with UVs and a material without maps, at one texel density, on
    /// `d`'s grid), and parts whose materials then agree (names aside)
    /// sharing one material, named after the first. A part whose material
    /// has a map keeps its own UVs (its map was painted for them) and its
    /// material: [`AtlasExport::own`] names it. Joints, placements and
    /// names are as [`Model::glb`] writes them.
    pub fn glb_atlas(&self, d: &crate::tools::Atlas) -> Result<AtlasExport, ModelError> {
        let built = self.build()?;
        let mut scene = built.scene();
        let packs = |m: &Material, s: &Solid| s.uvs.is_some() && m.maps().is_empty();
        let mut own = Vec::new();
        let mut taken: Vec<usize> = Vec::new();
        let mut solids: Vec<Solid> = Vec::new();
        for (k, p) in scene.parts.iter().enumerate() {
            if packs(&p.material, p.solid) {
                taken.push(k);
                solids.push(p.solid.clone());
            } else if p.solid.uvs.is_some() {
                own.push(p.name.clone());
            }
        }
        if solids.is_empty() {
            return Err(ModelError::Atlas(crate::tools::AtlasError::NoUvs));
        }
        let atlas = crate::tools::repack(&mut solids, d)?;
        // Materials alike (names aside) take the first one's name.
        let bare = |m: &Material| Material {
            name: String::new(),
            ..m.clone()
        };
        let mut firsts: Vec<(Material, String)> = Vec::new();
        for &k in &taken {
            let p = &mut scene.parts[k];
            let b = bare(&p.material);
            let name = match firsts.iter().find(|(m, _)| *m == b) {
                Some((_, n)) => n.clone(),
                None => {
                    let n = if p.material.name.is_empty() {
                        p.name.clone()
                    } else {
                        p.material.name.clone()
                    };
                    firsts.push((b, n.clone()));
                    n
                }
            };
            p.material.name = name;
        }
        for (k, s) in taken.iter().zip(&solids) {
            scene.parts[*k].solid = s;
        }
        let glb = crate::tools::glb_shared(&scene)?;
        Ok(AtlasExport {
            glb,
            atlas,
            materials: firsts.len(),
            own,
        })
    }

    /// Where [`Model::obj`] and [`Model::stl`] put the parts: at the pose
    /// when any joint has a value other than 0, else at rest.
    pub fn mesh_placement(&self) -> Placement {
        if self.posed() {
            Placement::Pose
        } else {
            Placement::Rest
        }
    }

    /// The model's meshes as OBJ text, at its pose when one is set (the
    /// header then ends `posed`; [`Model::mesh_placement`]), else at rest.
    /// Parts no joint moves keep their rest positions bit for bit.
    pub fn obj(&self) -> Result<String, ModelError> {
        Ok(crate::tools::obj_at(
            &self.build()?.scene(),
            self.mesh_placement(),
        )?)
    }

    /// The model's meshes as binary STL, at its pose when one is set (the
    /// header then ends `posed`), else at rest.
    pub fn stl(&self) -> Result<Vec<u8>, ModelError> {
        Ok(crate::tools::stl_at(
            &self.build()?.scene(),
            self.mesh_placement(),
        )?)
    }

    /// The model as URDF (`crate::tools::urdf`): its parts as links at
    /// rest, its joints, and each link's STL named `<prefix>_<part>.stl`.
    pub fn urdf(&self, prefix: &str) -> Result<crate::tools::Urdf, ModelError> {
        self.urdf_with(prefix, &crate::tools::UrdfDesc::default())
    }

    /// [`Model::urdf`] with each closed link's `inertial` at a density
    /// and the joints' limits (`crate::tools::urdf_with`).
    pub fn urdf_with(
        &self,
        prefix: &str,
        d: &crate::tools::UrdfDesc,
    ) -> Result<crate::tools::Urdf, ModelError> {
        Ok(crate::tools::urdf_with(&self.build()?.scene(), prefix, d)?)
    }

    /// The model as a USD stage (`crate::tools::usda`): its parts as
    /// meshes at rest with their materials, maps named
    /// `<prefix>_<part>_<kind>`.
    pub fn usda(&self, prefix: &str) -> Result<crate::tools::Usd, ModelError> {
        Ok(crate::tools::usda(&self.build()?.scene(), prefix)?)
    }

    /// A turntable of the posed model (`crate::tools::preview`).
    pub fn preview(
        &self,
        d: &Preview,
        codec: &crate::tools::Codec,
    ) -> Result<Vec<Image>, ModelError> {
        Ok(crate::tools::preview(&self.build()?.scene(), d, codec)?)
    }
}
