//! The names of the graph's targets, and of the lit image as the effects hand it on.

use super::*;

/// The target the AO passes end in: `ao`, or with contact shadows
/// `ao.pre`, which the contact pass reads.
pub(in crate::post) fn ao_target_name(contact: bool) -> &'static str {
    if contact { "ao.pre" } else { "ao" }
}

/// The metaball field targets, by field.
pub(crate) const FIELD_NAMES: [&str; crate::FIELDS] =
    ["metaballs", "metaballs.1", "metaballs.2", "metaballs.3"];

pub(in crate::post) fn luminance_name(i: u32) -> String {
    format!("luminance.{i}")
}

pub(in crate::post) fn bloom_name(i: u32) -> String {
    format!("bloom.{i}")
}

/// The names of the lit image as the effects hand it on: after the main
/// pass (and outlines), after TAA (`None`: the history; its sharpening:
/// `hdr.sharp`), after depth of field and motion blur.
pub(in crate::post) struct LitNames {
    /// The lit image after the main pass and the depth bands.
    pub(in crate::post) lit: &'static str,
    pub(in crate::post) main: &'static str,
    /// `None` when TAA writes the history and nothing sharpens it.
    pub(in crate::post) aa: Option<&'static str>,
    pub(in crate::post) after_dof: Option<&'static str>,
    /// The lit image after motion blur (the screen blur reads it).
    pub(in crate::post) after_mb: Option<&'static str>,
    /// The lit image the first custom post pass reads (after motion blur
    /// and the screen blur).
    pub(in crate::post) before_custom: Option<&'static str>,
    pub(in crate::post) last: Option<&'static str>,
}

/// The targets the custom post passes write, in order (`post_custom.rs`).
pub(crate) const CUSTOM_POST_NAMES: [&str; crate::post_custom::MAX_CUSTOM_POSTS] = [
    "hdr.post.0",
    "hdr.post.1",
    "hdr.post.2",
    "hdr.post.3",
    "hdr.post.4",
    "hdr.post.5",
    "hdr.post.6",
    "hdr.post.7",
];

pub(in crate::post) fn lit_names(passes: &[FramePass]) -> LitNames {
    let has = |p| passes.contains(&p);
    let lit = if has(FramePass::DepthBands) {
        "hdr.bands"
    } else {
        "hdr"
    };
    let main = if has(FramePass::OutlineWide) {
        "hdr.wide"
    } else if has(FramePass::Outline) {
        "hdr.outline"
    } else {
        lit
    };
    let aa = if has(FramePass::TaaSharpen) {
        Some("hdr.sharp")
    } else {
        (!has(FramePass::Taa)).then_some(main)
    };
    let after_dof = if has(FramePass::DofComposite) {
        Some("hdr.dof")
    } else {
        aa
    };
    let after_mb = if has(FramePass::MotionBlur) {
        Some("hdr.mb")
    } else {
        after_dof
    };
    let before_custom = if has(FramePass::ScreenBlur) {
        Some("hdr.blur")
    } else {
        after_mb
    };
    let customs = passes
        .iter()
        .filter(|p| matches!(p, FramePass::CustomPost(_)))
        .count();
    let last = match customs {
        0 => before_custom,
        n => Some(CUSTOM_POST_NAMES[n - 1]),
    };
    LitNames {
        lit,
        main,
        aa,
        after_dof,
        after_mb,
        before_custom,
        last,
    }
}
