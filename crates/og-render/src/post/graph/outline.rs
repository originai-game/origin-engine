//! The outlines: the thin one over the object ids, and the wide one's
//! jump flood (its seed, its jumps and the stroke over the field).

use super::*;

pub(super) fn groups(cx: &BindCx, pass: FramePass) -> Option<Pair> {
    let black = &cx.passes.black;
    Some(match pass {
        FramePass::Outline => cx.pair(|_| cx.fx(cx.view(cx.names.lit), cx.view("outline-id"))),
        FramePass::OutlineSeed => cx.pair(|_| cx.fx(black, cx.view("outline-id"))),
        // The flood's field in the depth's slot (unfilterable).
        FramePass::OutlineJump(i) => {
            let (read, _) = jfa_targets(i);
            cx.pair(|_| cx.fx_depth(black, cx.view(read), black))
        }
        FramePass::OutlineWide => {
            cx.pair(|_| cx.fx_depth(cx.view("hdr.outline"), cx.view("outline-jfa.a"), black))
        }
        _ => return None,
    })
}

pub(super) fn draw(pass: FramePass) -> Option<fx::Draw> {
    Some(match pass {
        FramePass::Outline => ("fs_outline", "hdr.outline"),
        FramePass::OutlineSeed => ("fs_outline_seed", "outline-jfa.a"),
        FramePass::OutlineJump(i) => (
            match JFA_STEPS[i as usize] {
                16 => "fs_jfa_16",
                8 => "fs_jfa_8",
                4 => "fs_jfa_4",
                2 => "fs_jfa_2",
                _ => "fs_jfa_1",
            },
            jfa_targets(i).1,
        ),
        FramePass::OutlineWide => ("fs_outline_wide", "hdr.wide"),
        _ => return None,
    })
}
