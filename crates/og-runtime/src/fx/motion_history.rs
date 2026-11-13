//! Per-object motion for TAA and motion blur: where each instance and joint was drawn before.

use super::*;

/// Per-object motion's state (`FxState`'s last two ticks and what the last
/// display frame drew), kept by a presentation checkpoint: the first frame
/// after a restore moves its objects from where the saved run's frame
/// before drew them, so TAA and motion blur reproject as that run did.
#[derive(Clone, Default)]
pub(crate) struct MotionCheckpoint {
    tick: u64,
    current: Vec<((u64, u32), Mat4)>,
    previous: Vec<((u64, u32), Mat4)>,
    drawn: Vec<((u64, u32), Mat4)>,
    drawn_joints: IdMap<u64, Vec<Mat4>>,
    drawn_weights: BTreeMap<(u64, usize), [f32; og_render::MAX_MORPH_TARGETS]>,
    drawn_deformed: BTreeMap<(u64, usize), (Mat4, og_render::DeformedVertices)>,
}

impl FxState {
    pub(crate) fn motion_checkpoint(&self) -> MotionCheckpoint {
        MotionCheckpoint {
            tick: self.tick,
            current: self.current.clone(),
            previous: self.previous.clone(),
            drawn: self.drawn.clone(),
            drawn_joints: self.drawn_joints.clone(),
            drawn_weights: self.drawn_weights.clone(),
            drawn_deformed: self.drawn_deformed.clone(),
        }
    }

    pub(crate) fn restore_motion(&mut self, c: &MotionCheckpoint) {
        self.tick = c.tick;
        self.current.clone_from(&c.current);
        self.previous.clone_from(&c.previous);
        self.previous_by_key = None;
        self.drawn.clone_from(&c.drawn);
        self.drawn_joints.clone_from(&c.drawn_joints);
        self.drawn_weights.clone_from(&c.drawn_weights);
        self.drawn_deformed.clone_from(&c.drawn_deformed);
    }
}

/// Whether the renderer resolved TAA, whose motion vectors want the
/// previous frame's transforms and joints whatever the post effects.
pub(crate) fn taa(rs: &RenderState) -> bool {
    rs.backend.as_ref().is_some_and(|b| b.antialias() == "taa")
}

/// Per-object motion: this tick's instance transforms, keyed, and last
/// tick's for `previous_transforms` (same order as the instances); a
/// display frame drawn between ticks (`crate::interpolate`) moves from the
/// display frame before it instead, as drawn, since the shutter is a
/// fraction of the frame. An entity in `cut` (teleported into the frame:
/// `Hierarchy::cut`) moves from where it is drawn, so it does not smear.
/// Under TAA without motion blur every frame moves from the frame drawn
/// before it (TAA's history is that frame), instances and posed models'
/// joints alike (`previous_joint_palettes`). Only what is drawn is read:
/// the simulation never sees any of it.
pub(crate) fn motion(rs: &mut RenderState, keys: &[(u64, u32)], tick: u64, cut: &IdMap<u64, ()>) {
    let taa = taa(rs);
    let RenderState {
        scene,
        fx,
        interp,
        atmo,
        ..
    } = rs;
    if fx.tick != tick {
        std::mem::swap(&mut fx.previous, &mut fx.current);
        fx.previous_by_key = None;
        fx.tick = tick;
    }
    fx.current.clear();
    fx.current.extend(
        keys.iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| (*k, i.transform)),
    );
    scene.objects.previous_transforms.clear();
    scene.objects.previous_joint_palettes.clear();
    scene.objects.previous_morph_weights.clear();
    scene.objects.previous_deformed_vertices.clear();
    let blur = scene.look.post.motion_blur.is_some();
    let keyed = keys.len() == scene.objects.instances.len();
    if blur && keyed {
        let previous = fx
            .previous_by_key
            .get_or_insert_with(|| fx.previous.iter().copied().collect());
        scene.objects.previous_transforms = keys
            .iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| previous.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
    // Kept only while motion blur or TAA is on (a copy of every instance's
    // transform a frame otherwise).
    if !blur && !taa {
        fx.drawn.clear();
        fx.drawn_joints.clear();
        fx.drawn_weights.clear();
        fx.drawn_deformed.clear();
        return;
    }
    // A capture (every extraction but a display frame's) draws the tick as
    // a still frame: its warm-up redraws one scene, so its instances and
    // joints move with nothing but the blur's tick before, and the frames
    // the display drew (or did not draw: `og run --capture-only`) neither
    // reach it nor are moved by it (docs/design.md §2.4).
    if !interp.display {
        return;
    }
    joints(scene, atmo, fx);
    let from_drawn = if blur {
        interp.drawn.is_some() && !scene.objects.previous_transforms.is_empty()
    } else {
        keyed
    };
    if from_drawn {
        let last: BTreeMap<(u64, u32), Mat4> = fx.drawn.iter().copied().collect();
        scene.objects.previous_transforms = keys
            .iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| last.get(k).copied().unwrap_or(i.transform))
            .collect();
    }
    if !cut.is_empty() {
        for ((k, i), p) in keys
            .iter()
            .zip(&scene.objects.instances)
            .zip(&mut scene.objects.previous_transforms)
        {
            if cut.contains_key(&k.0) {
                *p = i.transform;
            }
        }
    }
    fx.drawn.clear();
    fx.drawn.extend(
        keys.iter()
            .zip(&scene.objects.instances)
            .map(|(k, i)| (*k, i.transform)),
    );
}

/// TAA's skinned motion: each joint palette's joints as the frame before
/// drew them, in the world (`previous_joint_palettes`, in palette order;
/// a model new to the frame, or whose joint count changed, as it is drawn
/// now), and this frame's kept for the next.
fn joints(
    scene: &mut og_render::RenderScene,
    atmo: &crate::atmosphere::AtmoState,
    fx: &mut FxState,
) {
    let mut entity_of = vec![None; scene.objects.joint_palettes.len()];
    for (&e, &p) in &atmo.palettes {
        if let Some(slot) = entity_of.get_mut(p as usize) {
            *slot = Some(e);
        }
    }
    let mut at = vec![None; scene.objects.joint_palettes.len()];
    for s in &scene.objects.skinned_instances {
        if let Some(t) = at.get_mut(s.palette as usize) {
            t.get_or_insert(s.transform);
        }
    }
    let mut now = IdMap::new();
    scene.objects.previous_joint_palettes = scene
        .objects
        .joint_palettes
        .iter()
        .zip(entity_of.iter().zip(&at))
        .map(|(p, (e, t))| {
            let t = t.unwrap_or(Mat4::IDENTITY);
            let world: Vec<Mat4> = p.matrices.iter().map(|m| t * *m).collect();
            let before = e
                .and_then(|e| fx.drawn_joints.get(&e))
                .filter(|b| b.len() == world.len())
                .cloned()
                .unwrap_or_else(|| world.clone());
            if let Some(e) = e {
                now.insert(*e, world);
            }
            og_render::JointPalette::from(before)
        })
        .collect();
    fx.drawn_joints = now;
    // Each skinned instance's weights the frame before (its own where it
    // was not drawn then: no morph motion), and this frame's kept.
    let mut weights = BTreeMap::new();
    scene.objects.previous_morph_weights = scene
        .objects
        .skinned_instances
        .iter()
        .map(|s| {
            let Some(e) = entity_of.get(s.palette as usize).copied().flatten() else {
                return s.morph_weights;
            };
            let key = (e, s.mesh.index());
            weights.insert(key, s.morph_weights);
            fx.drawn_weights
                .get(&key)
                .copied()
                .unwrap_or(s.morph_weights)
        })
        .collect();
    fx.drawn_weights = weights;
    // Retain the draw inputs, not CPU-skinned vertices. The fallback evaluates
    // the saved pose in this frame's instance space; compute keeps GPU history.
    let mut drawn = BTreeMap::new();
    scene.objects.previous_deformed_vertices = scene
        .objects
        .deformed_vertices
        .iter()
        .zip(&fx.deformed_keys)
        .map(|(d, &(key, world))| {
            let before = fx
                .drawn_deformed
                .get(&key)
                .filter(|(_, b)| b.vertex_count() == d.vertex_count())
                .map(|(old_world, b)| {
                    let transform = world.inverse() * *old_world;
                    let mut before = b.clone();
                    if let Some(g) = &mut before.garment {
                        g.output_transform = transform;
                    } else {
                        before
                            .positions
                            .iter_mut()
                            .for_each(|p| *p = transform.transform_point3(*p));
                    }
                    before
                })
                .unwrap_or_else(|| d.clone());
            drawn.insert(key, (world, d.clone()));
            before
        })
        .collect();
    fx.drawn_deformed = drawn;
}
