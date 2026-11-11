//! The local lights' stage: the clusters they reach, their records and
//! shadow faces, the contact lights, the cookies past the shadow layers, the
//! volumetric fog's plan, the records' upload and the planar reflections'
//! clusters, in that order.

use super::*;

pub(super) struct PrepareLightsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub contact: bool,
    pub volumetric_fog: &'a Option<crate::volumetric::VolumetricFog>,
    pub reflections: &'a [camera::PlanarReflection],
}

pub(super) struct PrepareLightsOutputs {
    pub lights_gpu: Vec<frame::LightGpu>,
    pub local_shadows: Vec<frame::LocalShadow>,
    pub shadowed_dropped: u32,
    pub contact_rows: [[f32; 4]; 4],
    pub volumetric: Option<()>,
}

impl Renderer {
    /// Prepares and uploads the frame's local lights.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_lights(
        &mut self,
        PrepareLightsInputs {
            scene,
            camera,
            cam,
            view_frustum,
            contact,
            volumetric_fog,
            reflections,
        }: PrepareLightsInputs<'_>,
    ) -> PrepareLightsOutputs {
        // Local lights: the clusters they reach, their records, their shadow faces.
        let (mut lights_gpu, local_shadows, shadowed_dropped) = self.lights.prepare(
            scene,
            &clusters::ClusterView {
                camera: cam,
                frustum: view_frustum,
                position: camera.position,
                near: camera.near,
                far: camera.far,
                width: self.width,
                height: self.height,
            },
            camera.position,
            frame::LocalShadowBudget {
                max_shadowed: self.info.shadowed_lights.min(scene.quality.shadowed_lights) as usize,
                rect_shadows: scene.quality.rect_shadows,
                point: self.info.point_shadows,
                shadow_size: self.shadows.local_shadow_size,
            },
        );
        debug_assert!(
            local_shadows
                .last()
                .is_none_or(|ls| ls.layer + ls.faces.len() <= self.shadows.local_map.layers.len())
        );
        // The contact lights (frame.rs `contact_lights`): their view-space
        // positions for the contact pass, and in their records' shadow
        // slots a quarter per mask channel past the layer, which the lit
        // shaders' `contact_shadow` reads.
        let mut contact_rows = [[0.0f32; 4]; 4];
        if contact {
            for (c, ls) in
                frame::contact_lights(&local_shadows, &scene.lighting.lights, camera.position)
                    .into_iter()
                    .enumerate()
            {
                let p = cam
                    .view
                    .transform_point3(scene.lighting.lights[ls.light].position);
                contact_rows[c] = p.extend(1.0).to_array();
                lights_gpu[ls.record].direction_slot[3] += 0.25 * (c + 1) as f32;
            }
            contact_rows[3] = CONTACT_PARAMS;
            self.post.contact.prepare(
                &self.queue,
                cam,
                camera.far,
                &contact_rows,
                (self.width, self.height),
            );
        }
        // Cookies in the local shadow array past its shadow layers (area.rs).
        let cookie_base =
            shadows::capacity(self.info.shadowed_lights as usize, self.info.point_shadows) as u32;
        let textures = &self.cache.textures;
        self.shadows.prepare_cookies(
            (&self.device, &self.queue),
            scene,
            &self.lights.clusters.kept,
            &mut lights_gpu,
            cookie_base,
            |h| textures.get(h.index()).map(|t| &t.view),
        );
        let volumetric = VolumetricPass::plan(
            &mut self.volumetric,
            &self.device,
            &self.queue,
            &self.frame_layout,
            volumetric_fog.as_ref(),
            (self.width, self.height),
            scene,
        );
        self.lights.write(&self.queue, &lights_gpu);
        // The planar reflection's clusters: the same light records seen by
        // the mirrored camera (its oblique projection moves only depth).
        for (k, (_, mirrored, f, _, (rw, rh))) in reflections.iter().enumerate() {
            self.reflections.reflection_clusters[k].build_kept(
                &scene.lighting.lights,
                &self.lights.clusters.kept,
                &clusters::ClusterView {
                    camera: f,
                    frustum: &Frustum::from_view_proj(f.view_proj),
                    position: mirrored.position,
                    near: mirrored.near,
                    far: mirrored.far,
                    width: *rw,
                    height: *rh,
                },
            );
            if !lights_gpu.is_empty() {
                self.queue.write_buffer(
                    &self.reflections.reflection_light_buffers[k][1],
                    0,
                    bytemuck::cast_slice(&self.reflections.reflection_clusters[k].table),
                );
                self.queue.write_buffer(
                    &self.reflections.reflection_light_buffers[k][2],
                    0,
                    bytemuck::cast_slice(&self.reflections.reflection_clusters[k].ref_words()),
                );
            }
        }
        PrepareLightsOutputs {
            lights_gpu,
            local_shadows,
            shadowed_dropped,
            contact_rows,
            volumetric,
        }
    }
}
