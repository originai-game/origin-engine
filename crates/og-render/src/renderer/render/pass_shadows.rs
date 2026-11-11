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
        &r.shadows.cascade_map
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

/// A frame's shelter map: its view, frustum and casters, when redrawn.
pub(super) struct ShelterPlan<'f> {
    pub frustum: &'f Frustum,
    pub list: &'f [DrawList; 2],
    pub view: u32,
}

pub(super) struct ShelterPass;

impl Pass for ShelterPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Shelter,
        name: "shelter",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Plan<'f> = ShelterPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.shadows.cascade_map
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &ShelterPlan<'f>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        cx.ctx.shadow_pass(
            cx.encoder,
            st.layers.last().expect("fit_shelter_layer"),
            &st.depth,
            (&st.texture, st.layers.len() as u32 - 1, &st.depth_texture),
            SHELTER_SLOT,
            plan.list,
            crate::shadow_cache::Plan::ALL,
            plan.frustum,
            plan.view,
            cx.stats,
            false,
            &[ShadowTarget::Cascade],
        );
    }
}

/// A frame's shadowed local lights' faces: each layer's casters, cache
/// plan, frustum and meshlet view, by layer.
pub(super) struct LocalFacePlan<'f> {
    pub lists: &'f [Option<[DrawList; 2]>],
    pub plans: &'f [crate::shadow_cache::Plan],
    pub frustums: &'f [Option<Frustum>],
    pub views: &'f [u32],
}

pub(super) struct LocalFacePass;

impl Pass for LocalFacePass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::LocalFace(0),
        name: "local-shadows",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Plan<'f> = LocalFacePlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.shadows.local_map
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &LocalFacePlan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::LocalFace(layer) = pass else {
            unreachable!("a local face pass")
        };
        let l = layer as usize;
        cx.ctx.shadow_pass(
            cx.encoder,
            &st.layers[l],
            &st.depth,
            (&st.texture, layer, &st.depth_texture),
            CASCADES + l,
            plan.lists[l].as_ref().expect("a list per shadowed face"),
            plan.plans[l],
            plan.frustums[l]
                .as_ref()
                .expect("a frustum per shadowed face"),
            plan.views[l],
            cx.stats,
            true,
            &ShadowTarget::LOCAL,
        );
    }
}
