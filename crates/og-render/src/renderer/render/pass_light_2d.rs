//! The 2D lighting's pass (light2d.rs): the frame's 2D lights and their
//! shadows into the light map's strips, and their directions where the
//! sprites are shaded by normal maps.

use super::*;

pub(super) struct Light2dPass;

/// The 2D light map of a frame.
#[derive(Default)]
pub(super) struct Light2dFrame {
    /// Its size, its strips side by side; the graph's `lightmap`.
    pub size: (u32, u32),
    /// The light directions' target beside it (`lightdir`).
    pub lightdir: bool,
    /// The lights and occluders written for it.
    pub lights: u32,
    pub occluders: u32,
}

/// A frame's 2D lighting.
pub(super) struct Light2dPlan {
    pub ambient: Vec3,
    /// The lights drawn into the map.
    pub lights: u32,
    /// The light map's size, its strips side by side.
    pub size: (u32, u32),
    /// The light directions' target beside the map (sprite_normals.rs).
    pub lightdir: bool,
}

impl Pass for Light2dPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Light2d,
        name: "light-2d",
        needs: Caps::NONE,
    };
    type State = light2d::Light2dRenderer;
    type Frame = Light2dFrame;
    /// The scene and the light map's strips (sprite_arenas.rs).
    type PlanIn<'a> = (&'a RenderScene, &'a crate::light2d::LightSlots);
    /// The scene, the camera and the light map's strips.
    type UploadIn<'a> = (
        &'a RenderScene,
        &'a frame::CameraFrame,
        &'a crate::light2d::LightSlots,
    );
    type BindIn<'f> = &'f RenderScene;
    type Plan<'f> = Light2dPlan;

    fn state(r: &Renderer) -> &Self::State {
        &r.lights_2d
    }

    /// The 2D light map's frame: its strips' size (made even without 2D
    /// lighting, at half the frame), and whether the lights' directions
    /// get a second target beside it (never without MRT: normal-mapped
    /// receivers are lit as flat ones).
    fn plan(r: &mut Renderer, (scene, light_slots): Self::PlanIn<'_>) -> Light2dFrame {
        let lightmap_scale = scene
            .sprite
            .lighting_2d
            .map_or(0.5, |l| l.resolution.clamp(0.25, 1.0));
        // The lights' directions beside the light map, for normal-mapped
        // receivers (sprite_normals.rs).
        // A second target beside the light map (without MRT, normal-mapped
        // receivers are lit as flat ones).
        let lightdir_on = scene.sprite.lighting_2d.is_some()
            && scene.has_texture_normals()
            && r.info.caps.contains(Caps::MRT);
        // The light map: its strips side by side (light2d.rs).
        let lightmap_size = light2d::strips_size(
            (
                ((r.width as f32 * lightmap_scale) as u32).max(1),
                ((r.height as f32 * lightmap_scale) as u32).max(1),
            ),
            light_slots.count(),
            r.device.limits().max_texture_dimension_2d,
        );
        Light2dFrame {
            size: lightmap_size,
            lightdir: lightdir_on,
            lights: 0,
            occluders: 0,
        }
    }

    /// The 2D lights and occluders, as the camera sees them.
    fn upload(
        r: &mut Renderer,
        frame: &mut Light2dFrame,
        (scene, cam, light_slots): Self::UploadIn<'_>,
    ) {
        (frame.lights, frame.occluders) = match scene.sprite.lighting_2d {
            Some(l) => r.lights_2d.write(
                &r.device,
                &r.queue,
                cam.view_proj,
                &scene.sprite.lights_2d,
                &scene.sprite.occluders_2d,
                light_slots,
                l.shadow_angles,
            ),
            None => (0, 0),
        };
    }

    /// The light pass runs with the scene's 2D lighting.
    fn bind<'f>(
        _: &'f Renderer,
        frame: &'f mut Light2dFrame,
        scene: &'f RenderScene,
    ) -> Option<Light2dPlan> {
        scene.sprite.lighting_2d.map(|l| Light2dPlan {
            ambient: l.ambient,
            lights: frame.lights,
            size: frame.size,
            lightdir: frame.lightdir,
        })
    }

    /// The light directions are a second colour target.
    fn needs(plan: &Light2dPlan) -> Caps {
        if plan.lightdir { Caps::MRT } else { Caps::NONE }
    }

    fn encode<'f>(st: &Self::State, plan: &Light2dPlan, _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        st.encode(
            cx.encoder,
            r.graph.view("lightmap"),
            r.graph.try_view("lightdir"),
            plan.size,
            plan.ambient,
        );
        cx.stats.draw_calls += (plan.lights > 0) as u32;
    }
}
