//! Motion's separable TileMax, NeighborMax and reconstruction. All draws
//! belong to the one motion-blur timed pass, including their GPU cost.

use super::*;

pub(super) struct Motion {
    groups: [Pair; 4],
}

impl Motion {
    pub fn bind(cx: &BindCx) -> Option<Self> {
        cx.has(FramePass::MotionBlur).then(|| Self {
            groups: [
                cx.pair(|_| cx.fx_after(&cx.passes.black, &cx.passes.black)),
                cx.pair(|_| cx.fx_after(cx.view("motion.x"), &cx.passes.black)),
                cx.pair(|_| cx.fx_after(cx.view("motion.tile"), &cx.passes.black)),
                cx.pair(|slot| {
                    cx.fx_after(
                        cx.stage(cx.names.after_dof, slot),
                        cx.view("motion.neighbor"),
                    )
                }),
            ],
        })
    }

    pub fn encode(&self, encoder: &mut crate::Encoder, cx: &PostCx) -> u32 {
        for ((entry, target), groups) in [
            ("fs_motion_tile_x", "motion.x"),
            ("fs_motion_tile_y", "motion.tile"),
            ("fs_motion_neighbor", "motion.neighbor"),
            ("fs_motion_blur", "hdr.mb"),
        ]
        .into_iter()
        .zip(&self.groups)
        {
            fullscreen_pass(
                encoder,
                entry,
                cx.graph.view(target),
                CLEAR,
                cx.passes.fx[entry].get(),
                &groups[cx.slot],
            );
        }
        4
    }
}
