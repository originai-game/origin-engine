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
    pub skinned_section: u32,
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
            skinned_section,
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
        // The decals: their textures in the atlas, their records, and the
        // main view's clusters they reach (none in a renderer made without
        // them, RenderDesc::without).
        let drawn: &[Decal] = if self.info.without.contains(LitFeatures::DECALS) {
            &[]
        } else {
            &scene.effects.decals
        };
        self.sync_decal_atlas(scene, drawn);
        let (decals, decals_dropped) = self.lights.prepare_decals(
            (&self.device, &self.queue),
            drawn,
            &clusters::ClusterView {
                camera: cam,
                frustum: view_frustum,
                position: camera.position,
                near: camera.near,
                far: camera.far,
                width: self.width,
                height: self.height,
            },
            scene.effects.decals.len() as u32,
        );
        // The previous joints' section (`skin::append_previous`).
        if skinned_motion.is_some() {
            fx_uniform.decal_info[3] = skinned_section as f32;
        }
        fx_uniform.decal_info[0] = decals as f32;
        fx_uniform.decal_info[1] = f32::from(self.lights.decal_atlas.is_some());
        self.world
            .terrain_fx
            .write(&self.queue, &fx_uniform, self.world.trample.grid());
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
            let w = &scene.world.water[*i];
            let (n, at, ..) = water::surface(w, w.mesh.and_then(|m| scene.mesh(m)));
            fx_uniform.planar = n.extend(n.dot(at)).to_array();
        }
        crate::traffic::write_buffer(
            &self.queue,
            &self.lights.fx_buffer,
            0,
            bytemuck::bytes_of(&fx_uniform),
        );
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
            crate::traffic::write_buffer(
                &self.queue,
                &self.reflections.reflection_fx_buffer[k],
                0,
                bytemuck::bytes_of(&u),
            );
        }
        WriteFxUniformsOutputs {
            fx_uniform,
            decals,
            decals_dropped,
        }
    }

    /// Brings the decals' atlas in line with the textures `decals` name:
    /// each given its cells, and copied in when it is new there or its
    /// texels changed. A handle that names no texture of the scene draws
    /// its decal without one.
    fn sync_decal_atlas(&mut self, scene: &RenderScene, decals: &[Decal]) {
        use crate::decal_atlas::{Blit, Copy, DecalAtlas, DecalCopies, Key, Wanted};
        let mut wanted: Vec<Wanted> = Vec::new();
        let mut copies: Vec<(Key, Copy)> = Vec::new();
        for d in decals.iter().take(crate::effects::MAX_DECALS) {
            for (handle, normal) in [(d.texture, false), (d.normal_map, true)] {
                let Some(handle) = handle.filter(|&h| scene.contains_texture(h)) else {
                    continue;
                };
                let key = Key {
                    index: handle.index(),
                    normal,
                };
                let Some(gpu) = self.cache.textures.get(key.index) else {
                    continue;
                };
                if wanted.iter().any(|w| w.key == key) {
                    continue;
                }
                // What the copy can filter (not a depth target).
                let texture = gpu.view.texture();
                if !matches!(
                    texture
                        .format()
                        .sample_type(None, Some(self.device.features())),
                    Some(wgpu::TextureSampleType::Float { filterable: true })
                ) {
                    continue;
                }
                let size = texture.size();
                wanted.push(Wanted {
                    key,
                    size: (size.width, size.height),
                    revision: self.cache.texture_revisions[key.index],
                    live: gpu.target,
                });
                copies.push((
                    key,
                    if !normal {
                        Copy::Color
                    } else if scene.texture_color_space(handle) == Some(ColorSpace::Srgb) {
                        Copy::NormalSrgb
                    } else if gpu.format.two_channel() {
                        Copy::NormalRg
                    } else {
                        Copy::Normal
                    },
                ));
            }
        }
        if wanted.is_empty() {
            return;
        }
        let atlas = self
            .lights
            .decal_atlas
            .get_or_insert_with(|| DecalAtlas::new(&self.device));
        let stale = atlas.place(&self.device, &wanted);
        if stale.is_empty() {
            return;
        }
        let blits: Vec<Blit> = copies
            .iter()
            .filter(|(key, _)| stale.contains(key))
            .map(|&(key, copy)| Blit {
                key,
                source: &self.cache.textures[key.index].view,
                copy,
            })
            .collect();
        let copies = self
            .lights
            .decal_copies
            .get_or_insert_with(|| DecalCopies::new(&self.device));
        let mut encoder = Encoder::untimed(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("decal atlas"),
            },
        ));
        atlas.copy(&self.device, copies, &mut self.mips, &blits, &mut encoder);
        self.queue.submit(Some(encoder.finish()));
    }
}
