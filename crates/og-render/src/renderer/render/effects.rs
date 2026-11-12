//! The frame's effects decisions: which screen-space effects run (AO, GI,
//! contact shadows, bent normals, outlines, depth of field, motion and
//! screen blur, god rays, lens flare, SSR, custom posts, depth bands), who
//! reads the motion vectors, and whether the depth prepass runs.

use super::*;

pub(super) struct DecideEffectsInputs<'a> {
    pub scene: &'a RenderScene,
    pub cam: &'a frame::CameraFrame,
    pub batches: &'a [Batch],
    pub skinned_batches: &'a [Batch],
    pub skinned: &'a SkinnedFrame,
    pub skinned_motion: Option<bool>,
    pub temporal: bool,
    pub taa_on: bool,
    pub volumetric_fog: &'a Option<crate::volumetric::VolumetricFog>,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub scene_water: &'a [Water],
}

pub(super) struct FrameEffects {
    pub ao: Option<AmbientOcclusion>,
    pub gi: Option<ScreenSpaceGlobalIllumination>,
    pub ao_target: bool,
    pub contact: bool,
    pub bent: bool,
    pub outline: bool,
    pub dof: Option<DepthOfField>,
    pub motion_blur: Option<MotionBlur>,
    pub screen_blur: Option<ScreenBlur>,
    pub god_rays: Option<GodRays>,
    pub lens_flare: Option<crate::LensFlare>,
    pub flare_sources_on: bool,
    pub fluids_3d: bool,
    pub depth_bands: Option<DepthBands>,
    pub ssr: Option<ssr::ScreenSpaceReflections>,
    pub custom_posts: Vec<usize>,
    pub motion_read: bool,
    pub ui_depth: bool,
    pub prepass: bool,
}

impl Renderer {
    /// Decides the frame's effects and its depth prepass.
    #[allow(clippy::too_many_lines)]
    pub(super) fn decide_effects(
        &mut self,
        DecideEffectsInputs {
            scene,
            cam,
            batches,
            skinned_batches,
            skinned,
            skinned_motion,
            temporal,
            taa_on,
            volumetric_fog,
            particle_frame,
            scene_water,
        }: DecideEffectsInputs<'_>,
    ) -> FrameEffects {
        let post_fx = &scene.look.post;
        // The quality's resolution, where it sets one, over the scene's.
        let ao = post_fx
            .ao
            .filter(|a| {
                scene.frame.quality.ambient_occlusion && a.intensity > 0.0 && a.radius > 0.0
            })
            .map(|a| AmbientOcclusion {
                resolution: scene.frame.quality.ao_resolution.unwrap_or(a.resolution),
                ..a
            });
        // Screen-space GI (ssgi.rs), in the AO passes' place: its targets
        // hold the AO too, at the GI's resolution when both are on.
        let gi = post_fx
            .gi
            .filter(|g| scene.frame.quality.global_illumination && g.intensity > 0.0)
            .map(|g| ScreenSpaceGlobalIllumination {
                resolution: scene.frame.quality.ao_resolution.unwrap_or(g.resolution),
                ..g
            });
        let ao = ao.map(|a| AmbientOcclusion {
            resolution: gi.map_or(a.resolution, |g| g.resolution),
            ..a
        });
        // The AO passes run (for the AO, the GI or both).
        let ao_target = ao.is_some() || gi.is_some();
        // Screen-space contact shadows for the shadowed point and spot
        // lights (contact.wgsl, `post/contact.rs`): they resolve the centimetres
        // where a caster meets its receiver, which a local shadow map's
        // texels (a few centimetres at a few metres) cannot. Their masks
        // share the AO texture (gba), which the GI's bounce fills instead.
        let contact = WORLD_3D
            && self.runs(FramePass::Contact)
            && gi.is_none()
            && scene
                .frame
                .quality
                .shadowed_lights
                .min(self.info.shadowed_lights)
                > 0
            && scene
                .lighting
                .lights
                .iter()
                .any(|l| l.cast_shadows && !matches!(l.kind, LightKind::Rect { .. }));
        // GTAO's bent normals for the probe GI (fxpost.wgsl `fs_gtao_bent`):
        // the AO targets hold them in gba, where the screen GI's bounce or
        // the contact shadows' masks would be, so either takes the place.
        let bent = WORLD_3D
            && gi.is_none()
            && !contact
            && ao.is_some_and(|a| a.method == AoMethod::Gtao)
            && post_fx.gi_probes.is_some()
            && scene.frame.quality.global_illumination
            && !self.info.without.contains(LitFeatures::GI);
        // The style's outline also lines what no batch draws: the terrain
        // and scattered copies (lighting.wgsl outline_bytes).
        let outline = post_fx.outline.is_some()
            || (WORLD_3D && scene.look.stylise.material.outline_width > 0.0)
            || batches
                .iter()
                .chain(skinned_batches)
                .any(|b| !b.transparent && self.cache.materials[b.material].outline);
        let dof = post_fx.depth_of_field;
        let motion_blur = post_fx.motion_blur;
        let screen_blur = post_fx.screen_blur.filter(|b| b.strength != 0.0);
        let god_rays = post_fx.god_rays;
        // Lens flare (post/flare.rs), after bloom. The sun's and the
        // lights' own flares, seen by a perspective 3D camera, read the
        // prepass depth for their occlusion.
        let lens_flare = post_fx.lens_flare.filter(|f| f.intensity > 0.0);
        let flare_sources_on = WORLD_3D && !cam.orthographic && lens_flare.is_some();
        let fluids_3d = WORLD_3D
            && scene
                .world
                .fluids_3d
                .iter()
                .any(|f| !f.particles.is_empty() || f.visual.is_some());
        let depth_bands = scene.look.stylise.depth_bands.filter(|b| b.count >= 2);
        // Ink or hatching on depth edges reads the prepass in the lines
        // pass, a canvas, hatching or halftone on surfaces in the paint or
        // lines pass.
        let ink_depth = scene.look.stylise.depth_edges();
        // Screen-space reflections, traced through the prepass before the
        // main pass (ssr.rs).
        let ssr = self.frame_ssr(scene);
        // Custom post passes (post_custom.rs) that compiled; they read the
        // prepass's depth and normals.
        let custom_posts = self.custom_posts.prepare(&self.device, scene);
        // Whether anything reads the prepass's motion vectors this frame:
        // TAA (and the temporal upscalers), motion blur, the reflections'
        // temporal pass (ssr.wgsl `ssr_velocity`), screen-space GI's
        // reprojection (ssgi.wgsl) and a custom post pass, which binds the
        // whole prepass target.
        let motion_read = temporal
            || motion_blur.is_some()
            || ssr.is_some()
            || gi.is_some()
            || !custom_posts.is_empty();
        let ui_depth = scene
            .ui
            .overlay
            .as_ref()
            .is_some_and(|o| o.ui.needs_depth());
        let prepass = !custom_posts.is_empty()
            || ao_target
            || contact
            || ssr.is_some()
            // Volumetric fog reads each pixel's depth.
            || volumetric_fog.is_some()
            || depth_bands.is_some()
            || ink_depth
            || scene.look.stylise.on_surfaces()
            // The painterly filter's object edges and detail, and the
            // Ben-Day dots' facing (halftone_faces), where meshes
            // are drawn (a 2D scene's prepass would hold nothing).
            || (WORLD_3D
                && (scene.look.stylise.paint.is_some_and(|p| p.detail_aware())
                    // The watercolour wash's outer boundaries (object
                    // edges, silhouettes, creases), where its rim pools
                    // and its dry brush skips.
                    || scene.look.stylise.paint.is_some_and(|p| p.watercolor.edges())
                    || scene.look.stylise.halftone_faces())
                && !(batches.is_empty() && skinned_batches.is_empty()))
            || outline
            || dof.is_some()
            || motion_blur.is_some()
            || god_rays.is_some()
            || flare_sources_on
            || particle_frame.needs_depth
            // Water and 3D fluids read the depth below them.
            || !scene_water.is_empty()
            || fluids_3d
            // UI occluders and mass elements the scene hides (§14).
            || ui_depth
            // TAA's motion vectors of what moved besides the camera (GPU
            // culling and the meshlet path carry the previous transforms
            // too: cull_gpu.rs `previous`).
            || (taa_on
                && (skinned_motion == Some(true) || batches.iter().any(|b| b.simulated) || !skinned.garments.is_empty() || skinned.baked_moved || objects_moved(scene) || crate::debris::moving(scene)));
        FrameEffects {
            ao,
            gi,
            ao_target,
            contact,
            bent,
            outline,
            dof,
            motion_blur,
            screen_blur,
            god_rays,
            lens_flare,
            flare_sources_on,
            fluids_3d,
            depth_bands,
            ssr,
            custom_posts,
            motion_read,
            ui_depth,
            prepass,
        }
    }
}
