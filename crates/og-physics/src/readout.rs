//! What physics keeps beyond the components, read for drawing and by
//! game code: the points, poses and heights as the last step left them.

use super::*;

/// One bone of a ragdoll rig, in the entity's space at the rest pose.
#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBone {
    /// The parent bone's index in [`RagdollRig::bones`] (it comes earlier); `None` for the root.
    pub parent: Option<u32>,
    /// The bone's rest position and rotation relative to the entity.
    pub rest: (Vec3, Quat),
    /// How long the bone is (to its child, or a guess for a leaf).
    pub length: f32,
    /// The direction the bone points in its own frame (towards its child).
    pub axis: Vec3,
}

/// The bones a ragdoll is made of: registered by the host for an entity
/// with an `og.Ragdoll` ([`PhysicsSystem::set_ragdoll_rig`]); an asset, not
/// world state, like meshes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RagdollRig {
    pub bones: Vec<RagdollBone>,
}

/// A wheel of an `og.Vehicle` as it is now, for drawing (and for smoke,
/// skid marks and a squeal).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelPose {
    pub position: Vec3,
    pub quaternion: Quat,
    /// Whether it touches the ground.
    pub contact: bool,
    /// How much the tyre slides (rapier's skid info turned round): 0 it
    /// grips, towards 1 it asks for more than its grip holds (0.5: twice
    /// as much). 0 in the air and in 2D.
    pub slip: f32,
    /// How fast the tyre's contact moves sideways over the ground, m/s
    /// (along the axle; 0 in the air and in 2D).
    pub side_speed: f32,
    /// Its og.VehicleWheel entity's bits; 0 for an implicit wheel.
    pub entity: u64,
}

/// A cloth's points as the last step left them, row-major.
#[derive(Clone, Debug, PartialEq)]
pub struct ClothPoints {
    pub columns: u32,
    pub rows: u32,
    pub points: Vec<Vec3>,
    /// The points `pins` holds to the entity, ascending.
    pub pinned: Vec<u32>,
    /// Whether it hangs in the xy plane (the 2D profile).
    pub flat: bool,
}

/// An og.SoftBody's points and its surface's triangles, three indices each.
#[derive(Clone, Debug, PartialEq)]
pub struct SoftBodyPoints {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
    /// The points `pinTop` holds to the entity, ascending.
    pub pinned: Vec<u32>,
    /// Whether it was built in the xy plane (the 2D profile: a ring).
    pub flat: bool,
}

/// A height field's heights (relative to its entity's y), row-major, rows along +z.
#[derive(Clone, Debug, PartialEq)]
pub struct WaterHeights {
    pub resolution: u32,
    pub size: f32,
    pub heights: Vec<f32>,
}

/// Runs `$body` with `$s` bound to the active profile's simulation, or
/// answers `None` when there is none yet.
macro_rules! with_sim {
    ($shared:expr, |$s:ident| $body:expr) => {{
        let shared = lock(&$shared);
        #[cfg(feature = "physics3d")]
        if let Some($s) = shared.sim3.as_deref() {
            return $body;
        }
        #[cfg(feature = "physics2d")]
        if let Some($s) = shared.sim2.as_deref() {
            return $body;
        }
        None
    }};
}

/// What physics keeps beyond the components, read for drawing and by game
/// code (`og.physics.ropePoints` and friends): as the last step left it.
impl PhysicsSystem {
    /// The physics overlay's segments with their RGBA colours: collider
    /// shapes, joints and contacts as the last step left them (rapier's
    /// debug render). For drawing only; `None` before the first step.
    pub fn debug_lines(&self) -> Option<Vec<(Vec3, Vec3, [f32; 4])>> {
        with_sim!(self.shared, |s| Some(s.debug_lines()))
    }

    /// The points of the og.Rope on `entity`, start to end.
    pub fn rope_points(&self, entity: Entity) -> Option<Vec<Vec3>> {
        with_sim!(self.shared, |s| s
            .ropes
            .get(&entity.to_bits())
            .map(|r| r.points.clone()))
    }

    /// og.Physics.gravity as the last step applied it (`None` before it).
    pub fn gravity(&self) -> Option<Vec3> {
        with_sim!(self.shared, |s| Some(s.gravity()))
    }

    /// The world's wind the steps move things in, without the motors'
    /// local pushes ([`PhysicsSystem::set_air`]).
    pub fn wind(&self) -> Vec3 {
        lock(&self.shared).air.wind
    }

    /// The air at `p` as the steps take it: the wind and every motor's
    /// push there (m/s).
    pub fn air_at(&self, p: Vec3) -> Vec3 {
        lock(&self.shared).air.at(p)
    }

    /// The force fields' acceleration at `p` as the steps take it (m/s²;
    /// [`PhysicsSystem::set_forces`]).
    pub fn force_at(&self, p: Vec3) -> Vec3 {
        lock(&self.shared).forces.at(p)
    }

    /// The points of the og.Cloth on `entity`.
    pub fn cloth_points(&self, entity: Entity) -> Option<ClothPoints> {
        with_sim!(self.shared, |s| s.cloths.get(&entity.to_bits()).map(|c| {
            let (columns, rows) = c.dims();
            ClothPoints {
                columns,
                rows,
                points: c.points.clone(),
                pinned: (0..rows)
                    .flat_map(|y| (0..columns).map(move |x| (x, y)))
                    .filter(|&(x, y)| verlet::pinned(c.pins, x, y, columns))
                    .map(|(x, y)| y * columns + x)
                    .collect(),
                flat: s.flat(),
            }
        }))
    }

    /// The points of the og.SoftBody on `entity` and the triangles of its
    /// surface (2D: a fan over its ring, for drawing).
    pub fn soft_body_points(&self, entity: Entity) -> Option<SoftBodyPoints> {
        with_sim!(self.shared, |s| s.ext.softs.get(&entity.to_bits()).map(
            |b| {
                SoftBodyPoints {
                    points: b.points.clone(),
                    indices: b.triangles.iter().flatten().copied().collect(),
                    pinned: b.pinned().to_vec(),
                    flat: b.flat(),
                }
            }
        ))
    }

    /// The particle positions of the og.Fluid2D on `entity` (xy).
    pub fn fluid_particles(&self, entity: Entity) -> Option<Vec<Vec2>> {
        with_sim!(self.shared, |s| s
            .fluids
            .get(&entity.to_bits())
            .map(|f| f.positions.clone()))
    }

    /// The mix of the og.Fluid2D on `entity` when it shares a group: its
    /// lane in the group (entity order) and, per particle, how much of it
    /// is each lane's fluid (summing to 1). `None` for a fluid alone.
    pub fn fluid_mix(&self, entity: Entity) -> Option<(u32, Vec<[f32; 4]>)> {
        with_sim!(self.shared, |s| s
            .fluids
            .get(&entity.to_bits())
            .filter(|f| f.slot != u32::MAX)
            .map(|f| (f.slot, f.mix.clone())))
    }

    /// The particle positions of the og.Fluid3D on `entity` (world space).
    pub fn fluid3d_particles(&self, entity: Entity) -> Option<Vec<Vec3>> {
        with_sim!(self.shared, |s| s
            .fluids3
            .get(&entity.to_bits())
            .map(|f| f.positions.clone()))
    }

    /// The mix of the og.Fluid3D on `entity` when it shares a group, as
    /// [`Self::fluid_mix`] is for og.Fluid2D. `None` for a fluid alone.
    pub fn fluid3d_mix(&self, entity: Entity) -> Option<(u32, Vec<[f32; 4]>)> {
        with_sim!(self.shared, |s| s
            .fluids3
            .get(&entity.to_bits())
            .filter(|f| f.slot != u32::MAX)
            .map(|f| (f.slot, f.mix.clone())))
    }

    /// The heights of the og.WaterSurface on `entity`.
    pub fn water_heights(&self, entity: Entity) -> Option<WaterHeights> {
        with_sim!(self.shared, |s| s.surfaces.get(&entity.to_bits()).map(
            |w| {
                WaterHeights {
                    resolution: w.resolution.clamp(8, 256),
                    size: w.size,
                    heights: w.heights.clone(),
                }
            }
        ))
    }

    /// The bodies that cut the og.WaterSurface on `entity` in the last
    /// ticks, for the renderer's ripples (not world state).
    pub fn water_wakes(&self, entity: Entity) -> Option<Wakes> {
        with_sim!(self.shared, |s| s.wakes.get(&entity.to_bits()).cloned())
    }

    /// How far the waves bodies raised on `entity`'s og.WaterSurface or
    /// og.WaterLine stand over its rest level at world point (`x`, `z`),
    /// the entity at `centre` (a line reads `x` alone); 0 outside it.
    /// `None` for an entity with neither, or before its first step.
    pub fn water_ripple(&self, entity: Entity, centre: Vec3, x: f32, z: f32) -> Option<f32> {
        with_sim!(self.shared, |s| water::ripple(
            &s.surfaces,
            &s.lines,
            entity.to_bits(),
            centre,
            x,
            z
        ))
    }

    /// The entities whose og.WaterSurface or og.WaterLine has stepped, in
    /// entity order: the waters [`Self::water_at`] is given the places of.
    pub fn water_fields(&self) -> Vec<u64> {
        let fields = (|| -> Option<Vec<u64>> {
            with_sim!(self.shared, |s| Some(
                s.surfaces.keys().chain(s.lines.keys()).copied().collect()
            ))
        })();
        og_core::sort::set(fields.unwrap_or_default())
            .into_iter()
            .collect()
    }

    /// The water at world `point` at `seconds` of the world's clock
    /// ([`WaterAt`]): of the og.Water bodies `waters` and the height fields
    /// and lines at `places` (each of [`Self::water_fields`] with its
    /// entity's world position), the one standing highest over the point.
    /// Before the first step only `waters` answer.
    pub fn water_at(
        &self,
        waters: &[WaterBody],
        places: &[(u64, Vec3)],
        point: Vec3,
        seconds: f64,
    ) -> Option<WaterAt> {
        let stepped = (|| {
            with_sim!(self.shared, |s| Some(water::at(
                waters,
                &s.surfaces,
                &s.lines,
                places,
                point,
                seconds
            )))
        })();
        stepped
            .unwrap_or_else(|| water::at(waters, &IdMap::new(), &IdMap::new(), &[], point, seconds))
    }

    /// The spring heights of the og.WaterLine on `entity`, left to right
    /// (relative to its entity's y).
    pub fn water_line(&self, entity: Entity) -> Option<Vec<f32>> {
        with_sim!(self.shared, |s| s
            .lines
            .get(&entity.to_bits())
            .map(|l| l.heights.clone()))
    }

    /// The wheels of the og.Vehicle on `entity` as they are now.
    pub fn vehicle_wheels(&self, entity: Entity) -> Option<Vec<WheelPose>> {
        let shared = lock(&self.shared);
        #[cfg(feature = "physics3d")]
        if let Some(s) = shared.sim3.as_deref() {
            return dim3::wheels(s, entity.to_bits());
        }
        #[cfg(feature = "physics2d")]
        if let Some(s) = shared.sim2.as_deref() {
            return dim2::wheels(s, entity.to_bits());
        }
        None
    }

    /// What the last step cost, phase by phase, and which phases ran on the
    /// lanes pool ([`PhysicsStats`]); the default before the first step.
    /// Timings are wall clock, never world state.
    pub fn stats(&self) -> PhysicsStats {
        lock(&self.shared).stats.clone()
    }

    /// The bones of the og.Ragdoll on `entity` as the bodies hold them
    /// (rig order, world space; in 2D at their targets' z); `None` before
    /// its first step.
    pub fn ragdoll_pose(&self, entity: Entity) -> Option<Vec<(Vec3, Quat)>> {
        let shared = lock(&self.shared);
        #[cfg(feature = "physics3d")]
        if let Some(s) = shared.sim3.as_deref() {
            return dim3::ragdoll_pose(s, entity.to_bits());
        }
        #[cfg(feature = "physics2d")]
        if let Some(s) = shared.sim2.as_deref() {
            return dim2::ragdoll_pose(s, entity.to_bits());
        }
        None
    }

    /// Registers the rig an og.Ragdoll on `entity` is built from. Every
    /// bone's parent comes before it. A new rig rebuilds the ragdoll.
    pub fn set_ragdoll_rig(&self, entity: Entity, rig: RagdollRig) -> Result<()> {
        for (i, b) in rig.bones.iter().enumerate() {
            if b.parent.is_some_and(|p| p as usize >= i) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("ragdoll rig bone {i}: its parent must come before it"),
                ));
            }
            if !(b.length.is_finite() && b.rest.0.is_finite() && b.rest.1.is_finite()) {
                return Err(Error::new(
                    ErrorKind::Invalid,
                    format!("ragdoll rig bone {i} is not finite"),
                ));
            }
        }
        let mut shared = lock(&self.shared);
        shared.rigs.insert(entity.to_bits(), Arc::new(rig));
        Ok(())
    }

    /// Whether `entity` has a ragdoll rig registered.
    pub fn has_ragdoll_rig(&self, entity: Entity) -> bool {
        lock(&self.shared).rigs.contains_key(&entity.to_bits())
    }

    /// Registers the rig the og.Garment on `entity` is built from, under
    /// `key` (the host's name for it: a model's mesh); a rig of another key
    /// rebuilds the garment on the skin at its next step.
    pub fn set_garment_rig(&self, entity: Entity, key: u64, rig: GarmentRig) -> Result<()> {
        rig.check()?;
        lock(&self.shared)
            .garment_rigs
            .insert(entity.to_bits(), (key, Arc::new(rig)));
        Ok(())
    }

    /// The key of the garment rig registered for `entity`, if any.
    pub fn garment_rig_key(&self, entity: Entity) -> Option<u64> {
        lock(&self.shared)
            .garment_rigs
            .get(&entity.to_bits())
            .map(|r| r.0)
    }

    /// Where the skin puts the points of the garment on `entity` this tick
    /// (rig order, world space), for its next step. A garment given no pose
    /// in a tick keeps its points.
    pub fn set_garment_pose(&self, entity: Entity, pose: GarmentPose) {
        lock(&self.shared)
            .garment_poses
            .insert(entity.to_bits(), pose);
    }

    /// The points of the og.Garment on `entity` after the last step and the
    /// targets that step held them to (rig order, world space).
    pub fn garment_points(&self, entity: Entity) -> Option<GarmentPoints> {
        with_sim!(self.shared, |s| s.ext.garments.get(&entity.to_bits()).map(
            |g| GarmentPoints {
                points: g.points.clone(),
                targets: g.targets.clone(),
            }
        ))
    }

    /// The animation's bone poses (rig order, world space) the ragdoll on
    /// `entity` follows at its next step while its weight is 0.
    pub fn set_ragdoll_targets(&self, entity: Entity, targets: Vec<(Vec3, Quat)>) {
        lock(&self.shared)
            .ragdoll_targets
            .insert(entity.to_bits(), targets);
    }

    /// The og.Water bodies the next steps float og.Buoyancy bodies on (the
    /// host reads them from the world as each tick starts, in entity
    /// order), replacing the last ones.
    pub fn set_waters(&self, waters: Vec<WaterBody>) {
        lock(&self.shared).waters = waters;
    }

    /// The world's air (the host reads og.Wind and the og.WindMotors as
    /// each tick starts) the next steps move things through. Every
    /// og.Rope, og.Cloth and og.Garment is pushed by the air where it
    /// hangs, its velocity as an acceleration per second added to their
    /// own `wind`; a body with og.RigidBody.airDrag is carried towards it.
    pub fn set_air(&self, air: og_env::air::Air) {
        lock(&self.shared).air = air;
    }

    /// The world's force fields (the host reads the og.ForceFields as
    /// each tick starts) the next steps accelerate things by, whatever
    /// their mass. A dynamic body takes the fields where its centre of
    /// mass is and stays awake while one reaches it; an og.Rope, og.Cloth,
    /// og.Garment, og.SoftBody and a fluid take them at the middle of
    /// their points; an og.CharacterController with gravity takes their
    /// vertical part where it stands.
    pub fn set_forces(&self, forces: og_env::force::Forces) {
        lock(&self.shared).forces = forces;
    }
}
