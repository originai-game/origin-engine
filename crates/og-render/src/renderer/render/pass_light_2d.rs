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
