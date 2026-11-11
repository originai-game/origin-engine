//! The effects uniform: the motion rows, the decals' records and the main
//! view's clusters they reach, the AO and planar reflection rows, written
//! for the camera and then for each planar reflection.

use super::*;

pub(super) struct WriteFxUniformsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub view_frustum: &'a Frustum,
    pub prev_view_proj: Mat4,
    pub motion_read: bool,
    pub skinned_motion: Option<bool>,
    pub joints_drawn: u32,
    pub ao_target: bool,
    pub ao: Option<AmbientOcclusion>,
    pub gi: bool,
    pub bent: bool,
    pub reflection: Option<&'a camera::PlanarReflection>,
    pub reflections: &'a [camera::PlanarReflection],
}

pub(super) struct WriteFxUniformsOutputs {
    pub fx_uniform: fx::FxUniform,
    pub decals: u32,
    pub decals_dropped: u32,
}

impl Renderer {
    /// Builds and writes the effects uniform and the decals' records.
    #[allow(clippy::too_many_lines)]
    pub(super) fn write_fx_uniforms(
        &mut self,
        WriteFxUniformsInputs {
            scene,
            camera,
            cam,
            view_frustum,
            prev_view_proj,
            motion_read,
            skinned_motion,
            joints_drawn,
            ao_target,
            ao,
            gi,
            bent,
            reflection,
            reflections,
        }: WriteFxUniformsInputs<'_>,
    ) -> WriteFxUniformsOutputs {
        let mut fx_uniform =
            fx::fx_uniform(cam.view_proj, prev_view_proj, (self.width, self.height));
        // The prepass moves vertices by their material shader's offset of
        // the frame before only for a reader of the motion vectors.
        if !motion_read {
            fx_uniform.motion[0] = 0.0;
        }
        // The decals: their records, and the main view's clusters they reach
        // (none in a renderer made without them, RenderDesc::without).
        let (decals, decals_dropped) = self.lights.prepare_decals(
            (&self.device, &self.queue),
            if self.info.without.contains(LitFeatures::DECALS) {
                &[]
            } else {
                &scene.decals
            },
            &clusters::ClusterView {
                camera: cam,
                frustum: view_frustum,
                position: camera.position,
                near: camera.near,
                far: camera.far,
                width: self.width,
                height: self.height,
            },
            scene.decals.len() as u32,
        );
        // The previous joints' section (`skin::append_previous`).
        if skinned_motion.is_some() {
            fx_uniform.decal_info[3] = joints_drawn as f32;
        }
        fx_uniform.decal_info[0] = decals as f32;
        fx_uniform.decal_info[1] = match scene.decal_atlas {
            None => 0.0,
            Some(t) if scene.texture_color_space(t) == Some(ColorSpace::Srgb) => 2.0,
            Some(_) => 1.0,
        };
        self.terrain
            .decals
            .write(&self.queue, fx_uniform.decal_info);
        // w: 1 with the GI's bounce in the AO texture's gba, 2 with GTAO's
        // bent normal there.
        if ao_target {
            fx_uniform.ao = [
                1.0,
                1.0,
                ao.map_or(0.0, |a| a.direct),
                if gi {
                    1.0
                } else if bent {
                    2.0
                } else {
                    0.0
                },
            ];
        }
        // The planar reflection's plane (n, n·at) for the material shaders
        // that read it (pbr.wgsl `planar_reflection`); 0 without one.
        if let Some((i, ..)) = reflection {
            let w = &scene.water[*i];
            let (n, at, ..) = water::surface(w, w.mesh.and_then(|m| scene.mesh(m)));
            fx_uniform.planar = n.extend(n.dot(at)).to_array();
        }
        self.queue
            .write_buffer(&self.lights.fx_buffer, 0, bytemuck::bytes_of(&fx_uniform));
        for (k, (_, _, _, view_proj, (rw, rh))) in reflections.iter().enumerate() {
            let mut u = fx_uniform;
            u.ao = [0.0; 4];
            // Drawing the reflection itself: no plane, w -1 (a surface that
            // reads it may leave itself out, as three.js's readers of the
            // mirror do).
            u.planar = [0.0, 0.0, 0.0, -1.0];
            u.view_proj_unjittered = view_proj.to_cols_array_2d();
            u.prev_view_proj = view_proj.to_cols_array_2d();
            u.screen = [*rw as f32, *rh as f32, 1.0 / *rw as f32, 1.0 / *rh as f32];
            fx::unclustered(&mut u);
            self.queue
                .write_buffer(&self.reflection_fx_buffer[k], 0, bytemuck::bytes_of(&u));
        }
        WriteFxUniformsOutputs {
            fx_uniform,
            decals,
            decals_dropped,
        }
    }
}
