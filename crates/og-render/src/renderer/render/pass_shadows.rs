//! The shadow passes: the sun's cascades, the weather's shelter map in the
//! cascade array's last layer, and the shadowed local lights' faces, each
//! drawing its casters into its layer of the shadow arrays.

use super::*;

/// A frame's cascades: each one's view, casters, cache plan and meshlet
/// view, by cascade.
pub(super) struct CascadePlan<'f> {
    pub cascades: &'f [Cascade],
    pub lists: &'f [[DrawList; 2]],
    pub plans: &'f [crate::shadow_cache::Plan],
    pub views: &'f [u32],
}

pub(super) struct CascadePass;

impl Pass for CascadePass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Cascade(0),
        name: "cascade",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Plan<'f> = CascadePlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.cascade_map
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &CascadePlan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::Cascade(i) = pass else {
            unreachable!("a cascade pass")
        };
        cx.ctx.shadow_pass(
            cx.encoder,
            &st.layers[i as usize],
            &st.depth,
            (&st.texture, i, &st.depth_texture),
            i as usize,
            &plan.lists[i as usize],
            plan.plans[i as usize],
            &plan.cascades[i as usize].frustum,
            plan.views.get(i as usize).copied().unwrap_or(0),
            cx.stats,
            true,
            &[ShadowTarget::Cascade],
        );
    }
}
