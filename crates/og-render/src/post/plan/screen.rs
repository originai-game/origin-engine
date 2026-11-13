//! The effects over the lit image, each reading what the one before left:
//! the depth bands and the outlines, TAA and its sharpening, the depth of
//! field, the motion blur, the screen blur, the custom passes, and the god
//! rays the tonemap adds.

use super::*;

pub(super) fn declare(g: &mut RenderGraph, cx: &PlanCx, p: FramePass) {
    let cfg = cx.cfg;
    let names = &cx.names;
    let prepass = cx.prepass;
    let (aa, after_dof, lit) = (cx.aa, cx.after_dof, cx.lit);
    let name = p.name();
    match p {
        FramePass::DepthBands => g.pass(name, &["hdr", "lindepth"], &["hdr.bands"]),
        FramePass::Outline => g.pass(
            name,
            &[names.lit, "gbuffer", "lindepth", "outline-id"],
            &["hdr.outline"],
        ),
        FramePass::OutlineSeed => g.pass(
            name,
            &["lindepth", "gbuffer", "outline-id"],
            &["outline-jfa.a"],
        ),
        FramePass::OutlineJump(i) => {
            let (read, write) = jfa_targets(i);
            g.pass(name, &[read, "gbuffer"], &[write])
        }
        FramePass::OutlineWide => g.pass(
            name,
            &["hdr.outline", "outline-jfa.a", "gbuffer"],
            &["hdr.wide"],
        ),
        // With a prepass TAA binds its velocity and view depth.
        // The OIT revealage is its reactive mask (taa.wgsl).
        FramePass::Taa => {
            let mut reads = vec![names.main, "depth", "history-read"];
            if prepass {
                reads.extend(["gbuffer", "lindepth"]);
            }
            if cfg.oit.is_some() {
                reads.push("oit.reveal");
            }
            g.pass(name, &reads, &["history-write"])
        }
        FramePass::TaaSharpen if prepass => g.pass(
            name,
            &["history-write", "depth", "gbuffer", "lindepth"],
            &["hdr.sharp"],
        ),
        FramePass::TaaSharpen => g.pass(name, &["history-write", "depth"], &["hdr.sharp"]),
        // Every effect pass binds the prepass's lindepth and gbuffer
        // (`fx_group`), so each declares them: a target written here may
        // not alias a texture the pass still has bound.
        FramePass::DofCoc => g.pass(name, &[aa, "lindepth", "gbuffer"], &["dof.coc"]),
        FramePass::DofBlur => g.pass(name, &["dof.coc", "lindepth", "gbuffer"], &["dof.blur"]),
        FramePass::DofComposite => {
            g.pass(name, &[aa, "dof.blur", "lindepth", "gbuffer"], &["hdr.dof"])
        }
        FramePass::MotionBlur => g.pass(name, &[after_dof, "gbuffer", "lindepth"], &["hdr.mb"]),
        // The prepass's targets are bound when the frame has them
        // (`fx_group`); the screen blur alone does not ask for one.
        FramePass::ScreenBlur => {
            let after_mb = names.after_mb.unwrap_or("history-write");
            if prepass {
                g.pass(name, &[after_mb, "gbuffer", "lindepth"], &["hdr.blur"])
            } else {
                g.pass(name, &[after_mb], &["hdr.blur"])
            }
        }
        FramePass::CustomPost(i) => {
            let read = match i {
                0 => names.before_custom.unwrap_or("history-write"),
                i => CUSTOM_POST_NAMES[i as usize - 1],
            };
            g.pass(
                name,
                &[read, "lindepth", "gbuffer"],
                &[CUSTOM_POST_NAMES[i as usize]],
            )
        }
        FramePass::GodRaysMask => g.pass(name, &[lit, "lindepth", "gbuffer"], &["god.mask"]),
        FramePass::GodRaysBlur => g.pass(name, &["god.mask", "lindepth", "gbuffer"], &["god.blur"]),
        _ => unreachable!("{p:?} is not a screen effect"),
    };
}
