//! Post uniforms stage.

use super::*;
use crate::ssgi::GiOcclusion;

pub(super) struct WriteFramePostUniformsInputs<'a> {
    pub(super) cam: &'a frame::CameraFrame,
    pub(super) camera: &'a Camera,
    pub(super) post_fx: &'a PostEffects,
    pub(super) scene: &'a RenderScene,
    pub(super) ao: Option<AmbientOcclusion>,
    pub(super) gi: Option<ScreenSpaceGlobalIllumination>,
    pub(super) ssr: Option<&'a ssr::ScreenSpaceReflections>,
    /// TAA (or a temporal upscaler) resolves the frame.
    pub(super) taa: bool,
    /// The frame keeps a temporal history (TAA's uniform).
    pub(super) temporal: bool,
    /// The depth prepass writes velocity (TAA's uniform).
    pub(super) prepass: bool,
    pub(super) dof: Option<DepthOfField>,
    pub(super) motion_blur: Option<MotionBlur>,
    pub(super) god_rays: Option<GodRays>,
    pub(super) sun_ndc: Vec2,
    pub(super) god_visible: bool,
    pub(super) sun_fade: f32,
    pub(super) depth_bands: Option<DepthBands>,
    pub(super) screen_blur: Option<ScreenBlur>,
    /// Contact shadows' rows (`FxPostUniform::contact`).
    pub(super) contact: [[f32; 4]; 4],
}

/// What the post uniforms decided about the frame's history.
pub(super) struct FramePostHistory {
    /// The GI and SSR read their own copy of the main pass's image.
    pub(super) gi_copy: bool,
    /// TAA blends with a history.
    pub(super) taa_blend: bool,
}

impl Renderer {
    /// The post chain's frame uniforms: the GI's history, the effects'
    /// post rows, SSR's and TAA's uniforms.
    pub(super) fn write_frame_post_uniforms(
        &mut self,
        WriteFramePostUniformsInputs {
            cam,
            camera,
            post_fx,
            scene,
            ao,
            gi,
            dof,
            motion_blur,
            god_rays,
            sun_ndc,
            god_visible,
            sun_fade,
            depth_bands,
            screen_blur,
            contact,
            ssr,
            taa,
            temporal,
            prepass,
        }: WriteFramePostUniformsInputs<'_>,
    ) -> FramePostHistory {
        // Screen-space GI reads the previous frame's lit image: TAA's
        // history, or without TAA its own copy of the main pass's image.
        let gi_copy = (gi.is_some() || ssr.is_some()) && !taa;
        if gi_copy
            && self
                .persistent
                .ensure_gi(&self.device, self.width, self.height)
        {
            self.gi_history = None;
        }
        let gi_history = if taa {
            self.history.taa_history.is_some()
        } else {
            gi_copy && self.gi_history.is_some_and(|f| f + 1 == self.frame_index)
        };
        let proj = cam.view_proj * cam.view.inverse();
        let global_outline = post_fx.outline;
        let fxp = post::FxPostUniform {
            proj: proj.to_cols_array_2d(),
            inv_proj: proj.inverse().to_cols_array_2d(),
            view: cam.view.to_cols_array_2d(),
            screen: [
                self.width as f32,
                self.height as f32,
                1.0 / self.width as f32,
                1.0 / self.height as f32,
            ],
            // w: the AO debug views (DebugView::ao_code): 1 the AO target holds
            // the view depth's fraction, 2 the term unblurred.
            cam: [
                camera.far,
                cam.orthographic as u32 as f32,
                camera.near,
                scene.debug.ao_code(),
            ],
            // z: the AO targets' divisor of the screen (2 at half resolution).
            ao: {
                let divisor = match gi.map(|g| g.resolution).or(ao.map(|a| a.resolution)) {
                    Some(AoResolution::Half) => 2.0,
                    _ => 1.0,
                };
                ao.map_or([0.0, 0.0, divisor, 0.0], |a| {
                    [a.radius, a.intensity, divisor, 0.0]
                })
            },
            // x: radius per metre of depth (0: fixed), y: 1 for the screen
            // method, z: the scale GTAO's raw term is stored over.
            ao2: ao.map_or([0.0; 4], |a| {
                [
                    a.radius_scale,
                    if a.method == AoMethod::Screen {
                        1.0
                    } else {
                        0.0
                    },
                    // GTAO's raw term is stored over this (XeGTAO's
                    // XE_GTAO_OCCLUSION_TERM_SCALE, fxpost.wgsl fs_gtao).
                    if matches!(a.method, AoMethod::Ssao | AoMethod::Screen) {
                        1.0
                    } else {
                        1.5
                    },
                    0.0,
                ]
            }),
            outline_color: global_outline.map_or([0.0; 4], |o| o.color),
            // w: where silhouette lines lie (MaterialStyle::outline_place).
            outline: {
                let place = scene.stylise.material.outline_place.code();
                match global_outline {
                    Some(o) => [
                        o.width.max(0.0),
                        o.depth_threshold,
                        o.normal_threshold,
                        place,
                    ],
                    None => [0.0, 0.04, 0.4, place],
                }
            },
            dof: dof.map_or([0.0; 4], |d| {
                [
                    d.focus_distance,
                    d.focus_range.max(1e-3),
                    (d.max_blur * self.height as f32).max(0.0),
                    d.falloff.max(1e-3),
                ]
            }),
            dof2: [
                match dof.map(|d| d.quality) {
                    Some(DofQuality::High) => 48.0,
                    _ => 16.0,
                },
                0.0,
                0.0,
                0.0,
            ],
            mb: motion_blur.map_or([0.0; 4], |m| {
                [
                    m.shutter.max(0.0),
                    m.samples.clamp(4, 32) as f32,
                    m.max_pixels.max(1.0),
                    0.0,
                ]
            }),
            god: [
                sun_ndc.x * 0.5 + 0.5,
                0.5 - sun_ndc.y * 0.5,
                god_rays.map_or(0.96, |g| g.decay),
                god_rays.map_or(0.8, |g| g.density),
            ],
            god2: [
                god_rays.map_or(48.0, |g| g.samples.clamp(8, 128) as f32),
                god_visible as u32 as f32,
                god_rays.map_or(0.0, |g| g.intensity * sun_fade),
                god_rays.map_or(2.0, |g| g.threshold.max(0.0)),
            ],
            god_tint: god_rays
                .map_or(Vec3::ONE, |g| g.tint)
                .extend(1.0)
                .to_array(),
            bands: depth_bands.map_or([0.0; 4], |b| {
                [
                    b.count.clamp(2, 8) as f32,
                    b.near.max(0.0),
                    b.far.max(b.near + 0.01),
                    b.flatten.clamp(0.0, 1.0),
                ]
            }),
            bands_near: depth_bands.map_or([0.0; 4], |b| {
                [b.near_color[0], b.near_color[1], b.near_color[2], 1.0]
            }),
            bands_far: depth_bands.map_or([0.0; 4], |b| {
                [b.far_color[0], b.far_color[1], b.far_color[2], 1.0]
            }),
            blur: screen_blur.map_or([[0.0; 4]; 2], |b| {
                b.rows(self.width as f32, self.height as f32)
            }),
            // The noise moves per frame only under TAA, which averages it;
            // without TAA a still frame keeps one pattern.
            gi: ScreenSpaceGlobalIllumination::rows(
                gi.as_ref(),
                if self.persistent.history.is_some() {
                    self.phase()
                } else {
                    0
                },
                gi_history,
                match ao.map(|a| a.method) {
                    None => GiOcclusion::Rays,
                    Some(AoMethod::Ssao | AoMethod::Screen) => GiOcclusion::Ssao,
                    Some(_) => GiOcclusion::Gtao,
                },
            ),
            contact,
        };
        self.queue
            .write_buffer(&self.post.fx_buffer, 0, bytemuck::bytes_of(&fxp));
        // The passes after TAA see the frame it resolved: the output's
        // size under TAAU (`FramePass::after_taa`), the scene's otherwise.
        let (pw, ph) = self.history_size();
        let mut after = fxp;
        after.screen = [pw as f32, ph as f32, 1.0 / pw as f32, 1.0 / ph as f32];
        if let Some(d) = dof {
            after.dof[2] = (d.max_blur * ph as f32).max(0.0);
        }
        if let Some(b) = screen_blur {
            after.blur = b.rows(pw as f32, ph as f32);
        }
        self.queue
            .write_buffer(&self.post.fx_after_buffer, 0, bytemuck::bytes_of(&after));
        let view_t = cam.view.transpose();
        let pview = particles_gpu::ParticleViewGpu {
            view_proj: cam.view_proj.to_cols_array_2d(),
            cam_pos: camera
                .position
                .extend(cam.orthographic as u32 as f32)
                .to_array(),
            cam_forward: cam.forward.extend(0.0).to_array(),
            cam_right: view_t.x_axis.truncate().extend(0.0).to_array(),
            cam_up: view_t.y_axis.truncate().extend(0.0).to_array(),
            screen: fxp.screen,
        };
        self.queue.write_buffer(
            &self.particles.gpu.view_buffer,
            0,
            bytemuck::bytes_of(&pview),
        );
        self.queue
            .write_buffer(&self.sprite_fx_buffer, 0, bytemuck::bytes_of(&fxp.screen));
        if let Some(s) = ssr {
            self.prepare_ssr(s, cam, camera, gi_history);
        }
        let taa_blend = taa && self.history.taa_history.is_some();
        self.write_frame_taa_uniform(temporal, prepass, cam);
        FramePostHistory { gi_copy, taa_blend }
    }

    /// `velocity`: the prepass's velocity is bound (object and skinned
    /// motion).
    pub(super) fn write_frame_taa_uniform(
        &self,
        taa: bool,
        velocity: bool,
        cam: &frame::CameraFrame,
    ) {
        if taa {
            let prev = self.history.taa_history.unwrap_or(cam.view_proj);
            let jitter = self.taa_jitter();
            let (hw, hh) = self.history_size();
            let taa_uniform = post::TaaUniform {
                inv_view_proj: cam.jittered_view_proj.inverse().to_cols_array_2d(),
                cur_view_proj: cam.view_proj.to_cols_array_2d(),
                prev_view_proj: prev.to_cols_array_2d(),
                params: [
                    post::taa_weight(self.history.taa_history.is_some()),
                    self.width as f32,
                    self.height as f32,
                    self.post.taa_sharpen,
                ],
                still: [
                    still_camera(&prev, &cam.view_proj) as u32 as f32,
                    velocity as u32 as f32,
                    jitter.x,
                    jitter.y,
                ],
                upscale: [
                    hw as f32,
                    hh as f32,
                    self.taau as u32 as f32,
                    if self.info.antialias == Antialias::SmaaT2x {
                        frame::smaa_sample(self.phase()).1
                    } else {
                        0.0
                    },
                ],
            };
            self.queue
                .write_buffer(&self.post.taa_buffer, 0, bytemuck::bytes_of(&taa_uniform));
        }
    }
}
