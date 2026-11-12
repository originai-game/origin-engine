//! The custom post passes (post_custom.rs): a scene's own fullscreen
//! shaders over the lit image, reading the prepass's depth and normals.

use super::*;

pub(super) struct CustomPostPass;

/// A frame's custom post passes: the scene they draw, and which of its
/// posts compiled, in order.
pub(super) struct CustomPostPlan<'f> {
    pub scene: &'f RenderScene,
    pub posts: &'f [usize],
}

impl Pass for CustomPostPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::CustomPost(0),
        name: "custom-post",
        needs: Caps::NONE,
    };
    type State = post_custom::CustomPosts;
    type Frame = ();
    type PlanIn<'a> = ();
    type UploadIn<'a> = ();
    /// The scene and its custom posts that run (effects.rs).
    type BindIn<'f> = CustomPostPlan<'f>;
    type Plan<'f> = CustomPostPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.custom_posts
    }

    fn bind<'f>(
        _: &'f Renderer,
        _: &'f mut (),
        plan: CustomPostPlan<'f>,
    ) -> Option<CustomPostPlan<'f>> {
        (!plan.posts.is_empty()).then_some(plan)
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &CustomPostPlan<'f>,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::CustomPost(i) = pass else {
            unreachable!("a custom post pass")
        };
        let (r, slot) = (cx.ctx.r, cx.slot);
        let (read, write) = r.graph.custom_post_views(&r.persistent, i, slot);
        let lit = r.graph.custom_post_views(&r.persistent, 0, slot).0;
        cx.stats.draw_calls += st.encode(
            &r.device,
            &r.queue,
            cx.encoder,
            plan.scene,
            plan.posts[i as usize],
            [
                read,
                lit,
                r.graph.view("lindepth"),
                r.graph.view("gbuffer"),
                write,
            ],
            r.history_size(),
        );
    }
}
