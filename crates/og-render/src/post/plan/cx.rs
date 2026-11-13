//! What the declarations of a frame's graph share: its passes, its config
//! and the facts both lead to.

use super::*;

pub(super) struct PlanCx<'a> {
    pub(super) passes: &'a [FramePass],
    pub(super) cfg: &'a GraphConfig,
    pub(super) msaa: bool,
    /// SMAA T2x: the blend writes its own history, which the resolve reads.
    pub(super) smaa_t2x: bool,
    /// The GI's and the reflections' own copy of the lit image when there
    /// is no TAA history.
    pub(super) gi_copy: bool,
    pub(super) prepass: bool,
    /// With contact shadows the AO passes end in `ao.pre`, which the
    /// contact pass copies into `ao`'s r beside the lights' masks.
    pub(super) contact: bool,
    /// The target the AO passes end in ([`ao_target_name`]).
    pub(super) ao_out: &'static str,
    /// With water the AO darkens (`GraphConfig::water_ao`): the prepass
    /// depth with the water's surfaces in it, which the AO reads (lib.rs,
    /// water.rs `draw_ao_depth`); the prepass pass writes it after its own
    /// targets.
    pub(super) ao_depth: bool,
    /// The passes after the tonemap ([`chain_passes`]).
    pub(super) chain: Vec<FramePass>,
    pub(super) names: LitNames,
    /// The lit image after TAA, after the depth of field and after every
    /// effect: the history where [`LitNames`] has `None`.
    pub(super) aa: &'static str,
    pub(super) after_dof: &'static str,
    pub(super) lit: &'static str,
    /// Whether the frame meters its exposure, and whether it blooms.
    pub(super) auto: bool,
    pub(super) bloom: bool,
    /// The last level of the meter's reduction.
    pub(super) metered: String,
    /// What the exposure is read from: the adapted exposure when it adapts.
    pub(super) lum: String,
}

impl<'a> PlanCx<'a> {
    pub(super) fn new(passes: &'a [FramePass], cfg: &'a GraphConfig) -> PlanCx<'a> {
        let has = |p| passes.contains(&p);
        let names = lit_names(passes);
        let metered = luminance_name(meter_levels() - 1);
        let contact = has(FramePass::Contact);
        let prepass = has(FramePass::Prepass);
        PlanCx {
            passes,
            cfg,
            msaa: cfg.samples > 1,
            smaa_t2x: has(FramePass::SmaaResolve),
            gi_copy: (cfg.gi || cfg.ssr) && !cfg.taa,
            prepass,
            contact,
            ao_out: ao_target_name(contact),
            ao_depth: prepass && cfg.water_ao && has(FramePass::Ao),
            chain: chain_passes(passes),
            aa: names.aa.unwrap_or("history-write"),
            after_dof: names.after_dof.unwrap_or("history-write"),
            lit: names.last.unwrap_or("history-write"),
            names,
            auto: has(FramePass::Meter),
            bloom: has(FramePass::BloomPrefilter),
            lum: if has(FramePass::Adapt) {
                "exposure".to_string()
            } else {
                metered.clone()
            },
            metered,
        }
    }

    pub(super) fn has(&self, pass: FramePass) -> bool {
        self.passes.contains(&pass)
    }

    /// The metaball field targets the frame draws.
    pub(super) fn fields(&self) -> &'static [&'static str] {
        const ALL: &[&str] = &FIELD_NAMES;
        &ALL[..(self.cfg.fluid_fields as usize).clamp(1, ALL.len())]
    }
}
