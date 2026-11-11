//! The 2D lighting's pass (light2d.rs): the frame's 2D lights and their
//! shadows into the light map's strips, and their directions where the
//! sprites are shaded by normal maps.

use super::*;

pub(super) struct Light2dPass;

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
    type Plan<'f> = Light2dPlan;

    fn state(r: &Renderer) -> &Self::State {
        &r.lights_2d
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

impl Renderer {
    /// The 2D light map's frame: its strips' size (made even without 2D
    /// lighting, at half the frame), whether the lights' directions get a
    /// second target beside it (never without MRT: normal-mapped receivers
    /// are lit as flat ones), and the light pass's plan.
    pub(super) fn plan_light_2d(
        &self,
        scene: &RenderScene,
        light_slots: &crate::light2d::LightSlots,
        lights_2d: u32,
    ) -> (Option<Light2dPlan>, (u32, u32), bool) {
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
            && self.info.caps.contains(Caps::MRT);
        // The light map: its strips side by side (light2d.rs).
        let lightmap_size = light2d::strips_size(
            (
                ((self.width as f32 * lightmap_scale) as u32).max(1),
                ((self.height as f32 * lightmap_scale) as u32).max(1),
            ),
            light_slots.count(),
            self.device.limits().max_texture_dimension_2d,
        );
        let plan = scene.sprite.lighting_2d.map(|l| Light2dPlan {
            ambient: l.ambient,
            lights: lights_2d,
            size: lightmap_size,
            lightdir: lightdir_on,
        });
        (plan, lightmap_size, lightdir_on)
    }
}
