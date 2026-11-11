//! The 2D fluids' field pass (metaballs.rs): each fluid's particles
//! splatted into its density field, which its sprites composite.

use super::*;

pub(super) struct MetaballsPass;

impl Pass for MetaballsPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Metaballs,
        name: "metaballs",
        needs: Caps::NONE,
    };
    type State = metaballs::MetaballRenderer;
    type Plan<'f> = ();

    fn state(r: &Renderer) -> &Self::State {
        &r.world.metaballs
    }

    fn encode<'f>(st: &Self::State, _: &(), _: FramePass, cx: &mut EncodeCx<'_, 'f>) {
        let r = cx.ctx.r;
        let fields: Vec<&wgpu::TextureView> = post::FIELD_NAMES
            .iter()
            .map_while(|name| r.graph.try_view(name))
            .collect();
        st.encode_field(cx.encoder, &fields, cx.frame_bind_group);
        cx.stats.draw_calls += fields.len() as u32;
    }
}
