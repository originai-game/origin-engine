//! What the shadow passes draw from: the static shadow cache's split (the
//! batches that stood still long enough to stay in the cached layers) and
//! the weather occlusion map's view, drawn again only when what it was
//! drawn from changes.

use super::*;

pub(super) struct PrepareShadowInputsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub batches: &'a [Batch],
    pub sorted: u32,
    pub submission: &'a [u32],
    pub sorted_serial: u64,
    pub sorted_changed: Option<&'a [u32]>,
    pub transforms: &'a [instances::GpuTransform],
    pub spheres: &'a [(Vec3, f32)],
    pub debris_instances: &'a HashMap<u32, (usize, Mat4)>,
}

pub(super) struct PrepareShadowInputsOutputs {
    pub shadow_split: shadow_cache::Split,
    pub shelter: Option<(Mat4, Frustum, u64, bool)>,
    pub shelter_redraw: bool,
}

impl Renderer {
    /// Splits the static shadow cache and places the weather's shelter map.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_shadow_inputs(
        &mut self,
        PrepareShadowInputsInputs {
            scene,
            camera,
            batches,
            sorted,
            submission,
            sorted_serial,
            sorted_changed,
            transforms,
            spheres,
            debris_instances,
        }: PrepareShadowInputsInputs<'_>,
    ) -> PrepareShadowInputsOutputs {
        // The static shadow cache's split (shadow_cache.rs): which batches
        // stood still long enough to stay in the cached shadow layers.
        let shadow_split = {
            let wind = |w: Option<&crate::scatter::Wind>| w.is_some_and(|w| w.strength != 0.0);
            let swaying = wind(scene.atmosphere.wind.as_ref())
                || wind(self.history.previous_wind.as_ref().map(|w| &w.0));
            let (meshes, materials) = (&self.cache.meshes, &self.cache.materials);
            // Hashed again only where an instance or its mesh's or
            // material's revision changed (shadow_cache.rs `InstanceHashes`).
            let hashes = self.shadows.shadow_hashes.update(
                (sorted as usize).min(submission.len()),
                (sorted_serial, sorted_changed),
                meshes.iter().map(|m| m.revision),
                materials.iter().map(|m| m.revision),
                |k| {
                    let inst = &scene.objects.instances[submission[k] as usize];
                    let (mesh, material) = (inst.mesh.index(), inst.material.index());
                    let revisions = [meshes[mesh].revision, materials[material].revision];
                    let hash = shadow_cache::instance_hash(&transforms[k], inst, revisions);
                    (hash, (mesh as u32, material as u32))
                },
            );
            // A batch drawing debris that may still move is redrawn each
            // frame: the GPU moves it where the CPU cannot see.
            let debris_moving = crate::debris::moving(scene);
            let moves = |b: &Batch| {
                debris_moving
                    && submission[b.first as usize..(b.first + b.count) as usize]
                        .iter()
                        .any(|i| debris_instances.contains_key(i))
            };
            let info: Vec<shadow_cache::BatchInfo> = batches
                .iter()
                .map(|b| shadow_cache::BatchInfo {
                    first: b.first,
                    count: b.count,
                    cast: b.cast,
                    always_dynamic: b.simulated
                        || shadow_cache::deforms(b.foliage, swaying, b.shader.is_some())
                        || moves(b),
                })
                .collect();
            self.shadows.shadow_cache.split(
                scene.objects.instances.len(),
                &info,
                &submission[..sorted as usize],
                hashes,
                spheres,
            )
        };
        // The weather occlusion map (weather.rs `shelter_view`), in the
        // cascade array's last layer (`fit_shelter_layer` gave it one):
        // drawn again only when what it was drawn from changes.
        let shelter = scene
            .atmosphere
            .surface_weather
            .as_ref()
            .filter(|_| self.shadows.cascade_map.layers.len() as u32 > self.frame_cascades(scene))
            .and_then(|w| {
                let (view_proj, cell) =
                    weather::shelter_view(w, camera.position, scene.frame.origin)?;
                let frustum = Frustum::from_view_proj(view_proj);
                let mut inside = vec![false; spheres.len()];
                for b in batches.iter().filter(|b| b.cast) {
                    inside[b.first as usize..(b.first + b.count) as usize].fill(true);
                }
                frustum.test_spheres(&mut inside, spheres);
                let key = weather::shelter_key(
                    w,
                    cell,
                    spheres
                        .iter()
                        .zip(&inside)
                        .filter(|(_, i)| **i)
                        .map(|(s, _)| *s),
                    scene.world.terrains.len(),
                );
                Some((
                    view_proj,
                    frustum,
                    key,
                    self.shadows.shelter_key != Some(key),
                ))
            });
        let shelter_redraw = shelter.as_ref().is_some_and(|s| s.3);
        PrepareShadowInputsOutputs {
            shadow_split,
            shelter,
            shelter_redraw,
        }
    }
}
